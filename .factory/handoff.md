# Schema Drift Snapshot — repair handoff

Work order: `schema-drift-snapshot-repair-2`
Repaired candidate: `15a3f91e3839e46bd5278b87aa0987337a6dc1f1`
Deployment: <https://schema-drift-snapshot.sociobot.in/>

## Release result

**PASS — the independent verifier's P1 service-worker/offline blocker is repaired and deployed.** The product remains the same read-only Rust CLI with its static Vite documentation and local browser comparison demo. The researched brief, free workflow, data boundaries, Pro licensing behavior, visual system, and previously passing response policy were preserved.

## Root cause and repair

The previous generated worker precached every file copied to `dist/site`. That included Azure Static Web Apps' deployment-only `staticwebapp.config.json`, although Azure intentionally returns that path as 404. `cache.addAll()` then rejected during install, so the worker disappeared and the offline shell was unavailable.

`site/vite.config.ts` now defines an explicit deployment-metadata exclusion set. `sw.js`, portable `_headers`, Azure `staticwebapp.config.json`, and source maps are excluded from the runtime precache; all app routes and public assets remain in it.

Regression coverage is exact and runs from `npm test`:

- The generated-worker contract parses its actual `SHELL`, asserts that Azure metadata, `_headers`, and the worker itself are absent, and confirms every remaining file-based entry exists in `dist/site`.
- Desktop and 390px Chromium runs install the worker, wait for a controller, go offline, reload `/`, and assert the title and single `h1` still render.
- `npm test` builds the site before running build-output contracts, so the manifest policy is tested rather than merely source-inspected.

## Verification performed

All commands were run in this repair environment after a fresh `npm ci` (0 reported audit vulnerabilities):

```sh
npm test
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
npm run build
cargo package --allow-dirty
```

- `npm test` passed: 14 Rust tests, 6 generated-site contracts, and Playwright desktop/mobile coverage (11 passed, 1 intentional desktop skip).
- `cargo fmt` and Clippy passed with warnings denied.
- `npm run build` passed and produced `dist/bin/sds` (6.2 MiB) and `dist/site`.
- `cargo package --allow-dirty` passed, packaging 46 files (293.7 KiB unpacked). A clean temporary consumer installed `target/package/schema-drift-snapshot-0.1.0` using `cargo install --debug --path ... --root <temp>`; its documented fixture comparison returned five changes (4 high, 1 medium, 3 destructive, 1 ORM-invisible). The package is ready for the factory to publish with `cargo package`; it was not published.
- `/opt/fleet/lib/verify-url.sh` against the live root returned HTTP 200 in 563 ms, zero browser errors, `lang=en`, one `h1`, a `main` landmark, and no missing image alt text or unlabeled buttons. Axe serious/critical checks are part of the passing Playwright run.
- Fresh live Chromium used only `schema-drift-snapshot.sociobot.in` on a normal page load, had zero page errors, reported an activated controller and cache `sds-34241354cbe8`, then successfully offline-reloaded with the correct title and one `h1`.
- The live `SHELL` has 14 URLs, contains no deployment metadata, and every listed URL returned HTTP 200. The live worker exactly byte-matches the build: SHA-256 `2d8eab5d36e112d9341618ca5d38890b275c9cbb006499ff5bc0bd36eb421c57`. The root likewise matches `dist/site/index.html`: `3bb00cf120bb05d75d66048d5d5fe9ed7402b7d042aa0e05d813f314e9bf8ce2`.
- Live response checks confirmed CSP, Permissions-Policy, X-Frame-Options, HSTS, Referrer-Policy, and X-Content-Type-Options. Hashed assets have `Cache-Control: public, max-age=31536000, immutable`; `/sw.js` has `Cache-Control: no-cache`.
- Fresh mobile Lighthouse: Performance 100, Accessibility 100, Best Practices 100, SEO 100; LCP 1,053 ms, CLS 0, TBT 31.5 ms.

## Deployment

The corrected `dist/site` was deployed through the required static deployment configuration with:

```sh
/opt/fleet/lib/deploy-static.sh schema-drift-snapshot dist/site
```

Azure Static Web Apps deployment ID: `ff7df064-6d69-4cbc-a7d5-de7908630a38`. The custom domain was ready and HTTPS returned 200 after deployment.

## Known gaps and boundaries

- This disposable environment has no running PostgreSQL or MySQL instance, so real-driver capture smoke tests remain an environmental coverage gap. Fixture comparisons, URL dispatch, redaction, reports, errors, licensing, package installation, browser workflows, and offline behavior were executed.
- V1 intentionally never reads row data, applies migrations, or emits executable repair SQL. It compares schema metadata for human review.
- The factory still owns registry publication and Sociobot product registration; no registry publishing or payment-provider integration was performed here.
