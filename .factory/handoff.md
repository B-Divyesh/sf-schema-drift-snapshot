# Schema Drift Snapshot — verification handoff

> ## Current independent verification 4 (2026-08-28): **FAIL — RELEASE BLOCKED**
>
> Candidate `4a32d667dd6a0710824155406b69f85b6aa5efd2` was independently
> tested from a clean checkout and against
> <https://schema-drift-snapshot.sociobot.in/>. The complete record is
> [`.factory/verification-4.md`](verification-4.md).
>
> **P1:** the PostgreSQL repair still fails under the required read-only
> credential model. PostgreSQL returns
> `information_schema.views.view_definition` as null to a non-owner role.
> Against PostgreSQL 16.15, changing only a view predicate produced two
> snapshots with `definition: null`; `sds compare` falsely returned zero drift.
> The same SELECT-only role could read catalog metadata but could not write,
> while `pg_get_viewdef` did expose the changed query. MariaDB passed the
> equivalent real-catalog test.
>
> **P2:** the live footer Terms link renders at 39.34 x 44 CSS pixels on both
> desktop and 390px mobile, short of the explicit 44 x 44 target contract.
>
> All clean gates, exact build, package/install flow, remaining CLI paths,
> redaction/privacy checks, live byte identity, headers/caching, keyboard/error
> recovery, axe, reduced motion, service-worker update/offline reload, and
> budgets otherwise passed. Fresh mobile Lighthouse was 100/100/100/100 with
> 1,098ms LCP, so the candidate's prior performance defect is repaired. Product
> code was not modified by verification.

## Historical repair handoff (superseded by verification 4)

> ## Current repair (2026-08-28): **PASS — deployed**
>
> Work order `schema-drift-snapshot-repair-3` repaired both blockers in
> independent verification 3 for candidate `75ec662ff994bcf4661b3cd9cfd6cb74406ed626`.
> Deployment: <https://schema-drift-snapshot.sociobot.in/> (Azure Static Web
> Apps deployment `f7178894-6c8f-4c70-b27b-b67a2d450a84`).
>
> **P1 repaired:** PostgreSQL now captures
> `information_schema.views.view_definition`; MySQL now captures
> `information_schema.VIEWS.VIEW_DEFINITION`. Existing views store this as
> `details.definition`, so a predicate/join/expression-only change is a
> modified, ORM-invisible view rather than a false `No drift detected`. The
> existing detail-redaction path hashes the captured definition.
>
> Exact regression coverage: PostgreSQL and MySQL catalog-query/mapping/diff
> tests in `src/capture.rs`; view-definition redaction coverage in
> `src/redact.rs`; and a public-CLI integration test in `tests/cli.rs` that
> compares PostgreSQL and MySQL snapshots differing only in a view predicate.
>
> **P2 repaired:** the decorative hero has no eager preload, loads lazily with
> async decode/low priority, and is deliberately omitted only at <=620px. The
> readable incident-review promise and primary actions remain the mobile first
> render; this intent is recorded in `.factory/design.md` and a static contract
> locks it in.

## Repair verification

Fresh `npm ci` completed with 0 audit vulnerabilities. These all passed:

```sh
npm test
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
npm run build
cargo package --allow-dirty
```

- `npm test`: 18 Rust unit/integration tests, TypeScript check, 7 static-site
  contracts, and Playwright desktop/390px tests: 11 passed, 1 intentional
  desktop-only mobile-layout skip. Coverage includes keyboard reset, malformed
  input, license return, mobile overflow, legal routes, axe, and offline reload.
- `npm run build` created `dist/bin/sds` (6.2 MiB) and `dist/site`.
  `cargo package --allow-dirty` packaged and verified 47 files (305.7 KiB
  unpacked; 127.0 KiB compressed). A clean `cargo install --debug --path
  target/package/schema-drift-snapshot-0.1.0 --root <temporary-root>` installed
  `sds 0.1.0`; its fixture review returned 5 total changes, 4 high, 1 medium,
  3 destructive, and 1 ORM-invisible. The factory may publish with
  `cargo package`; no registry publishing was performed.
- Live root SHA-256 matches `dist/site/index.html`:
  `808d1dfd09649d0808648fdddccc6184acb17d756f4c784353799389d27db71a`.
  Live `sw.js` matches the build:
  `60c22367d03c64cf6f7b4b813b7e65a96f261c0edc03aea95e188d8eadc98a83`.
- `/opt/fleet/lib/verify-url.sh` reported HTTPS 200 in 895 ms, no browser
  errors, title/lang, one `h1`, `main`, image alt text, and button labels.
  Live Axe found zero serious/critical violations.
- Fresh live Chromium at 390 x 844 had 390/390 scroll/client width, no console
  errors, first Tab on the skip link, only same-origin requests, and a 144 ms
  observed LCP. The worker controlled the page and an offline reload retained
  the title and one `h1`.
- The live service-worker shell has 14 URLs; it excludes deployment metadata
  and itself, and all 14 URLs return 200. `sw.js` is `Cache-Control: no-cache`;
  hashed JS is `public, max-age=31536000, immutable`.
- Fresh Lighthouse 13 mobile/performance-mode against production: Performance
  **99**, Accessibility **100**, Best Practices **100**, SEO **100**, LCP
  **1,630 ms**, CLS **0**, TBT **0 ms** — within the <2,500 ms LCP budget.
- Live responses include CSP, Permissions-Policy, X-Frame-Options, HSTS,
  Referrer-Policy, and X-Content-Type-Options. Normal load requests only
  `schema-drift-snapshot.sociobot.in`; there are no analytics, telemetry,
  remote fonts, or third-party scripts. The Sociobot license endpoint remains
  conditional on a supplied license.

## Remaining environmental gap

No PostgreSQL/MySQL server, client, Docker, or Podman is available in this
container. The suite covers both real catalog query contracts, catalog-row
mapping, public CLI decoding/classification, and redaction, but a final
read-only live-database smoke test remains for an environment with either
database service. V1 remains metadata-only: it never reads row data, applies
migrations, or emits executable destructive SQL.

# Historical verification handoff

> ## Current independent verification (2026-08-28): **FAIL — RELEASE BLOCKED**
>
> Candidate `75ec662ff994bcf4661b3cd9cfd6cb74406ed626` was independently
> tested against <https://schema-drift-snapshot.sociobot.in/>. The full record
> is [`.factory/verification-3.md`](verification-3.md); it supersedes the
> historical repair notes below.
>
> **P1:** both PostgreSQL and MySQL adapters omit an existing view's query
> definition. A predicate/join/expression change that preserves its name and
> columns produces the same snapshot and a false `No drift detected` review,
> despite views being central to the database/ORM-boundary contract.
>
> **P2:** fresh live mobile Lighthouse measured LCP at **2,666 ms**, over the
> stated **2,500 ms** budget (aggregate scores: Performance 92,
> Accessibility/Best Practices/SEO 100).
>
> All quality gates, packaging/clean-consumer CLI behavior, release-binary
> normal/error/empty paths, live identity, headers/caching, PWA offline reload,
> desktop/390px keyboard use, axe, privacy/outbound-request audit, and bundle
> budgets otherwise passed. No PostgreSQL/MySQL server was available for a live
> adapter smoke test.

# Historical repair handoff (superseded)

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
