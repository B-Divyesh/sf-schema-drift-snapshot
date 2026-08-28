# Independent verification 2 — FAIL

Work order: `schema-drift-snapshot-verify-2`  
Candidate commit: `15a3f91e3839e46bd5278b87aa0987337a6dc1f1`  
Verified URL: <https://schema-drift-snapshot.sociobot.in/>  
Date: 2026-08-28 UTC

## Release decision

**FAIL — do not mark this candidate released until the live service worker can install and an offline reload succeeds.**

The candidate's CLI, static-site build, live content identity, normal browser workflow, accessibility, privacy posture, response policy, and performance checks pass. A fresh production PWA check fails: the worker's required precache list includes a file that the deployed host returns as 404. The worker consequently never activates or controls the page, so the promised offline shell is not available.

## Defect

### P1 — production service worker fails installation; offline reload fails

`dist/site/sw.js` precaches `/staticwebapp.config.json`. This file is present in the build directory, but the Azure deployment correctly treats it as deployment configuration rather than publicly served content:

```text
GET https://schema-drift-snapshot.sociobot.in/staticwebapp.config.json -> 404
```

The worker's `cache.addAll(SHELL)` therefore rejects during install. Fresh Chromium evidence after a normal production load and five seconds:

```json
{"controller":false,"regs":[],"caches":["sds-ab91cfb0ec6a"]}
```

An explicit `navigator.serviceWorker.register('/sw.js')` initially returns an `installing` worker, then the registration disappears. Switching the context offline and reloading returns `net::ERR_INTERNET_DISCONNECTED`; no cached page is shown. This also makes service-worker update behavior untestable in production.

Fix by excluding deployment-only configuration from the generated precache manifest (or serving every precached URL), deploy the corrected output, then verify an activated controller, version replacement, and offline root reload.

## Passing evidence

### Clean checkout and quality gates

- Began from a clean worktree at exactly `15a3f91e3839e46bd5278b87aa0987337a6dc1f1`.
- `npm ci` passed; audit reported 0 vulnerabilities.
- `npm test` passed: 14 Rust tests, five site contract tests, production site build, and Playwright (9 passed, 1 intentional desktop skip).
- `cargo fmt --all -- --check` and `cargo clippy --all-targets -- -D warnings` passed.
- The exact production command `npm run build` passed and emitted `dist/bin/sds` (6,399,072 bytes) and `dist/site`.
- `cargo package --allow-dirty` passed, packaging 45 files (285.6 KiB unpacked). A clean temporary consumer root installed the packaged crate with `cargo install --debug --path target/package/schema-drift-snapshot-0.1.0 --root <temp> --target-dir target`; installed `sds 0.1.0` produced the documented 5-change fixture review.

### CLI product workflow

- Release `sds compare --before expected.sds.json --after observed.sds.json --json` returned 5 classified changes: 4 high, 1 medium, 3 destructive, and 1 ORM-invisible.
- Markdown export completed; an explicit scan found no `DROP TABLE`, `ALTER TABLE`, `CREATE TABLE`, `DELETE FROM`, or `UPDATE` executable SQL. Identical snapshots render the `No drift detected` empty state.
- Invalid/recovery paths were exercised: missing redaction key on an unreachable supported PostgreSQL URL failed before connection with exit 2 and no file; unsupported `sqlite:` URL and missing snapshot each exited 2 with actionable errors; unlicensed Pro `check` exited 3.
- `sds --help` documents the read-only, no-row-data, no-repair-SQL boundary and stable non-interactive commands.
- No running PostgreSQL or MySQL service was available in this disposable environment, so real-database capture remains a coverage gap; fixture, decoder, URL-dispatch, redaction, reporting, error, and consumer-install paths were executed.

### Live identity, security, privacy, and performance

- Candidate `dist/site` SHA-256 values exactly match the live root, primary JS, both CSS/JS chunks, hero WebP, service worker, privacy page, and terms page. Root hash: `3bb00cf120bb05d75d66048d5d5fe9ed7402b7d042aa0e05d813f314e9bf8ce2`.
- Fresh live root/privacy/terms responses include CSP, `Permissions-Policy`, `X-Frame-Options: DENY`, HSTS, `Referrer-Policy`, and `X-Content-Type-Options`. The hashed JS has `Cache-Control: public, max-age=31536000, immutable`; `/sw.js` has `Cache-Control: no-cache`.
- Static audit found no analytics, telemetry, remote font, or third-party script. On normal production load, Chromium observed only `schema-drift-snapshot.sociobot.in` requests. The only product API is the documented Sociobot license verification endpoint and is reached only when a license is supplied.
- Live `/privacy/` and `/terms/` return 200. MIT license and README/CHANGELOG are present.
- Production bytes: initial JS 8,056 bytes total, CSS 13,863 bytes, fonts 0 bytes, hero WebP 53,184 bytes — below the stated 200/50/120/300 KiB budgets.
- Fresh mobile Lighthouse: Performance 100, Accessibility 100, Best Practices 100, SEO 100; LCP 1,080 ms, CLS 0, TBT 0 ms.

### Browser, accessibility, and responsive workflow

- Fresh live Chromium desktop and 390 x 844 mobile checks found title, `lang=en`, exactly one `h1`, exactly one `main`, zero console/page errors, and zero axe serious/critical violations.
- At 390px, page scroll width equalled client width (390px): no horizontal overflow. The browser-local comparison produced `4 differences found`.
- Keyboard-only desktop: first Tab focused `Skip to main content` with visible `rgb(141, 48, 37) solid 3px` outline; Enter on Compare produced four change entries. On mobile, keyboard invalid JSON produced the announced actionable message and Enter on Reset returned to the empty state.
- With `prefers-reduced-motion`, button transition duration computed to `1e-05s`.

## Retest commands

```sh
npm ci
npm test
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
npm run build
cargo package --allow-dirty
```

After deployment, additionally verify every URL in `sw.js` returns 200, then use a clean browser profile to confirm `navigator.serviceWorker.controller !== null`, a versioned active cache, previous-cache eviction after an update, and successful offline reload of `/`.
