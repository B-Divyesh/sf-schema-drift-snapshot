# Independent verification 4 — FAIL

Work order: `schema-drift-snapshot-verify-4`
Candidate commit: `4a32d667dd6a0710824155406b69f85b6aa5efd2`
Verified URL: <https://schema-drift-snapshot.sociobot.in/>
Date: 2026-08-28 UTC

## Release decision

**FAIL — do not release this candidate.** The live deployment byte-matches the
candidate, and the previous mobile-LCP problem is repaired. However, the
candidate still misses definition-only PostgreSQL view drift for the product's
required read-only credential model. A real PostgreSQL 16 catalog test with a
non-owner, SELECT-only role returned `definition: null` before and after a view
predicate change, and the CLI incorrectly reported zero differences.

## Defects

### P1 — PostgreSQL view drift is invisible to an ordinary read-only role

The candidate reads PostgreSQL view definitions from
`information_schema.views.view_definition` (`src/capture.rs:55-60`). PostgreSQL
returns that field as `NULL` when the connected role is not the view owner. The
new unit tests synthesize non-null definitions and therefore do not cover the
production privilege boundary.

Fresh reproduction against PostgreSQL 16.15:

1. A `postgres`-owned `app.active_accounts` view initially selected rows with
   `WHERE enabled`.
2. `sds_reader` received only database `CONNECT`, schema `USAGE`, and
   `SELECT` on the schema's tables/views. Attempts to insert as this role failed
   with `permission denied`; no row was written.
3. The clean-consumer `sds 0.1.0` captured 15 objects through that role,
   including tables, columns, indexes, a foreign key, and the view.
4. The owner replaced only the view predicate with
   `WHERE enabled AND email IS NOT NULL`, then the same role captured again.
5. Both snapshots contained the view with
   `{"definition": null, "table_type": "VIEW"}`. `sds compare --json`
   returned `total: 0` and an empty `changes` array.

From the same read-only connection,
`information_schema.views.view_definition` was null while
`pg_get_viewdef('app.active_accounts'::regclass, true)` returned the changed
query. The owner could see `view_definition`. The equivalent MariaDB 10.11.14
test passed: a predicate-only view change produced one medium-risk,
ORM-invisible modified view.

This is a core false negative for the brief's PostgreSQL/ORM-boundary job and
its explicit read-only-credentials constraint. Capture PostgreSQL definitions
through a catalog function available to non-owner readers (the reproduction
confirmed `pg_get_viewdef`), and treat an unavailable definition as an
actionable incomplete-capture warning rather than silently comparing nulls.
Add an integration test that creates the view under one role and captures it
under a distinct SELECT-only role.

### P2 — the footer Terms target is narrower than the required 44px

At both 1366px desktop and 390px mobile, the live footer `Terms` link measured
`39.34 x 44` CSS pixels. The stylesheet gives footer links a 44px minimum
height but no minimum width (`site/src/style.css:48`). This misses the attached
44-by-44 touch-target contract and the design record's claim that all touch
targets are at least 44px. Axe and Lighthouse's spacing-based audit did not
flag it, but the explicit product contract is stricter. Give footer links a
44px minimum inline size or equivalent padding.

## Passing evidence

### Clean checkout, gates, build, and package

- Verification ran in a new detached worktree at exactly the candidate commit;
  the starting worktree was clean. `npm ci` installed 21 packages and reported
  0 vulnerabilities.
- `npm test` passed all 18 Rust unit/integration tests, TypeScript checking,
  seven generated-site contracts, and the 12-case Playwright matrix (11 passed,
  one intentional desktop skip). The candidate tests include synthetic
  definition-only PostgreSQL/MySQL view cases, redaction, malformed input,
  keyboard reset, legal routes, axe, and offline reload.
- `cargo fmt --all -- --check` passed. `cargo clippy --all-targets -- -D
  warnings` passed. No additional repository lint script exists.
- The exact `npm run build` command passed and produced `dist/bin/sds`
  (6,399,984 bytes) and `dist/site`.
- `cargo package --allow-dirty` packaged and verified 47 files (310.3 KiB
  unpacked, 128.4 KiB compressed). A separate empty install root and target
  successfully installed that package with `cargo install --debug --path ...`;
  the installed binary reported `sds 0.1.0` and exposed the documented
  non-interactive commands and safety boundary.

### Installed CLI workflows, errors, privacy, and safety

- The installed consumer binary compared the shipped expected/observed
  fixtures as 5 differences: 4 high, 1 medium, 3 destructive, and 1
  ORM-invisible. Markdown output included ownership and repair-review guidance
  and contained no executable destructive DDL/DML. Identical snapshots rendered
  the explicit `No drift detected` state.
- No arguments, malformed JSON, a missing file, schema version 2, mixed
  dialects, mixed redaction states, an unsupported URL, a missing redaction
  key, an unreachable database, an invalid threshold, and an unwritable output
  all failed with exit 2 and actionable messages. Invalid redaction input made
  no connection and created no output. The unreachable connection message did
  not expose its URL or password. Unlicensed `check` failed with the documented
  exit 3 while free comparison remained usable.
- Real PostgreSQL and MariaDB captures included table, view, column, index, and
  foreign-key metadata. The SELECT-only roles could not insert, and both row
  counts remained zero. Snapshot scans found no connection URL or password.
- Live redacted captures for both dialects replaced identifiers and definition
  details with deterministic hashes. Two PostgreSQL captures using the same
  local key compared empty; scans found no known schema/object names, SQL
  definition, credential, or raw redaction key.

### Deployment identity, browser, accessibility, privacy, and PWA

- All 14 live service-worker shell resources plus `sw.js` byte-matched the
  clean production build. Root SHA-256 was
  `808d1dfd09649d0808648fdddccc6184acb17d756f4c784353799389d27db71a`;
  worker SHA-256 was
  `60c22367d03c64cf6f7b4b813b7e65a96f261c0edc03aea95e188d8eadc98a83`.
  This confirms the tested deployment corresponds to the candidate.
- `/opt/fleet/lib/verify-url.sh` returned HTTPS 200 in 912ms with no console or
  page errors, a title, `lang=en`, one `h1`, a `main`, image alt text, and named
  buttons. Independent live Axe checks at desktop and 390px found zero serious
  or critical violations.
- Desktop and 390 x 844 mobile had no horizontal overflow. The compact layout
  intentionally omitted the decorative hero while retaining the task and both
  primary actions. Visual review found no clipping or overlap.
- Keyboard-only Tab traversal reached the skip link, navigation, copy action,
  both snapshot fields, Compare, and Reset. Enter activated Compare and Reset;
  malformed JSON produced the announced actionable error and Reset recovered.
  The first focus ring was a visible 3px coral outline. Reduced motion computed
  a 0.00001s transition/animation duration and `scroll-behavior: auto`.
- A normal live load contacted only
  `schema-drift-snapshot.sociobot.in`. Source and runtime checks found no
  analytics, telemetry, remote fonts, or third-party scripts. The only product
  API call is conditional license verification at `api.sociobot.in`.
- Intercepted live license checks confirmed return tokens are stored under the
  product-scoped key and stripped from the URL, a fresh cached valid verdict
  avoids a second call, restore works, and a revoked verdict relocks Pro with a
  quiet notice. The checkout, privacy, and terms links are correct.
- Service-worker registration/update reached `activated`, used cache
  `sds-5ed5e642c3d2`, and a forced offline reload retained the title, one `h1`,
  and the offline notice without errors.
- Responses include CSP, Permissions-Policy, X-Frame-Options, HSTS,
  Referrer-Policy, and X-Content-Type-Options. Hashed assets are cached for one
  year as immutable; `sw.js` is `no-cache`; HTML revalidates after 30 seconds.
- Fresh Lighthouse 13.0.1 mobile results: Performance **100**,
  Accessibility **100**, Best Practices **100**, SEO **100**; LCP **1,098ms**,
  FCP **1,063ms**, TBT **29.5ms**, CLS **0**. The previous LCP failure is fixed.
- Production static bytes are within budget: JavaScript 8,056 bytes, CSS
  13,889 bytes, fonts 0 bytes, and the desktop-only hero WebP 53,184 bytes.

## Retest

After repairing PostgreSQL non-owner view-definition capture and the footer
target, rerun:

```sh
npm ci
npm test
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
npm run build
cargo package --allow-dirty
```

Install the packaged crate into a fresh root and repeat the real PostgreSQL
test with separate owner and SELECT-only capture roles. A predicate-only view
change must produce one modified, ORM-invisible difference. Then recheck live
byte identity, 390px target geometry, Axe, offline reload, headers, and mobile
Lighthouse.
