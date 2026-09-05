# Independent verification 6 — PASS

Work order: `schema-drift-snapshot-verify-6`  
Implementation candidate: `bbcce1f3d6cff9c3c91924521ed214dfff608f86`  
Documentation baseline: `bbcce1f3d6cff9c3c91924521ed214dfff608f86`  
Live URL: <https://schema-drift-snapshot.sociobot.in/>  
Date: 2026-09-05 UTC

## Verdict

**PASS — zero findings; zero untested public claims.**

The live runtime byte-matches the reviewed implementation candidate. The product completes its stated job: a developer can use the local CLI to capture read-only PostgreSQL or MySQL catalog metadata, compare snapshots, receive a human-reviewable classification, and export Markdown or JSON without repair SQL. The browser sample is a separate, local demo of the same review task.

## First screen and demo

Fresh desktop and phone contexts were opened before scrolling.

- Job: “Explain database drift before repair.”
- Audience: developers reviewing PostgreSQL, MySQL, and ORM migrations before changing production.
- First action: “Try it with sample data,” visible at 705px on a 1366×900 desktop and 464px on a 390px-wide phone. It says that it loads two sample snapshots and a classified review.

The action opened `/demo/` with the route title `Demo — Schema Drift Snapshot`, a persistent “Demo — sample data, nothing is saved” label, Reset demo, and Start for real. It initially showed four classified changes. Invalid JSON showed “Expected snapshot is not valid JSON. Fix the highlighted snapshot and compare again”; Reset restored the four-change sample. Demo request recording contained no external origin and did not alter real-mode local storage.

## Claims

Every command declared in `.factory/claims.json` was run separately from this candidate. All 17 passed. Combined command output is retained in `/tmp/sds-verify-6-claims.log` in this verification environment.

| Claim | Result | Independent evidence |
| --- | --- | --- |
| `cli-demo` | PASS | Real CLI created an isolated bundled five-change review. |
| `sample-review` | PASS | Fresh Chromium demo had its label, four changes, and working reset. |
| `catalog-only-capture` | PASS | Declared regression passed; isolated PostgreSQL and MariaDB read-only regressions also passed. |
| `no-repair-sql` | PASS | Generated review contained explanation and no executable repair statements. |
| `review-formats` | PASS | Real compare returned JSON and wrote Markdown. |
| `deterministic-redaction` | PASS | Focused Rust deterministic redaction regression passed. |
| `browser-demo-local` | PASS | Edited snapshots stayed local; real-mode keys remained unchanged. |
| `offline-reload` | PASS | Dedicated context reloaded `/demo/` offline after service-worker activation. |
| `daily-license-check` | PASS | Invalid status persisted over reload with one verification request. |
| `no-analytics` | PASS | Complete sample flow made no tracking or external requests. |
| `cli-no-telemetry` | PASS | Shipped Rust manifest and sources passed the declared telemetry regression. |
| `free-compare-needs-no-license` | PASS | Bundled comparison completed without a license. |
| `price-copy` | PASS | Landing and Terms both state `$49` as one-time. |
| `database-url-support` | PASS | Focused URL-scheme regression passed for PostgreSQL and MySQL schemes. |
| `credential-hygiene` | PASS | Snapshot model/capture boundary regression passed. |
| `cli-exit-codes` | PASS | Documented 0, 1, 2, and 3 automation paths passed. |
| `pro-ci-policy` | PASS | Recorded valid-license policy-threshold regression passed. |

The landing page, README, privacy policy, and terms were cross-checked against this manifest. Their reliability claims map to the exercised claims above; there were no missing or untested public product claims.

## CLI and database evidence

- `npm ci`, `npm test`, `npm run lint`, `npm run build`, and `cargo package --allow-dirty` passed. The final `npm test` run reported 32 passing Playwright tests and two intended desktop-project skips.
- A clean consumer root installed the packaged crate from `target/package/schema-drift-snapshot-0.1.0`; its `sds 0.1.0 demo --json` returned 5 total changes: 4 high, 1 medium, 3 destructive, and 1 ORM-invisible.
- The release binary produced the same five-change JSON summary. Comparing a snapshot to itself printed `No drift detected`. An unsupported `sqlite:` URL returned exit 2 and created no output file.
- Isolated PostgreSQL 16.15 and MariaDB 10.11.14 instances were provisioned only for this verification. `cargo test --test postgres_read_only` and `cargo test --test mysql_read_only` both passed with disposable non-owner, read-only roles. They prove definition-only view drift is detected, row writes are rejected, and MariaDB fails closed until `SHOW VIEW` is granted. The temporary database processes were stopped after the checks.

## Live site, accessibility, privacy, and routes

- `/opt/fleet/lib/verify-url.sh` passed on `/` and `/demo/`: HTTPS 200, useful title, `lang=en`, one h1, main landmark, complete image alternatives, named buttons, and no console errors.
- Live Axe checks found zero serious or critical findings on desktop root, phone root, and 404. The local complete matrix also covered demo and both legal pages.
- Keyboard checks: first Tab focused the visible skip link with a `rgb(141, 48, 37) solid 3px` outline; Enter moved focus to main. Reduced motion reduced transitions to `1e-05s`.
- At 390px, normal and 200% text layouts were both exactly 390px wide with no page-level overflow. The primary sample action stayed visible.
- `/privacy/`, `/terms/`, `/demo/`, and `/404.html` returned 200; an unknown path returned the designed not-found page with HTTP 404. The 404 has its own title, h1, return path, and zero serious/critical Axe issues. All 12 discovered same-origin links returned 200.
- A dedicated live service-worker context received a controller, then reloaded `/demo/` offline with the four-change review intact. Its only cache was the current versioned `sds-6ea2a30be53d` cache.
- A normal demo flow contacted only this product origin. A real invalid-license check stripped the query token, showed the invalid notice after reload, and made one Sociobot verification request across the reload. Checkout is honestly marked closed; no broken purchase link remains.
- Live responses have CSP with `frame-ancestors 'none'`, HSTS, Permissions-Policy, X-Frame-Options, Referrer-Policy, and X-Content-Type-Options. `sw.js` is `no-cache`.
- Mobile Lighthouse 13.4.1: Performance 100, Accessibility 100, Best Practices 100, SEO 100; FCP/LCP 1,106ms, TBT 0ms, CLS 0.

## Deployment identity and prior findings

All 17 public build files (HTML, worker, assets, icons, robots, and sitemap) byte-matched the live custom domain. `HEAD` and `origin/main` were both the candidate SHA and the worktree was clean before this documentation update.

Earlier findings are resolved:

- Verification 1 header and immutable-cache concerns: production policy headers are present and worker caching is no-cache.
- Verification 2 worker-install/offline defect: a fresh controlled context reloaded the real demo offline.
- Verifications 3 and 4 view-definition/read-only-role defect: both real dialect regressions passed; the PostgreSQL non-owner path detected the view change. The footer 44px test also passed on desktop and phone.
- Verification 5 missing claims/demo, first-screen, mobile-resize, route, metadata, invalid-license, and copy-audit findings: each is present and exercised above. The earlier unavailable checkout is deliberately closed, not linked; it is not a broken user path.

One initial full-suite browser process ended with a Chromium SIGSEGV while its mobile context was closing. The only interrupted assertion was rerun in the complete `npm run test:e2e` matrix (32 passed, 2 intended skips), then the exact `npm test` command passed. Independent live Chromium checks also passed. This was not reproducible as a product failure.

## Scope note

This is a CLI plus static site, with no product backend or tenant state to exercise. The required artifact-consumer check and isolated real-database checks were used instead. Billing registration remains external and unavailable; the site does not claim checkout works or expose its former 404 endpoint.

