# Schema Drift Snapshot

Schema Drift Snapshot (`sds`) is a read-only CLI for developers who need to
explain PostgreSQL or MySQL production drift before anyone attempts a repair.
It captures portable metadata snapshots, compares them, classifies likely
destructive and ORM-invisible differences, and writes a Markdown incident
review. It never reads row data or emits executable repair SQL.

## Install

Build the single binary with stable Rust:

```sh
cargo install --path .
sds --help
```

Prebuilt archives can be attached to GitHub releases by the factory. This
worker does not publish packages or releases.

## Usage

Capture each environment with a read-only database role:

```sh
sds snapshot \
  --url "$STAGING_DATABASE_URL" \
  --output staging.sds.json

sds snapshot \
  --url "$PRODUCTION_DATABASE_URL" \
  --output production.sds.json \
  --redact-names \
  --redaction-key "$INCIDENT_REDACTION_KEY"
```

Create the review artifact:

```sh
sds compare \
  --before staging.sds.json \
  --after production.sds.json \
  --output drift-review.md
```

Use JSON on CI or in another tool:

```sh
sds compare --before expected.json --after actual.json --json
sds check --before expected.json --after actual.json --fail-on high
```

`snapshot` auto-detects `postgres://`, `postgresql://`, and `mysql://` URLs and
only issues catalog queries. `--schema` can be repeated to limit capture.
Snapshots contain table, view (including query definitions), column, index, and
foreign-key metadata—not row data. `--redact-names` replaces identifiers and
definition details with deterministic local hashes so
the same object still matches across snapshots made with the same
`--redaction-key`.

Exit codes are stable: `0` success/no blocking drift, `1` drift reached the
configured `check` threshold, `2` invalid input or connection failure, and `3`
a paid CI feature was requested without an active license. No command prompts,
so the CLI is safe in CI.

## Free and Pro

The free CLI includes live capture, redaction, complete classification,
ownership guidance, Markdown reports, and JSON export. A one-time Pro license
adds configurable policy-based CI gates. Set `SDS_LICENSE` or pass
`--license`; verification is cached for one day and never blocks the free
workflow. Buy and restore links live on the product site. Safety behavior and
core exports are never gated.

## Snapshot format

The public JSON format is versioned as `schema_version: 1`. A minimal fixture:

```json
{
  "schema_version": 1,
  "dialect": "postgresql",
  "captured_at": "2026-08-27T12:00:00Z",
  "source": "expected",
  "redacted": false,
  "objects": []
}
```

The documented command examples and snapshot decoder are covered by tests.

## Develop and verify

```sh
npm install
npm test                 # Rust tests + site tests
npm run build            # release binary + site -> dist/
npm run build:site       # static site only -> dist/site/
cargo test
cargo clippy --all-targets -- -D warnings
cargo package --allow-dirty
```

The landing/docs site is Vite + vanilla TypeScript. It does not receive or
upload schema text; the interactive sample runs entirely in the browser.

## Deploy

The factory deploys `dist/site` to
<https://schema-drift-snapshot.sociobot.in>. It owns registry credentials,
binaries, billing registration, DNS, and infrastructure. See
`.factory/handoff.md` for the verified release state.

## Privacy and security

Database URLs are never written into snapshots. Use a database role restricted
to catalog visibility and retain snapshot files like other infrastructure
metadata. The CLI has no telemetry. The site uses local storage only for a
license token/verdict and makes one direct verification request to Sociobot.

## License

MIT © 2026 Sociobot (Param Factory). See [LICENSE](LICENSE).
