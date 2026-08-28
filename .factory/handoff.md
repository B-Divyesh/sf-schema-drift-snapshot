# Schema Drift Snapshot — build handoff

> ## Independent verifier decision (2026-08-28): **FAIL**
>
> Candidate `7df275e8fedc88e39cccbe95362ac280d640a261` functionally builds,
> tests, packages, and hash-matches the live content at
> <https://schema-drift-snapshot.sociobot.in/>. Do not release the current
> deployment: it omits the CSP, `Permissions-Policy`, and `X-Frame-Options`
> declared in the shipped `_headers`, and serves hashed assets and `sw.js` with
> only `public, must-revalidate, max-age=30` rather than the shipped immutable/
> no-cache policies. See `.factory/verification.md` for exact fresh evidence,
> P1/P2/P3 defects, full test results, and retest commands.

Work order: `schema-drift-snapshot-build-1`

Version: `0.1.0`

Completed: 2026-08-27

## What shipped

- A single Rust binary, `sds`, with PostgreSQL and MySQL catalog adapters.
  Connections are put in read-only mode and only table/view, column, index,
  and foreign-key metadata is queried. Database URLs are not persisted.
- `snapshot`, `compare`, and Pro `check` commands with helpful `--help`, stable
  exit codes, non-interactive CI behavior, JSON output, versioned portable
  snapshots, deterministic name/definition redaction, and atomic output files.
- A complete classifier for added, removed, and modified objects. Every change
  receives a risk level, destructive flag, ORM-visibility flag, likely owner,
  and explanation. Markdown reports contain a repair checklist but never
  executable repair SQL.
- A one-time $49 Pro CI policy gate. The free workflow retains live capture,
  redaction, all classification and ownership guidance, Markdown, and JSON.
  CLI license checks use the Sociobot endpoint, cache only a token hash and
  verdict for one day, and tolerate offline use of a prior valid verdict.
- A Vite/vanilla TypeScript landing and documentation site at the requested
  `dist/site` root, including a real browser-local snapshot comparison, empty,
  error, and offline states, install guidance, pricing, license restore,
  `/privacy/`, and `/terms/`.
- The paid-unlock return flow stores `sb_license:schema-drift-snapshot`, removes
  the token from the URL, verifies in the background at most daily, reconciles
  invalid licenses quietly, and never blocks free features.
- A product-specific paper-cut diorama system and original generated hero. The
  optimized WebP is 53,184 bytes; prompt and provenance are in
  `.factory/design.md`.
- A versioned service worker offline shell, immutable asset cache headers,
  security headers, `robots.txt`, sitemap, CI workflow, MIT license, changelog,
  sample fixtures, and public usage documentation.

## Build and deploy

```sh
npm ci
npm test
npm run build
```

The exact build command is `npm run build`. It creates:

- `dist/bin/sds` — stripped Linux release binary (6.2 MB in this build)
- `dist/site/index.html` — static deployment root

The factory should deploy `dist/site`. It owns release archives, registry
credentials, billing product registration, DNS, and deployment. No publish,
billing-registration, or infrastructure action was performed by this worker.

## Verification evidence

All checks were run locally against the final production build:

- `npm test` — pass: 13 Rust tests (6 library, 2 binary, 5 integration), 4
  Node contract tests, strict TypeScript, and 9 Playwright checks across
  desktop Chromium and a 390×844 mobile viewport (one intentional
  desktop-project skip for the mobile-only overflow assertion).
- Playwright axe integration — 0 serious or critical violations on desktop and
  mobile.
- `cargo fmt --all -- --check` — pass.
- `cargo clippy --all-targets -- -D warnings` — pass.
- `npm audit` — 0 known vulnerabilities.
- `npm run build` — pass; release CLI and static site emitted to `dist/`.
- `cargo package --allow-dirty` — pass; 43 files, 116.0 KiB compressed, package
  verification compile successful. Ready-to-publish dry run only; nothing was
  published.
- `/opt/fleet/lib/verify-url.sh http://127.0.0.1:4173 <evidence-dir>` — HTTP
  200, load 534 ms, zero console/page errors, title and `lang=en` present, one
  `h1`, main landmark present, zero missing alt text, zero unlabeled buttons.
- Lighthouse 13 mobile defaults against the production preview: Performance
  **100**, Accessibility **100**, Best Practices **100**, SEO **100**. LCP
  **1,362 ms**, CLS **0**, total blocking time **0 ms**.
- Initial production assets: JavaScript 7.9 KiB uncompressed in total, CSS
  13.9 KiB, fonts 0 bytes, hero WebP 53.2 KiB. These are below the 200/50/120/
  300 KiB budgets respectively.
- Manual full-page review completed at desktop and 390px: no horizontal
  overflow, clipped content, obscured controls, or broken hierarchy.

## Known gaps and deliberate boundaries

- Live catalog adapters compile and use real driver APIs, but this disposable
  build environment did not provide running PostgreSQL/MySQL services. Fixture
  comparisons, URL dispatch, redaction, reporting, CLI exits, and mocked live
  license verification are covered. A release smoke test should capture one
  disposable database of each supported dialect.
- V1 intentionally excludes row data, permissions, triggers, procedures,
  partition internals, migration execution, and generated repair SQL. These
  boundaries keep the artifact read-only and review-oriented.
- PostgreSQL TLS uses the platform's native certificate store. Environments
  with private CAs must configure trust through their standard system/driver
  settings.
- The billing slug is used in the documented Sociobot URL; no product ID is
  hardcoded. The factory still needs to register the live product and attach
  its return URL before checkout can complete in production.

## Recommended next steps

1. Run disposable PostgreSQL and MySQL capture smoke tests in release CI.
2. Register the one-time product in Sociobot and exercise a hosted checkout,
   return, restore, refund/revocation, and offline-verdict cycle.
3. Produce signed binary archives for Linux, macOS, and Windows from the
   factory release workflow and link them from the install section.
