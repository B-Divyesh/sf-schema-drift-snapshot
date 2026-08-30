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

## Try the bundled sample

Run the real binary with no database or account:

```sh
sds demo
```

The command copies two realistic PostgreSQL snapshots into a new temporary
directory. It writes a Markdown review with five classified differences and
prints the sandbox path. Use `--output <directory>` when you want a known
location, or `--json` for a machine-readable result.

The browser version is at
<https://schema-drift-snapshot.sociobot.in/demo/>. It opens with a separate
four-change sample review and never reads or writes production data.

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
workflow. License restore lives on the product site. Checkout remains closed
until the factory registers this product with the Sociobot billing service.
Safety behavior and core exports are never gated.

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
npm ci
npm test                 # Rust tests + site tests
npm run build            # release binary + site -> dist/
npm run build:site       # static site only -> dist/site/
cargo test
cargo clippy --all-targets -- -D warnings
cargo package --allow-dirty
```

To run the PostgreSQL privilege-boundary regression, point the suite at a
disposable database whose test role can create and remove roles:

```sh
SDS_TEST_POSTGRES_ADMIN_URL="postgresql://test-admin:password@127.0.0.1/test-db?sslmode=disable" cargo test --test postgres_read_only
SDS_TEST_MYSQL_ADMIN_URL="mysql://test-admin:password@127.0.0.1/test-db" cargo test --test mysql_read_only
```

The test creates a view as the admin, captures it through a separate
SELECT-only role, changes only its predicate, and requires one ORM-invisible
difference. It also proves that the capture role cannot insert row data.
The MySQL/MariaDB test also proves that a role without `SHOW VIEW` fails with
an incomplete-capture error instead of writing blank view definitions.

The landing/docs site is Vite + vanilla TypeScript. It does not receive or
upload schema text; the interactive sample runs entirely in the browser.
Every user-facing product claim and its exact test command is listed in
`.factory/claims.json`.

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
