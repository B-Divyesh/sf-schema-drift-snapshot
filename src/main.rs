use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use schema_drift_snapshot::{
    capture,
    diff::{self, Risk},
    license,
    model::Snapshot,
    redact, report,
};
use serde_json::json;

#[derive(Debug, Parser)]
#[command(
    name = "sds",
    version,
    about = "Explain PostgreSQL/MySQL schema drift before attempting a repair",
    long_about = "Capture read-only schema metadata, compare two environments, and write a human-reviewable risk report. SDS never reads row data or generates executable repair SQL.",
    arg_required_else_help = true
)]
struct Cli {
    /// One-time Pro license token. Can also be set with SDS_LICENSE.
    #[arg(long, env = "SDS_LICENSE", global = true, hide_env_values = true)]
    license: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Capture metadata through read-only PostgreSQL or MySQL catalog queries.
    Snapshot {
        /// Database URL. Supported schemes: postgres, postgresql, mysql.
        #[arg(long, env = "DATABASE_URL", hide_env_values = true)]
        url: String,

        /// Destination for the portable snapshot JSON.
        #[arg(short, long)]
        output: PathBuf,

        /// Limit capture to this schema/database. Repeat to include more.
        #[arg(long)]
        schema: Vec<String>,

        /// Replace identifiers and SQL definitions with deterministic hashes.
        #[arg(long)]
        redact_names: bool,

        /// Secret shared across captures that must compare after redaction.
        #[arg(long, env = "SDS_REDACTION_KEY", hide_env_values = true)]
        redaction_key: Option<String>,

        /// Print a machine-readable command result to stdout.
        #[arg(long)]
        json: bool,
    },

    /// Compare expected and observed snapshots and produce a safe review.
    Compare {
        /// Expected/before snapshot.
        #[arg(long)]
        before: PathBuf,

        /// Observed/after snapshot.
        #[arg(long)]
        after: PathBuf,

        /// Write the Markdown review here. Prints to stdout when omitted.
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Print the complete review as JSON instead of Markdown.
        #[arg(long)]
        json: bool,
    },

    /// Pro: apply a drift-risk threshold for CI without interactive prompts.
    Check {
        /// Expected/before snapshot.
        #[arg(long)]
        before: PathBuf,

        /// Observed/after snapshot.
        #[arg(long)]
        after: PathBuf,

        /// Exit 1 when drift at or above this level is present.
        #[arg(long, value_enum, default_value = "high")]
        fail_on: RiskArg,

        /// Print the review as JSON.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum RiskArg {
    Low,
    Medium,
    High,
}

impl From<RiskArg> for Risk {
    fn from(value: RiskArg) -> Self {
        match value {
            RiskArg::Low => Self::Low,
            RiskArg::Medium => Self::Medium,
            RiskArg::High => Self::High,
        }
    }
}

fn write_output(path: &Path, contents: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .with_context(|| format!("could not create {}", parent.display()))?;
    }
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, contents)
        .with_context(|| format!("could not write {}", temporary.display()))?;
    fs::rename(&temporary, path)
        .with_context(|| format!("could not finalize {}", path.display()))?;
    Ok(())
}

fn load_review(before: &Path, after: &Path) -> Result<diff::Review> {
    diff::compare(&Snapshot::read(before)?, &Snapshot::read(after)?)
}

fn run(cli: Cli) -> Result<u8> {
    match cli.command {
        Command::Snapshot {
            url,
            output,
            schema,
            redact_names,
            redaction_key,
            json: as_json,
        } => {
            let mut snapshot = capture::capture(&url, &schema)?;
            if redact_names {
                let key = redaction_key.as_deref().ok_or_else(|| {
                    anyhow::anyhow!(
                        "--redact-names requires --redaction-key or SDS_REDACTION_KEY so captures remain comparable"
                    )
                })?;
                snapshot = redact::redact(snapshot, key);
            }
            let encoded = serde_json::to_vec_pretty(&snapshot)?;
            write_output(&output, &encoded)?;
            if as_json {
                println!(
                    "{}",
                    serde_json::to_string(&json!({
                        "ok": true,
                        "output": output,
                        "dialect": snapshot.dialect,
                        "objects": snapshot.objects.len(),
                        "redacted": snapshot.redacted
                    }))?
                );
            } else {
                println!(
                    "Captured {} {} objects to {}{}",
                    snapshot.objects.len(),
                    snapshot.dialect,
                    output.display(),
                    if snapshot.redacted {
                        " (names redacted)"
                    } else {
                        ""
                    }
                );
            }
            Ok(0)
        }
        Command::Compare {
            before,
            after,
            output,
            json: as_json,
        } => {
            let review = load_review(&before, &after)?;
            if let Some(path) = &output {
                write_output(path, report::markdown(&review).as_bytes())?;
                if !as_json {
                    println!(
                        "Wrote {} classified differences to {}",
                        review.summary.total,
                        path.display()
                    );
                }
            }
            if as_json {
                println!("{}", serde_json::to_string_pretty(&review)?);
            } else if output.is_none() {
                print!("{}", report::markdown(&review));
            }
            Ok(0)
        }
        Command::Check {
            before,
            after,
            fail_on,
            json: as_json,
        } => {
            let status = match license::verify(cli.license.as_deref()) {
                Ok(status) if status.valid => status,
                Ok(status) => {
                    if let Some(notice) = status.notice {
                        eprintln!("{notice}");
                    }
                    return Ok(3);
                }
                Err(error) => {
                    eprintln!("{error:#}");
                    return Ok(3);
                }
            };
            if let Some(notice) = status.notice {
                eprintln!("{notice}");
            }
            let review = load_review(&before, &after)?;
            if as_json {
                println!("{}", serde_json::to_string_pretty(&review)?);
            } else {
                println!(
                    "{} differences: {} high, {} medium, {} low",
                    review.summary.total,
                    review.summary.high,
                    review.summary.medium,
                    review.summary.low
                );
            }
            Ok(u8::from(review.reaches(fail_on.into())))
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_contract_parses_documented_compare() {
        let cli = Cli::try_parse_from([
            "sds",
            "compare",
            "--before",
            "expected.json",
            "--after",
            "actual.json",
            "--json",
        ]);
        assert!(cli.is_ok());
    }

    #[test]
    fn snapshot_requires_a_url() {
        let cli = Cli::try_parse_from(["sds", "snapshot", "--output", "out.json"]);
        assert!(cli.is_err());
    }
}
