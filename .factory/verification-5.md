# Independent verification 5 — FAIL

Work order: `schema-drift-snapshot-verify-5`

Candidate commit: `5d8b4853a82d68028b71d21f0127e7165039baaf`

Verified URL: <https://schema-drift-snapshot.sociobot.in/>

Date: 2026-08-30 UTC

## Release decision

**FAIL — do not release this candidate.** The live deployment byte-matches the
candidate and the core CLI job works against fresh PostgreSQL 16.15 and MariaDB
10.11.14 instances. However, the mandatory first gate failed before any other
QA: `.factory/claims.json` does not exist. The cold first screen also fails the
explicit acceptance test, the required CLI/demo sandbox is absent, the paid
checkout is currently a 404, and 200% text resize causes horizontal overflow.

This result comes from fresh evidence. It does not rely on the builder's prior
deployment report.

## Mandatory gates run first

### Claims gate — FAIL

The very first repository command attempted to read `.factory/claims.json` and
returned:

```text
sed: can't read .factory/claims.json: No such file or directory
```

Consequently there are no listed claim commands to run, no `@claim:*` tests,
and no claim evidence produced through the required demo entry point. The work
order explicitly makes a missing manifest release-blocking.

The omission is substantive, not clerical. Claim-like copy appears throughout
the live site and README, including “No row data,” “No repair SQL,” “No schema
upload,” “Never uploaded,” “Input stays in this tab and is never sent
anywhere,” no telemetry, deterministic redaction, offline reopen, and
once-daily license verification. None is inventoried in a claims manifest.

### Cold first-read gate — FAIL

Opened the live root in a fresh browser context at 1366 x 900 and 390 x 844.

- What it does is understandable: compare PostgreSQL/MySQL catalog differences
  and explain drift before repair.
- Who it is for is not stated in plain words on the first screen. The lede says
  “your team” rather than naming developers with database/ORM migrations.
- The visually primary action is “Install the CLI.” The secondary action says
  “Try the local demo,” not the required “Try it with sample data.”
- The actions were in the first viewport (desktop y 824–873; mobile y 548–656),
  but there is no first-screen one-click action with the required sample-data
  wording and expectation.

Under the work order, either first-read miss is an automatic FAIL.

## Defects

### P0 — Required claim inventory and claim tests are absent

`.factory/claims.json` is missing. There are no tagged claim tests, and the
landing/README claims are therefore all unlisted. Ordinary test success cannot
substitute for this release gate.

### P1 — Required CLI sample demo and sandbox are not shipped

- The clean-consumer `sds demo` command exits 2 with `unrecognized subcommand
  'demo'`; `--help` exposes only `snapshot`, `compare`, and `check`.
- The landing page has a static shell snippet, not a recording of the real
  binary using the bundled sample. It says “Wrote 6 classified differences”
  while the shipped fixture produces 5.
- `/demo` is only the root navigation fallback. It retains the root title and
  h1, has no persistent “Demo — sample data, nothing is saved” banner, no
  “Start for real,” and no separate sandbox state.
- `.factory/demo.md` is absent.

This fails both the CLI-specific demo contract and the mandatory cold-page
sample demo.

### P1 — The advertised $49 purchase is unavailable

The live “Buy Pro through Sociobot” URL returned HTTP 404 on a fresh GET:

```json
{"error":"enabled factory product","status":404}
```

The page advertises a $49 one-time Pro product, but a buyer cannot reach hosted
checkout. This appears to be external product-registration/deployment state;
it is still a live release blocker.

### P1 — 200% text resize creates horizontal scrolling

At a 390px viewport, setting the root text size to 200% increased document
width to 494px. The “portable snapshot” / MySQL workflow strip reached x=494.1.
The normal-size page is 390px wide. This violates the attached accessibility
baseline requiring 200% resize without loss or horizontal overflow.

### P2 — Required route and metadata structure is incomplete

- A random unknown route returns the root landing page with HTTP 200; no real
  designed 404 route exists.
- `/demo` has no route-specific `Demo — Schema Drift Snapshot` title and is
  absent from `sitemap.xml`.
- Root and legal pages have no canonical link, Open Graph metadata, Twitter
  card, 1200 x 630 social image, or apple-touch icon.
- The footer has no version/build identity.

### P2 — A cached invalid license loses its notice on reload

Restoring an invalid token showed “License no longer active (invalid)” and made
one API request. A reload correctly made no second request, but the status
became blank for the rest of the cache period. The paid-unlock contract requires
a quiet inactive-license notice with the buy link while features remain locked.

### P2 — Required copy audit is absent

`.factory/copy-audit.md` does not exist, so there is no sentence word-count,
banned-word, or terminology audit required by the plain-words contract.

## Passing evidence

### Clean checkout, install, tests, build, and package

- The worktree began clean at the exact candidate. `origin/main` also pointed
  to the candidate.
- `npm ci`: 21 packages installed, 22 audited, 0 vulnerabilities.
- `npm test`: 21 Rust tests passed, TypeScript passed, the site production build
  passed, 7 site contract tests passed, and Playwright reported 13 passed with
  1 intentional project skip. The two database tests initially returned early
  because admin URLs were absent; they were then rerun against real databases.
- Fresh PostgreSQL and MariaDB runs: both real privilege-boundary integration
  tests passed.
- `npm run typecheck`, `cargo fmt --all -- --check`, and
  `cargo clippy --all-targets -- -D warnings` passed.
- Exact `npm run build` passed and produced `dist/bin/sds` (6,404,592 bytes)
  and `dist/site`.
- `cargo package` verified 50 files (328.9 KiB unpacked, 133.5 KiB compressed).
- The packaged crate installed into a separate empty consumer root. The
  installed CLI reported `sds 0.1.0` and exposed useful non-interactive help,
  JSON output, and stable errors.

### Packaged CLI and real-database workflows

- The shipped expected/observed fixtures produced 5 differences: 4 high, 1
  medium, 3 destructive, and 1 ORM-invisible. Markdown contained ownership and
  repair-review guidance and no executable DDL/DML. Identical snapshots showed
  `No drift detected`.
- No arguments, empty/malformed JSON, missing input, schema version 2, mixed
  dialects, incomplete redaction, unsupported URL, unreachable database,
  invalid threshold, and unwritable output all failed with exit 2 and useful
  messages. The connection failure did not expose its URL or password.
- Unlicensed Pro `check` returned the documented exit 3 while free comparison
  remained available.
- An installed consumer binary captured 8 objects from each of PostgreSQL
  16.15 and MariaDB 10.11.14 through separate read-only roles. Insert attempts
  failed and both table row counts remained zero.
- A predicate-only view change produced exactly 1 medium-risk, ORM-invisible
  modified view in each dialect. PostgreSQL worked for a non-owner reader.
  MariaDB first failed closed with exit 2 and no output until the read-only
  `SHOW VIEW` privilege was granted.
- Two live redacted PostgreSQL captures with the same key compared empty.
  Scans found no tested schema/object names, view SQL, connection host,
  password, or raw redaction key.

### Live browser, accessibility, privacy, and PWA

- `/opt/fleet/lib/verify-url.sh` returned HTTPS 200 in 711ms with no console
  errors, a useful title, `lang=en`, one h1, one main, image alt text, and named
  buttons.
- Independent axe checks at desktop and 390px found 0 serious/critical
  violations. Normal-size layouts had no horizontal overflow or undersized
  interactive targets.
- Keyboard traversal began at the skip link with a visible
  `rgb(141, 48, 37) solid 3px` outline, reached Compare in 12 desktop / 9 mobile
  tabs, and Enter produced the 4-change sample review. Malformed JSON was
  announced as an alert and Reset recovered.
- Identical browser snapshots showed `No drift detected`; mixed dialects gave
  an actionable alert and Reset restored the sample.
- Reduced motion matched and computed 0.00001s durations with automatic scroll.
- The complete normal demo flow requested only same-origin documents, scripts,
  CSS, and the desktop image. There were no analytics, third-party scripts, or
  remote fonts, and no console/page errors or failed responses.
- Real invalid-license verification allowed the product origin through CORS,
  returned `{valid:false, reason:"invalid"}`, stored only product-scoped local
  keys, and made one request across a reload.
- License API rate limiting is enforced. In a fresh burst, 30 requests
  succeeded and request 31 returned HTTP 429 with `Retry-After: 2` and
  `X-RateLimit-After: 2`. No allowance is documented on the product page.
- Service-worker update reached `activated`, removed an injected obsolete
  `sds-*` cache, and retained only `sds-651407dbcc68`. A separate offline
  context reloaded the root with its title, one h1, offline notice, and no
  errors.

### Performance, headers, caching, and deployment identity

- Lighthouse 13.0.1 mobile: Performance 100, Accessibility 100, Best Practices
  100, SEO 100; FCP 0.9s, LCP 0.9s, TBT 80ms, CLS 0, interactive 1.1s, and
  12 KiB transferred on the compact first load.
- Production static bytes: 7,888 bytes initial JavaScript, 13,904 bytes CSS,
  0 font bytes, and 53,184 bytes for the desktop-only hero WebP. All budgets
  pass.
- Root/legal HTML revalidates after 30 seconds. Hashed JS/CSS/image assets use
  one-year immutable caching. `sw.js` uses `no-cache`.
- Live responses include CSP, HSTS, Permissions-Policy, X-Frame-Options,
  Referrer-Policy, and X-Content-Type-Options.
- Root SHA-256 is
  `a07d54bfee569daf3e8c5d8661970ca190ec603ce6408a3c575ef524f761793a`.
  Worker SHA-256 is
  `305e388af11cd840ea4b1905c234dc23b03eb1450b15936cea8da35bddba5fb8`.
  Root, worker, legal pages, all hashed assets, the hero image, favicon,
  robots, and sitemap byte-match the clean candidate build. The tested live
  deployment is therefore the candidate.

## Retest requirements

1. Add `.factory/claims.json` and one observable demo-path test for every live
   or README claim; run every listed command first.
2. Ship `sds demo` (or `--demo`) with bundled realistic sample data and a temp
   output, add the real terminal recording, `/demo` sandbox/banner/reset/start
   behavior, and `.factory/demo.md`.
3. Rewrite the first screen to name developers and use “Try it with sample
   data” as the primary action with adjacent outcome copy.
4. Enable the billing product so the checkout link redirects normally.
5. Repair the 200% text overflow, add missing route/metadata/build identity,
   persist the invalid-license notice, and add `.factory/copy-audit.md`.
6. Rerun claims first, then the complete clean install/test/build/package,
   real-database, live-browser, accessibility, privacy, rate-limit, PWA,
   Lighthouse, and byte-identity matrix above.
