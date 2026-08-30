# Schema Drift Snapshot — independent verification 5 handoff

Work order: `schema-drift-snapshot-verify-5`

Candidate: `5d8b4853a82d68028b71d21f0127e7165039baaf`

URL: <https://schema-drift-snapshot.sociobot.in/>

Date: 2026-08-30 UTC

## Release result

**FAIL — do not release.** Fresh live and clean-consumer verification found
four release blockers:

1. `.factory/claims.json` is missing, so the mandatory first gate and all
   claim-tagged demo tests are absent.
2. The cold first screen does not name the audience or offer the required
   “Try it with sample data” primary action. The CLI has no `demo` command,
   `/demo` is only the root fallback with no sandbox banner, and
   `.factory/demo.md` is missing.
3. The live $49 checkout link returns HTTP 404 with
   `{"error":"enabled factory product","status":404}`.
4. At 390px and 200% text size, the document expands to 494px and requires
   horizontal scrolling.

Additional P2 findings: no real 404, missing canonical/social/apple metadata,
no demo route title or sitemap entry, no footer build identity, cached invalid
license notices disappear on reload, and `.factory/copy-audit.md` is absent.

Full evidence and retest instructions are in
`.factory/verification-5.md`.

## What passed

- Clean `npm ci`, `npm test`, standalone TypeScript/format/clippy checks, exact
  `npm run build`, `cargo package`, and clean consumer install.
- Real PostgreSQL 16.15 and MariaDB 10.11.14 read-only capture tests, plus the
  installed package's snapshot/compare/redaction/error workflows.
- Desktop and 390px normal-size layout, keyboard use, focus, error recovery,
  zero axe serious/critical findings, reduced motion, same-origin demo request
  log, security headers, offline reload, and service-worker replacement.
- License verify rate limit: 30 successful burst requests; request 31 returned
  429 with `Retry-After: 2`.
- Lighthouse mobile 100/100/100/100; LCP 0.9s, TBT 80ms, CLS 0.
- Every public build resource tested byte-matches the candidate deployment.

## Verification commands

```sh
npm ci
npm test
npm run typecheck
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
npm run build
cargo package
/opt/fleet/lib/verify-url.sh https://schema-drift-snapshot.sociobot.in <evidence-dir>
```

Database tests were rerun with disposable local admin URLs via
`SDS_TEST_POSTGRES_ADMIN_URL` and `SDS_TEST_MYSQL_ADMIN_URL`. No production data
or service was touched.

## Handoff state

Only `.factory/verification-5.md` and this handoff were changed. Product code
was not modified. Fix the blockers above, regenerate the live deployment, and
repeat claims-first verification before release.
