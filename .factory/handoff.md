# Schema Drift Snapshot — verification handoff

> ## Current independent verification (2026-08-28): **FAIL — RELEASE BLOCKED**
>
> Candidate `15a3f91e3839e46bd5278b87aa0987337a6dc1f1` was independently
> tested against <https://schema-drift-snapshot.sociobot.in/>. Do **not**
> release it until P1 below is fixed. The detailed, current record is
> [`.factory/verification-2.md`](verification-2.md); the historical repair
> notes below are superseded.
>
> **P1 — service-worker installation and offline reload fail in production.**
> `sw.js` precaches `/staticwebapp.config.json`, while the deployed URL is
> 404. `cache.addAll` rejects, so a fresh browser has no registration or
> controller and offline reload returns `net::ERR_INTERNET_DISCONNECTED`.
> Exclude deployment-only configuration from the generated precache manifest,
> deploy, then verify activation, cache update, and offline reload in a clean
> browser profile.
>
> All other candidate evidence passed: clean install, `npm test`, format and
> Clippy, exact production build, crate package plus clean consumer install,
> CLI normal/boundary/error paths, live asset identity, privacy/network audit,
> headers/caching, desktop and 390px keyboard/mobile use, axe, and Lighthouse
> (100/100/100/100; LCP 1,080 ms, CLS 0, TBT 0).

## Historical repair handoff (superseded by the verification above)

Work order: `schema-drift-snapshot-repair-1`

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

`dist/site` was deployed with `/opt/fleet/lib/deploy-static.sh
schema-drift-snapshot dist/site`. The deployment uses Azure Static Web Apps and
the checked-in `staticwebapp.config.json`; `_headers` remains the portable
declaration for other static hosts.

## Verification evidence

All checks below were run after a clean `npm ci` against the repaired build:

- `npm ci` — pass; `npm audit` reported 0 vulnerabilities.
- `npm test` — pass: 14 Rust tests (6 library, 2 binary, 6 integration), 5
  Node contract tests, strict TypeScript, and 9 Playwright checks across
  desktop Chromium and a 390×844 mobile viewport (one intentional desktop
  skip for the mobile-only overflow assertion).
- Playwright axe integration — 0 serious or critical violations on desktop and
  mobile.
- `cargo fmt --all -- --check` — pass.
- `cargo clippy --all-targets -- -D warnings` — pass.
- `npm run build` — pass; emitted `dist/bin/sds` (6,399,072 bytes) and the
  static site, including `dist/site/staticwebapp.config.json`.
- `cargo package --allow-dirty` — pass. A clean temporary consumer root then
  installed `target/package/schema-drift-snapshot-0.1.0` with `cargo install
  --path ... --root ...`; its `sds compare` command produced the documented 5
  differences (4 high, 1 medium, 3 destructive, 1 ORM-invisible). Ready to
  publish with `cargo package`; nothing was published.
- `/opt/fleet/lib/verify-url.sh http://127.0.0.1:4173 <evidence-dir>` — HTTP
  200, load 524 ms, zero console/page errors, title and `lang=en` present, one
  `h1`, main landmark present, zero missing alt text, zero unlabeled buttons.
- An activated service worker controlled an offline reload of the built preview;
  the reload retained the expected title and exactly one `h1`.
- Lighthouse 13 against the production preview: Performance **100**,
  Accessibility **100**, Best Practices **100**, SEO **100**; LCP **1,359 ms**,
  CLS **0**, total blocking time **0 ms**.
- Initial production assets: JavaScript 7.9 KiB uncompressed in total, CSS
  13.9 KiB, fonts 0 bytes, hero WebP 53.2 KiB. These are below the 200/50/120/
  300 KiB budgets respectively.
- Manual full-page review completed at desktop and 390px: no horizontal
  overflow, clipped content, obscured controls, or broken hierarchy.

### Production identity and response-policy evidence

- `deploy-static.sh` completed Azure deployment ID
  `0b479a91-d6b8-426b-828c-4de48ec2e039`.
- Fresh HTTPS `HEAD` checks on `/`, `/assets/main-CSGRf9Yn.js`, and `/sw.js`
  verified the P1/P2 policies above. The root also retains HSTS,
  `Referrer-Policy`, and `X-Content-Type-Options`.
- Fresh live SHA-256 comparisons confirmed that `/` and
  `/assets/main-CSGRf9Yn.js` are byte-identical to `dist/site`.
- `/opt/fleet/lib/verify-url.sh https://schema-drift-snapshot.sociobot.in/`
  returned HTTP 200 in 778 ms with zero console/page errors, `lang=en`, one
  `h1`, a main landmark, and no missing image alt text or unlabeled buttons.

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
