# Review 1 — Explain database drift before repair

Work order: `schema-drift-snapshot-review-1`  
Implementation candidate: `bbcce1f3d6cff9c3c91924521ed214dfff608f86`  
Documentation baseline: `267c5b8c9c5fbdb6e24b3111c71a3c351318ee2a`  
Live URL: <https://schema-drift-snapshot.sociobot.in/>  
Date: 2026-09-05 UTC

## Verdict

**PASS — zero findings of every severity and zero untested public claims.**

The documentation baseline differs from the implementation only in
`.factory/verification-6.md` and `.factory/handoff.md`. All 17 publicly served
build files byte-match a clean production build of the implementation
candidate. No product code was changed during this review.

## First screen and sample

Fresh 1366×900 desktop and 390×844 phone contexts were opened at scroll
position zero.

- Job: **Explain database drift before repair.**
- Audience: developers reviewing PostgreSQL, MySQL, and ORM migrations before
  anyone changes production.
- First action: **Try it with sample data.** It was visible at y=705 on desktop
  and y=464 on phone, with the adjacent result: “Loads two sample snapshots and
  a classified review.”

The action opened `/demo/` with its own title, four realistic classified
changes, and the persistent “Demo — sample data, nothing is saved” label.
Malformed JSON produced an announced, actionable error. Reset restored all four
changes. “Start for real” returned to the install section. A sentinel placed in
the real license namespace was unchanged by demo editing and reset; before
leaving demo mode, every request remained on the product origin.

## Declared claims

Every command in `.factory/claims.json` was run separately from a fresh
detached checkout after `npm ci`. All 17 passed.

| Claim | Declared command | Result |
| --- | --- | --- |
| `cli-demo` | `node --test --test-name-pattern='@claim:cli-demo' site/tests/claims.test.mjs` | PASS — isolated five-change review written |
| `sample-review` | `npm run test:claims -- --grep '@claim:sample-review'` | PASS — label, four changes, reset |
| `catalog-only-capture` | `node --test --test-name-pattern='@claim:catalog-only-capture' site/tests/claims.test.mjs` | PASS — catalog-only session contract |
| `no-repair-sql` | `node --test --test-name-pattern='@claim:no-repair-sql' site/tests/claims.test.mjs` | PASS — no executable repair statement |
| `review-formats` | `node --test --test-name-pattern='@claim:review-formats' site/tests/claims.test.mjs` | PASS — JSON plus Markdown |
| `deterministic-redaction` | `node --test --test-name-pattern='@claim:deterministic-redaction' site/tests/claims.test.mjs` | PASS — stable hashes, raw names absent |
| `browser-demo-local` | `npm run test:claims -- --grep '@claim:browser-demo-local'` | PASS — local comparison and isolated storage |
| `offline-reload` | `npm run test:claims -- --grep '@claim:offline-reload'` | PASS — dedicated offline context |
| `daily-license-check` | `npm run test:claims -- --grep '@claim:daily-license-check'` | PASS — one request and persistent invalid notice |
| `no-analytics` | `npm run test:claims -- --grep '@claim:no-analytics'` | PASS — no tracking request or script |
| `cli-no-telemetry` | `node --test --test-name-pattern='@claim:cli-no-telemetry' site/tests/claims.test.mjs` | PASS — shipped manifest and sources |
| `free-compare-needs-no-license` | `node --test --test-name-pattern='@claim:free-compare-needs-no-license' site/tests/claims.test.mjs` | PASS — free comparison completed |
| `price-copy` | `npm run test:claims -- --grep '@claim:price-copy'` | PASS — $49 one-time copy agrees |
| `database-url-support` | `node --test --test-name-pattern='@claim:database-url-support' site/tests/claims.test.mjs` | PASS — all three schemes and rejection path |
| `credential-hygiene` | `node --test --test-name-pattern='@claim:credential-hygiene' site/tests/claims.test.mjs` | PASS — no URL or credential snapshot field |
| `cli-exit-codes` | `node --test --test-name-pattern='@claim:cli-exit-codes' site/tests/claims.test.mjs` | PASS — exits 0, 1, 2, and 3 |
| `pro-ci-policy` | `node --test --test-name-pattern='@claim:pro-ci-policy' site/tests/claims.test.mjs` | PASS — recorded verification and risk threshold |

The landing page, demo, README, privacy policy, and terms were cross-checked
against the manifest. Database access, formats, redaction, telemetry, privacy,
offline, license, price, and automation statements all map to a claim above.
No public claim was missing or left untested.

## Clean checkout, package, and CLI

- `npm ci`: passed; 21 packages installed and zero vulnerabilities.
- `npm test`: passed; 23 Rust tests, 21 Node tests, and 32 Playwright tests
  passed with two intentional desktop-project skips.
- `npm run lint`: TypeScript, rustfmt, and clippy with warnings denied passed.
- `npm run build`: passed and produced `dist/bin/sds` and `dist/site`.
- `cargo package --allow-dirty`: passed; 61 files, 437.7 KiB unpacked and
  199.7 KiB compressed.

The packaged crate installed into an empty consumer root and reported
`sds 0.1.0`. Its `sds demo --json` created a unique temporary sandbox and
reported five changes: four high, one medium, three destructive, and one
ORM-invisible. The installed binary also completed the documented fixture
comparison and Markdown export.

Direct normal, empty, invalid, boundary, and recovery checks passed. Identical
snapshots rendered “No drift detected.” Unsupported SQLite and a missing file
returned exit 2. A missing redaction key was rejected before connection with
exit 2. An unlicensed CI check returned exit 3. The generated review contained
no executable repair SQL.

## Real database checks

PostgreSQL 16.15 and MariaDB 10.11.14 were installed and started as isolated
temporary processes on non-default local ports. Both real integration tests
passed, then both processes were stopped.

- PostgreSQL used a distinct non-owner reader. Its attempted row insert was
  rejected, and a predicate-only view change produced one ORM-invisible
  modified-view difference.
- MariaDB first failed closed when the reader lacked `SHOW VIEW`, then captured
  the definition after that read-only metadata grant. A predicate-only change
  produced one ORM-invisible difference, and the row insert was rejected.

## Live browser, accessibility, privacy, and routes

- The factory URL verifier passed on `/` and `/demo/`: HTTPS 200, useful titles,
  `lang=en`, one h1, one main, image alternatives, named buttons, and no console
  errors.
- Fresh desktop and phone Axe runs found zero serious or critical violations on
  root, demo, privacy, terms, and the designed 404.
- Keyboard Tab exposed the skip link with a 3px coral focus ring; Enter moved
  focus to main. Under reduced motion, transitions were 0.01ms and scroll was
  automatic. The focus result was checked after a rendered frame.
- At 390px, normal and 200% text layouts had 390px document width. Footer links
  measured at least 44×44 CSS pixels.
- `/demo/`, `/privacy/`, `/terms/`, and `/404.html` returned 200. A deliberately
  missing path correctly returned HTTP 404 with the designed not-found title,
  one h1, and a working home link. This expected 404 is not a defect.
- Every discovered link returned 200, including the repository and Sociobot
  contact links. All route titles, canonical metadata, sitemap entries, and
  legal pages were present.
- A fresh service-worker context obtained control and reloaded `/demo/` offline
  with the four-change review and offline notice intact. Only the current
  `sds-6ea2a30be53d` cache was present.
- A normal root/sample flow made only same-origin requests. A separate invalid
  license flow made one direct Sociobot verification request, removed the token
  from the URL, and kept the inactive notice after reload without a second
  request. Static inspection found no analytics, remote font, or third-party
  script.
- Live responses include CSP with `frame-ancestors 'none'`, HSTS,
  Permissions-Policy, X-Frame-Options, Referrer-Policy, and
  X-Content-Type-Options. Hashed assets are immutable for one year; `sw.js` is
  `no-cache`.
- Mobile Lighthouse 13.4.1: Performance 100, Accessibility 100, Best Practices
  100, SEO 100; FCP/LCP 923ms, TBT 13ms, CLS 0. The first Lighthouse browser
  process crashed before measurement; a fresh rerun passed, and the crash did
  not reproduce in the browser matrix.
- Production build sizes were 7,495 bytes for main JS, 15,326 bytes for CSS,
  zero font bytes, and 53,184 bytes for the desktop-only hero image.

This product is a CLI plus static site. It has no product backend, tenant
storage, health endpoint, or persistence service, so backend tenant/restart/
429 checks do not apply. No shared database or external product service was
used.

## Earlier finding disposition

| Earlier finding | Current proof |
| --- | --- |
| Verification 1: security headers absent | Required headers are live on HTML, JS, and worker responses. |
| Verification 1: immutable assets absent | Hashed JS is one-year immutable; worker is no-cache. |
| Verification 1: redaction key checked too late | Installed CLI rejects the missing key before connection and creates no output. |
| Verification 2: worker precached an unavailable config file | Fresh live worker controls the page and reloads the four-change demo offline. |
| Verification 3: view definitions not captured | Both real dialect regressions detect definition-only view drift. |
| Verification 3: mobile LCP over 2.5s | Fresh mobile LCP is 923ms. |
| Verification 4: PostgreSQL non-owner sees null definitions | Real non-owner PostgreSQL regression passes through `pg_get_viewdef`. |
| Verification 4: footer Terms target under 44px | Live phone target is 44×44 CSS pixels; full matrix passes. |
| Verification 5: claims manifest/tests absent | All 17 claims exist and passed individually. |
| Verification 5: CLI/browser demo absent | Installed `sds demo` and one-click `/demo/` both pass. |
| Verification 5: first screen misses audience/action | Job, developers, required action, and result are visible before scrolling on desktop and phone. |
| Verification 5: advertised checkout returned 404 | Checkout is honestly marked closed and no purchase link is exposed. |
| Verification 5: 200% text overflow | Live 390px document remains exactly 390px wide. |
| Verification 5: route/metadata/build identity gaps | Route titles, 404, canonical/social metadata, sitemap, icon, and build identity are live. |
| Verification 5: invalid-license notice disappears | Notice persists over reload with one verification request. |
| Verification 5: copy audit absent | `.factory/copy-audit.md` exists; banned-word and terminology scans are clean. |

## Remaining external task

Factory billing registration is still closed. The product states this plainly
and exposes no broken checkout path, so it is not a finding. Registration and a
future checkout-opening change remain outside this repository review.
