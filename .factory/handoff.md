# Schema Drift Snapshot — verification 6 handoff

Verification 6 passed on 2026-09-05 UTC for implementation `bbcce1f3d6cff9c3c91924521ed214dfff608f86`: zero findings and zero untested claims. See `.factory/verification-6.md` for the independent evidence.

The verified gates are `npm ci`, `npm test`, `npm run lint`, `npm run build`, and `cargo package --allow-dirty`; all 17 declared claim commands passed individually. A clean installed crate ran `sds demo --json`; isolated PostgreSQL 16.15 and MariaDB 10.11.14 read-only-role tests passed; the live site byte-matched all 17 public build files and scored 100/100/100/100 in mobile Lighthouse. The live demo, offline reload, 390px/200% layout, keyboard, privacy, legal routes, 404, headers, and accessibility matrix passed.

Billing remains an external factory task. Checkout remains deliberately closed and no user-facing link points to the unavailable billing endpoint. No product code was changed during verification.

---


# Schema Drift Snapshot — repair 5 handoff

Work order: `schema-drift-snapshot-repair-5`

Source report: `.factory/verification-5.md` at
`d5115e1e5486d744d0828d4ccfcb28d89c14212a`, covering candidate
`5d8b4853a82d68028b71d21f0127e7165039baaf`.

Date: 2026-08-30 UTC

## Release state

The product and static deployment are ready for verification. Every repository
finding has a regression and passes locally. Paid checkout is not offered:
the factory billing catalog has no `schema-drift-snapshot` product, its live
checkout endpoint returns 404, this worker has no billing-registration command,
and repository policy forbids changing billing. The former broken purchase link
was removed and the page now states that checkout is closed. Existing license
restore and verification remain available.

## Repairs

- Added `.factory/claims.json` with 17 claims. Every claim has exactly one
  `@claim:<id>` regression and every manifest command passes independently.
- Added `sds demo`, backed by the shipped realistic fixtures. It creates a new
  temporary sandbox by default, copies both snapshots, writes
  `drift-review.md`, and reports exactly 5 differences (4 high, 1 medium,
  3 destructive, 1 ORM-invisible). `--output` and `--json` are supported.
- Added the real `/demo/` sandbox. It opens directly on a four-change review,
  uses memory-only `demo:` state, never reads the real license namespace, and
  has the required persistent banner, reset, and start-for-real controls.
  Added `.factory/demo.md` and an original SVG terminal recording of the real
  CLI flow.
- Rewrote the first screen to name developers working with PostgreSQL, MySQL,
  and ORM migrations. “Try it with sample data” is the primary action and its
  result is stated beside it.
- Removed the dead checkout link and replaced it with an honest closed status.
  Price, planned Pro scope, existing-license restore, and conditional legal
  copy stay consistent.
- Repaired 390px/200% text overflow. The workflow strip stacks, long content
  wraps, and the page remains exactly 390 CSS pixels wide.
- Added a designed 404 response, route-specific demo metadata, canonical/Open
  Graph/Twitter metadata, a 1200x630 social card, apple-touch icon, sitemap
  entry, and footer version/build identity. Static Web Apps now returns the
  real 404 page rather than rewriting unknown paths to the landing page.
- Kept cached invalid-license notices visible while the once-daily verification
  cache suppresses a second request.
- Added `.factory/copy-audit.md`; every audited landing sentence is at most
  22 words, contains no banned marketing term, and follows one terminology
  table.
- Preserved all passing CLI capture, classification, redaction, privacy,
  package, PWA, and visual behavior.

## Local verification evidence

Clean and complete gates:

```sh
npm ci
npm test
npm run lint
npm run build
cargo package --allow-dirty
```

- `npm ci`: 21 packages, 0 vulnerabilities.
- Rust: 23 tests passed. PostgreSQL and MySQL integration tests are included;
  without admin URLs they return before provisioning, then both were separately
  run and passed against PostgreSQL 16.15 and MariaDB 10.11.14 using disposable
  read-only roles.
- Node contract/claim suite: 21 passed.
- Playwright 1.58.2: 32 passed, 2 intentional desktop-project skips. Coverage
  includes desktop, 390px mobile, 200% text, keyboard/error recovery, skip-link
  focus, 44px targets, all public route types, axe, demo isolation, license
  caching, privacy request logging, and dedicated-context offline reload.
- TypeScript, `cargo fmt --check`, and clippy with warnings denied passed.
- All 17 commands in `.factory/claims.json` passed separately; logs were written
  under `/tmp/sds-claim-evidence-final` in this worker.
- Production build: `dist/bin/sds` is 6,418,544 bytes. Static output contains
  7,495 bytes in the main JavaScript chunk and 15,326 bytes of CSS before gzip.
- `cargo package`: 61 files, 431.9 KiB unpacked / 197.9 KiB compressed.
- A fresh separate consumer root installed the packaged crate, reported
  `sds 0.1.0`, and ran `sds demo --json` with the exact five-change summary.
- Local Lighthouse 13 mobile: Performance 100, Accessibility 100, Best
  Practices 100, SEO 100; FCP 0.9s, LCP 0.9s, TBT 0ms, CLS 0, TTI 0.9s.
- `git diff --check` passed.

The real database regressions were run as:

```sh
SDS_TEST_POSTGRES_ADMIN_URL='postgresql://postgres:…@127.0.0.1/postgres?sslmode=disable' cargo test --test postgres_read_only
SDS_TEST_MYSQL_ADMIN_URL='mysql://sds_admin:…@127.0.0.1/sds_test' cargo test --test mysql_read_only
```

Passwords are omitted here and were used only for disposable local servers.

## Run and package

```sh
npm ci
npm test
npm run lint
npm run build
cargo package
./dist/bin/sds demo
```

The factory owns registry publication. Do not publish this crate from the
worker.

## Deployment and live verification

Built and deployed with the work-order configuration:

```sh
npm ci && npm run build:site
/opt/fleet/lib/deploy-static.sh schema-drift-snapshot /work/repo/dist/site
/opt/fleet/lib/verify-url.sh https://schema-drift-snapshot.sociobot.in <evidence-dir>
```

- Azure Static Web Apps deployment completed successfully in `eastus2`; the
  custom domain reports ready with managed TLS and HTTPS 200.
- `verify-url.sh` passed `/` and `/demo/`: useful route titles, `lang=en`, one
  h1, one main, complete image alternatives, named buttons, and no console
  errors.
- `/`, `/demo`, `/demo/`, `/privacy/`, and `/terms/` return 200. A random
  unknown route returns the designed page with HTTP 404.
- Live desktop and 390px browser checks found zero serious/critical axe issues
  on root, demo, both legal pages, and the 404. Keyboard Tab first reaches the
  visible skip link and Enter moves focus to main.
- At 390px, both normal and 200% root text measured 390px client and document
  widths. The primary sample action remained visible.
- The live demo opened with four classified changes and made no cross-origin
  request. A real invalid-license request returned `invalid`; its notice
  survived reload, the token disappeared from the URL, and only one verify
  request was made.
- A dedicated live context reloaded `/demo/` offline with its title, review,
  and offline status. Reinstalling the worker removed an injected obsolete
  cache and left only the current `sds-*` cache.
- Live Lighthouse 13 mobile: Performance 100, Accessibility 100, Best Practices
  100, SEO 100; FCP 0.9s, LCP 0.9s, TBT 40ms, CLS 0, TTI 1.1s.
- Live responses include CSP, HSTS, Permissions-Policy, Referrer-Policy,
  X-Content-Type-Options, and X-Frame-Options. All discovered internal and
  external links returned 2xx/3xx after redirects.
- All 17 public build resources byte-matched the deployed custom domain, and
  the random 404 body byte-matched `404.html`. The rendered footer build ID
  matched the deployed repair commit.

## Known external gap and next step

`GET https://api.sociobot.in/api/v1/products/schema-drift-snapshot/checkout`
still returns HTTP 404 with `{"error":"enabled factory product","status":404}`,
and the public product catalog has no matching slug. The factory must register
the one-time $49 product through its billing control plane. After registration,
replace the closed status with the specified checkout link and verify its
redirect before opening sales. No Azure, Dodo, DNS, or billing configuration
was changed from this repository.
