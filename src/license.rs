use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const PRODUCT_SLUG: &str = "schema-drift-snapshot";
const CACHE_SECONDS: u64 = 86_400;

#[derive(Debug, Deserialize)]
struct VerifyResponse {
    valid: bool,
    reason: String,
    #[allow(dead_code)]
    expires_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LicenseCache {
    token_id: String,
    valid: bool,
    reason: String,
    checked_at: u64,
}

#[derive(Debug)]
pub struct LicenseStatus {
    pub valid: bool,
    pub notice: Option<String>,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn token_id(token: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(token.as_bytes());
    hex::encode(digest.finalize())[..16].to_owned()
}

fn cache_path() -> Option<PathBuf> {
    if let Some(root) = std::env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(root).join("schema-drift-snapshot/license.json"));
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|root| root.join(".config/schema-drift-snapshot/license.json"))
}

fn read_cache() -> Option<LicenseCache> {
    let path = cache_path()?;
    let contents = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&contents).ok()
}

fn write_cache(cache: &LicenseCache) {
    let Some(path) = cache_path() else { return };
    let Some(parent) = path.parent() else { return };
    if std::fs::create_dir_all(parent).is_ok()
        && let Ok(contents) = serde_json::to_vec(cache)
    {
        let _ = std::fs::write(path, contents);
    }
}

pub fn verify(token: Option<&str>) -> Result<LicenseStatus> {
    let token = token
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            anyhow!("Pro CI checks need a license; buy or restore one on the product site")
        })?;
    let id = token_id(token);
    let cached = read_cache().filter(|entry| entry.token_id == id);
    if let Some(entry) = &cached
        && now().saturating_sub(entry.checked_at) < CACHE_SECONDS
    {
        return Ok(LicenseStatus {
            valid: entry.valid,
            notice: (!entry.valid).then(|| format!("license is not active ({})", entry.reason)),
        });
    }

    let base = std::env::var("SDS_BILLING_BASE_URL")
        .unwrap_or_else(|_| "https://api.sociobot.in".to_owned());
    let url = format!("{base}/api/v1/products/{PRODUCT_SLUG}/verify");
    let response = ureq::get(&url)
        .query("license", token)
        .call()
        .and_then(|mut response| response.body_mut().read_to_string())
        .map_err(|error| anyhow!(error.to_string()))
        .and_then(|body| {
            serde_json::from_str::<VerifyResponse>(&body)
                .context("billing service returned an unreadable response")
        });

    match response {
        Ok(response) => {
            write_cache(&LicenseCache {
                token_id: id,
                valid: response.valid,
                reason: response.reason.clone(),
                checked_at: now(),
            });
            Ok(LicenseStatus {
                valid: response.valid,
                notice: (!response.valid)
                    .then(|| format!("license is not active ({})", response.reason)),
            })
        }
        Err(error) => match cached {
            Some(entry) if entry.valid => Ok(LicenseStatus {
                valid: true,
                notice: Some(format!(
                    "offline: using the last valid license verdict ({error})"
                )),
            }),
            _ => Err(error).context("license could not be verified; reconnect and retry"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_identifier_does_not_store_plaintext() {
        let id = token_id("secret-license");
        assert_eq!(id.len(), 16);
        assert!(!id.contains("secret"));
    }
}
