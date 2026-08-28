# Independent verification — FAIL

Work order: `schema-drift-snapshot-verify-1`  
Candidate: `7df275e8fedc88e39cccbe95362ac280d640a261`  
Verified URL: <https://schema-drift-snapshot.sociobot.in/>  
Date: 2026-08-28 UTC

## Decision

**FAIL — do not release this deployment as-is.** The candidate source and its
functional product checks pass, and the live HTML and every referenced asset
hash-match the candidate production build. The live CDN, however, does not
apply the required browser security and caching policies shipped in
`dist/site/_headers`. This is a deployment defect, not a content mismatch.

## Release-blocking defects

### P1 — live deployment omits the shipped security response policy

`site/public/_headers` declares a restrictive CSP, `Permissions-Policy`, and
`X-Frame-Options: DENY`. Fresh HTTPS responses for `/`, `/privacy/`, and
`/terms/` omit all three. They do include HSTS, `Referrer-Policy`, and
`X-Content-Type-Options`.

This leaves the production product without its declared content-source,
framing, and browser-feature restrictions. It matters particularly because the
site can retain a Pro license token in local storage. Configure the deployment
platform/CDN to emit these headers, then re-verify the live response.

### P2 — live assets are not immutably cached

`_headers` declares `Cache-Control: public, max-age=31536000, immutable` for
`/assets/*` and `no-cache` for `/sw.js`. Fresh live responses for the hashed
JS/CSS assets and `/sw.js` instead return
`Cache-Control: public, must-revalidate, max-age=30`. This defeats the shipped
hashed-asset caching plan and causes needless revalidation. Apply the deployed
cache policy and recheck headers.

### P3 — invalid redaction configuration is validated after database capture

In `snapshot`, `--redact-names` without a key is checked only after
`capture(...)`. A supported but unreachable PostgreSQL URL therefore reports a
connection error rather than the missing-key error; against a reachable
database it will first read the catalog, then reject the invocation. Validate
the key before connecting so invalid input fails immediately and predictably.
No snapshot file is written in this path.

## Passing evidence

### Clean checkout, quality gates, and package

- Clean checkout began at exactly `7df275e8fedc88e39cccbe95362ac280d640a261`.
- `npm ci` completed with 0 audited vulnerabilities.
- `npm test` passed (Rust tests, TypeScript/contracts, production-site build,
  and Playwright suite).
- `cargo fmt --all -- --check` and
  `cargo clippy --all-targets -- -D warnings` passed.
- Exact production command `npm run build` passed and produced `dist/bin/sds`
  (6,399,008 bytes) and `dist/site`.
- `cargo package --allow-dirty` passed. The resulting crate was installed from
  `target/package/schema-drift-snapshot-0.1.0` into a fresh temporary consumer
  root using `cargo install --path ... --root ...`; the installed `sds 0.1.0`
  successfully compared the documented fixtures.

### CLI end to end

- The release binary's documented `compare` workflow against
  `expected.sds.json` and `observed.sds.json` produced JSON with 5 classified
  changes: 4 high, 1 medium, 3 destructive, and 1 ORM-invisible.
- Markdown output was written atomically, included the review checklist, and
  contained no `DROP TABLE`, `ALTER TABLE`, `CREATE TABLE`, `DELETE FROM`, or
  `UPDATE` SQL.
- Comparing the fixture to itself produced the clear `No drift detected` empty
  state. Unsupported `sqlite:` URL, missing snapshot, and unlicensed Pro
  `check` all failed with safe, actionable errors (exit codes 2, 2, and 3).
- Help states the read-only/no-row-data/no-repair-SQL boundary. No telemetry or
  non-Sociobot product network endpoint was found by source audit; the site
  makes no outbound request on a normal first load.
- No disposable PostgreSQL/MySQL server is present in this container, so a
  real-driver capture against both dialects remains an environmental coverage
  gap. Fixture, decoder, URL-dispatch, error, redaction, reporting, and
  package-install paths were exercised.

### Browser, accessibility, PWA, and performance

- Independent Chromium checks on the built preview and the live page: one
  `h1`, one `main`, `lang=en`, zero console/page errors, and zero axe
  serious/critical findings. The live mobile page also had no horizontal
  overflow at 390 px.
- Desktop and 390 x 844 mobile keyboard paths worked: Tab first reaches the
  skip link with a visible `rgb(141, 48, 37) solid 3px` outline; Enter ran the
  normal comparison, showed an announced invalid-JSON recovery message, and
  reset to the empty state. With reduced motion, transition duration computed
  to `0.00001s`.
- Browser-local normal comparison reported 4 differences. No normal-load
  outbound requests were observed.
- PWA: service worker reached `activated` and controlled the page; an offline
  reload retained the correct title and one `h1`. The generated worker uses a
  versioned cache (`sds-34241354cbe8`), `skipWaiting`, `clients.claim`, and
  removes earlier `sds-*` caches on activation.
- Lighthouse 13 on the built preview: Performance 100, Accessibility 100,
  Best Practices 100, SEO 100; LCP 1,357 ms, CLS 0, TBT 22 ms.
- Production bytes: JS 8,056 bytes, CSS 13,863 bytes, fonts 0 bytes, hero
  WebP 53,184 bytes — all within the stated budgets.

### Live-match and privacy evidence

- Live `/` HTML SHA-256:
  `3bb00cf120bb05d75d66048d5d5fe9ed7402b7d042aa0e05d813f314e9bf8ce2`.
  It is byte-identical to `dist/site/index.html`.
- The live `main-CSGRf9Yn.js`, `style-DJXGkVpm.js`, `style-DNyT1wnc.css`, and
  `schema-diorama.webp` all SHA-256 match the candidate build.
- Static audit found no analytics, telemetry, remote fonts, or third-party
  scripts. The only product API endpoint is the documented Sociobot license
  verification endpoint; normal first loads made no external request.
- Privacy and terms routes are present and live; the free browser demo is
  local-only. The deployment/header defects above prevent a PASS despite these
  passing checks.

## Retest

After configuring the actual deployment response rules, run:

```sh
curl -sSI https://schema-drift-snapshot.sociobot.in/
curl -sSI https://schema-drift-snapshot.sociobot.in/assets/main-CSGRf9Yn.js
curl -sSI https://schema-drift-snapshot.sociobot.in/sw.js
```

Confirm CSP, `Permissions-Policy`, `X-Frame-Options`, immutable `/assets/*`,
and no-cache `/sw.js`, then re-run `npm ci && npm test && npm run build` and a
disposable PostgreSQL/MySQL read-only capture smoke test.
