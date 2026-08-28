# Independent verification 3 — FAIL

Work order: `schema-drift-snapshot-verify-3`  
Candidate commit: `75ec662ff994bcf4661b3cd9cfd6cb74406ed626`  
Verified URL: <https://schema-drift-snapshot.sociobot.in/>  
Date: 2026-08-28 UTC

## Release decision

**FAIL — do not release this candidate.** The deployment-only service-worker
failure reported previously is fixed and the live deployment byte-matches this
candidate. The product nevertheless misses a material kind of in-scope schema
drift: changes to the definition of an existing database view. This conflicts
with the brief's database/ORM-boundary use case, especially its explicit
view/ORM evidence, and can produce a false `No drift detected` review.

## Defects

### P1 — existing view definitions are not captured or compared

Both live-capture adapters query only a view's schema, name, and table type.
PostgreSQL capture selects `table_schema, table_name, table_type` from
`information_schema.tables` and saves only `{"table_type": ...}` for the
view (`src/capture.rs:69-91`). MySQL does the equivalent against
`information_schema.TABLES` (`src/capture.rs:183-201`). Neither captures the
view query/definition.

Consequently, a view whose name, type, and exposed columns do not change, but
whose predicate, joins, expressions, or security-relevant semantics do, yields
the same SDS object in both snapshots. `compare` sees identical `details` and
emits no change. The classifier has a `Modified View` branch, but snapshots
produced by either adapter cannot reach it for a definition-only change.

This is a core false-negative for a read-only production drift-review tool and
is directly relevant to ORM-invisible database views. Capture a dialect-specific
view definition (and redact it with the existing detail-redaction path), then
add PostgreSQL and MySQL integration coverage for a definition-only view
change.

### P2 — fresh mobile Lighthouse LCP exceeds the stated budget

Fresh Lighthouse 13 mobile/performance-mode evidence against the live URL was:
Performance **92**, Accessibility **100**, Best Practices **100**, SEO **100**,
CLS **0**, TBT **31 ms**, and LCP **2,666 ms**. The stated performance contract
sets LCP below 2,500 ms. The aggregate Lighthouse score passes, but this fresh
measurement misses the explicit LCP threshold. Re-measure after optimizing or
otherwise stabilizing the initial render, and record a passing cold-load result.

## Passing evidence

### Clean checkout, build, tests, and package

- Began at exactly `75ec662ff994bcf4661b3cd9cfd6cb74406ed626` with a clean
  worktree. `npm ci` completed with 0 audited vulnerabilities.
- `npm test` passed: 14 Rust unit/integration tests, TypeScript check, six site
  contracts, and the 12-project Playwright run (11 pass plus the one intended
  desktop skip). The suite includes normal comparison, invalid JSON recovery,
  license-return storage, mobile overflow, legal pages, and offline reload.
- `cargo fmt --all -- --check` and
  `cargo clippy --all-targets -- -D warnings` passed.
- The exact production command `npm run build` passed and created
  `dist/bin/sds` (6,399,072 bytes) and `dist/site`. `cargo package --allow-dirty`
  passed.
- A clean temporary consumer installed
  `target/package/schema-drift-snapshot-0.1.0` with `cargo install --debug
  --path ... --root <temporary-root> --target-dir target`. Installed `sds 0.1.0`
  successfully produced the documented fixture review (5 changes: 4 high, 1
  medium, 3 destructive, 1 ORM-invisible).

### CLI workflow and safety boundaries

- The release binary `dist/bin/sds 0.1.0` compared the supplied expected and
  observed fixtures and wrote a Markdown report with five classified changes.
  A scan found no `DROP TABLE`, `ALTER TABLE`, `CREATE TABLE`, `DELETE FROM`, or
  `UPDATE` SQL.
- Comparing the expected fixture to itself rendered `No drift detected`.
  Unsupported `sqlite:` input, a missing snapshot, and `--redact-names` without
  a key each exited 2 with actionable errors; unlicensed `check` exited 3.
  The missing redaction key was rejected before the attempted PostgreSQL
  connection and no output snapshot was created.
- `sds --help` documents its read-only/no-row-data/no-repair-SQL boundary;
  documented JSON output and stable exit paths work without prompts.
- No PostgreSQL or MySQL server/client executable exists in this disposable
  environment. Actual database-driver capture therefore remains an
  environmental coverage gap; source inspection identified the P1 above.

### Live deployment, PWA, browser, accessibility, and privacy

- Live `/` and `sw.js` exactly match this candidate's production build:
  root SHA-256 `3bb00cf120bb05d75d66048d5d5fe9ed7402b7d042aa0e05d813f314e9bf8ce2`;
  worker SHA-256 `2d8eab5d36e112d9341618ca5d38890b275c9cbb006499ff5bc0bd36eb421c57`.
- The live worker has a 14-URL shell, excludes `_headers`, `sw.js`, and
  `staticwebapp.config.json`, and every shell URL returned 200. A clean
  Chromium profile received a controller and successfully reloaded `/` offline
  with the expected title and one `h1`.
- Fresh desktop and 390 x 844 mobile checks found `lang=en`, one `h1`, one
  `main`, no page/console errors, and no horizontal overflow (390/390 px on
  mobile). Normal local comparison produced four browser-demo changes;
  malformed JSON displayed the announced actionable error; keyboard Enter on
  Reset returned to the empty state.
- Axe found zero serious or critical violations. First Tab focused the skip link
  with `rgb(141, 48, 37) solid 3px` outline. Reduced-motion computed a
  `0.00001s` button transition and `scroll-behavior: auto`.
- Normal first load made requests only to
  `schema-drift-snapshot.sociobot.in`; no analytics, telemetry, remote fonts,
  or third-party scripts were found. The only product API source path is the
  documented Sociobot license verification endpoint, reached only when a
  license is supplied. Privacy and terms routes are present.
- Live responses include CSP, Permissions-Policy, X-Frame-Options, HSTS,
  Referrer-Policy, and X-Content-Type-Options. Hashed assets use
  `Cache-Control: public, max-age=31536000, immutable`; `/sw.js` uses
  `Cache-Control: no-cache`.
- Production bytes are within declared transfer budgets: JS 8,056 bytes, CSS
  13,863 bytes, fonts 0 bytes, and hero WebP 53,184 bytes.

## Retest

After capturing and comparing view definitions for both supported dialects,
and obtaining a cold mobile LCP under 2.5 seconds, run:

```sh
npm ci
npm test
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
npm run build
cargo package --allow-dirty
```

Then install the packaged crate in a new temporary `cargo install --root`,
exercise definition-only PostgreSQL and MySQL view changes with read-only
credentials, and repeat the live identity/PWA/header/mobile/axe/Lighthouse
checks.
