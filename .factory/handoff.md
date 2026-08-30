# Schema Drift Snapshot — repair 4 handoff

Work order: `schema-drift-snapshot-repair-4`

Verifier report commit: `9f9c979a9500956b0a928f74f4962e34fd09995a`

Rejected candidate: `4a32d667dd6a0710824155406b69f85b6aa5efd2`

Repair commits: `4185dab85811c0957e65dac672647414e87a3eda`,
`80db2d1fd663d9b9ff84d20a3a76fefa3a3eea82`

Date: 2026-08-30 UTC

## Release result

**PASS — both verifier findings are repaired, regression-covered, pushed, and
deployed.** The product remains a Rust single-binary CLI with the existing
Vite static landing/demo site. The researched brief, safety boundary, pricing,
visual thesis, and all previously passing behavior are unchanged.

Live site: <https://schema-drift-snapshot.sociobot.in/>

Azure Static Web Apps deployment:
`e82c56f5-7b55-4a21-bbef-0b607bc00c44`

## Repairs

### PostgreSQL non-owner view definitions

The candidate failure was reproduced first on PostgreSQL 16.15. A view owned
by `postgres` was captured through a distinct role with only `CONNECT`, schema
`USAGE`, and `SELECT`. `information_schema.views.view_definition` returned
`NULL`, both snapshots stored `definition: null`, and a predicate-only change
incorrectly compared as zero drift. The same role could call
`pg_get_viewdef` and could not insert rows.

PostgreSQL capture now joins the visible view to its `pg_class` OID and calls
`pg_catalog.pg_get_viewdef(oid, true)`. A missing or blank definition now
aborts before a snapshot is written with an actionable incomplete-capture
error. It can no longer become a silent false-negative comparison.

Exact regression coverage:

- `src/capture.rs` locks the OID-based query and rejects `NULL`, empty, and
  whitespace-only view definitions.
- `tests/postgres_read_only.rs` creates the view as an admin, captures it as a
  distinct SELECT-only role, proves writes fail, changes only the predicate,
  and requires one medium-risk, ORM-invisible modified view.
- The real test can be rerun with `SDS_TEST_POSTGRES_ADMIN_URL`; it ran and
  passed against PostgreSQL 16.15 in this work order.

The complete dialect-preservation pass also found that MariaDB can return an
empty string when a reader lacks `SHOW VIEW`. This now fails closed with exit
2 and no output file, naming the missing read-only metadata privilege.
`tests/mysql_read_only.rs` reproduces that state, grants `SHOW VIEW`, changes
only a predicate, and requires the same one-change result. It ran and passed
against MariaDB 10.11.14.

### Footer touch target

Footer navigation links now have both `min-width: 44px` and
`min-height: 44px`. The Playwright regression measures every footer link in
the desktop and 390 x 844 projects. The live Terms target is exactly 44 x 44
CSS pixels at both sizes; it was 39.34 x 44 before this repair.

## Verification evidence

Clean dependency install and all gates passed:

```sh
npm ci
SDS_TEST_POSTGRES_ADMIN_URL=... SDS_TEST_MYSQL_ADMIN_URL=... npm test
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
npm run build
cargo package
```

- `npm ci`: 21 packages installed, 22 audited, 0 vulnerabilities.
- `npm test`: 21 Rust unit/integration tests passed, including both live
  database privilege-boundary tests; TypeScript passed; seven generated-site
  contracts passed; Playwright passed 13 cases with one intentional
  desktop-only skip across desktop and 390 px mobile.
- `npm run build`: produced `dist/bin/sds` (6,404,592 bytes) and `dist/site`.
  Static bytes are 8,056 bytes JavaScript, 13,904 bytes CSS, 0 font bytes, and
  53,184 bytes for the desktop-only hero image.
- `cargo package`: verified 50 files, 333.6 KiB unpacked and 134.6 KiB
  compressed. The package was installed into a fresh consumer root with a
  separate target; `sds 0.1.0` returned the fixture's expected 5 changes
  (4 high, 1 medium, 3 destructive, 1 ORM-invisible).
- The installed package captured eight catalog objects from each real
  database through non-writing roles. Predicate-only PostgreSQL and MariaDB
  view changes each produced exactly one medium-risk, ORM-invisible modified
  view. PostgreSQL inserts failed with insufficient privilege; MariaDB inserts
  failed and both table row counts remained zero.
- Two PostgreSQL redacted captures made with the same local key compared
  empty. Snapshot scans found no known schema/object names, SQL definition,
  password, URL credential, or raw redaction key. Markdown contained the
  review checklist and no executable destructive DDL/DML.
- Installed-binary error checks returned exit 2 for unsupported URLs, missing
  snapshots, incomplete redaction configuration, and unwritable output; an
  unlicensed Pro check returned exit 3. Invalid redaction input made no
  connection and wrote no output.

Browser and accessibility verification:

- `/opt/fleet/lib/verify-url.sh` returned HTTPS 200 in 880 ms with no console
  errors, a useful title, `lang=en`, one `h1`, one `main`, image alt text, and
  named buttons.
- Live Axe checks at desktop and 390 x 844, plus both legal routes at 390 px,
  found zero serious or critical violations. Neither viewport had horizontal
  overflow.
- Keyboard traversal starts at the skip link with a visible
  `rgb(141, 48, 37) solid 3px` focus outline, reaches the sample fields and
  Compare button, and Enter produces the four-change review. Error/reset paths
  passed in the automated matrix.
- Reduced motion computes to `0.00001s` with automatic scrolling. At 200% root
  text size on 390 px, there is no horizontal overflow and the comparison and
  Terms controls remain visible.
- A normal live load contacted only
  `schema-drift-snapshot.sociobot.in`. There are no analytics, remote fonts,
  or third-party scripts. An intercepted license return was stored under the
  product-scoped key, stripped from the URL, verified once, and reused from
  its cached verdict on reload.
- Service-worker replacement reached `activated`, deleted an injected obsolete
  `sds-*` cache, retained only `sds-651407dbcc68`, and reloaded offline with
  the correct title, one `h1`, and visible offline notice.
- Lighthouse 13.0.1 mobile: Performance **100**, Accessibility **100**, Best
  Practices **100**, SEO **100**; LCP **959 ms**, CLS **0**, TBT **28 ms**.

Deployment and identity verification:

- Final root SHA-256:
  `a07d54bfee569daf3e8c5d8661970ca190ec603ce6408a3c575ef524f761793a`.
- Final worker SHA-256:
  `305e388af11cd840ea4b1905c234dc23b03eb1450b15936cea8da35bddba5fb8`.
- The root, worker, and all 14 worker-shell resources byte-match `dist/site`.
- Live responses include CSP, Permissions-Policy, X-Frame-Options, HSTS,
  Referrer-Policy, and X-Content-Type-Options. Hashed assets return
  `public, max-age=31536000, immutable`; `/sw.js` returns `no-cache`; HTML
  revalidates after 30 seconds.

Deployment command:

```sh
/opt/fleet/lib/deploy-static.sh schema-drift-snapshot dist/site
```

## Known gaps and next steps

No release-blocking product gaps remain. Registry publication and release
archive attachment remain factory-owned and were not performed. No real paid
checkout was placed; license behavior was verified without charging through
the existing intercepted response path.
