import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const root = new URL('../../', import.meta.url);
const cwd = fileURLToPath(root);

function sds(args) {
  return execFileSync('cargo', ['run', '--quiet', '--', ...args], { cwd, encoding: 'utf8' });
}

async function withDemo(run) {
  const parent = await mkdtemp(path.join(tmpdir(), 'sds-claim-'));
  const sandbox = path.join(parent, 'demo');
  try {
    const result = JSON.parse(sds(['demo', '--output', sandbox, '--json']));
    await run({ result, sandbox });
  } finally {
    await rm(parent, { recursive: true, force: true });
  }
}

test('@claim:cli-demo writes the bundled five-change sample to a sandbox', async () => {
  await withDemo(async ({ result, sandbox }) => {
    assert.equal(result.summary.total, 5);
    assert.equal(result.summary.high, 4);
    assert.equal(result.summary.medium, 1);
    assert.equal(result.sandbox, sandbox);
    assert.match(await readFile(path.join(sandbox, 'drift-review.md'), 'utf8'), /\*\*5\*\* total differences/);
  });
});

test('@claim:no-repair-sql produces explanation without executable repair statements', async () => {
  await withDemo(async ({ sandbox }) => {
    const report = await readFile(path.join(sandbox, 'drift-review.md'), 'utf8');
    assert.match(report, /no executable repair SQL/);
    assert.doesNotMatch(report, /(?:^|\n)\s*(?:ALTER|CREATE|DELETE|DROP|INSERT|UPDATE)\s+/im);
  });
});

test('@claim:review-formats exports observable Markdown and JSON classifications', async () => {
  const parent = await mkdtemp(path.join(tmpdir(), 'sds-formats-'));
  const reportPath = path.join(parent, 'review.md');
  try {
    const output = sds([
      'compare', '--before', 'examples/fixtures/expected.sds.json', '--after',
      'examples/fixtures/observed.sds.json', '--output', reportPath, '--json',
    ]);
    const review = JSON.parse(output);
    assert.equal(review.summary.total, 5);
    assert.match(await readFile(reportPath, 'utf8'), /# Schema drift review/);
  } finally {
    await rm(parent, { recursive: true, force: true });
  }
});

test('@claim:deterministic-redaction hides names consistently', () => {
  const output = execFileSync('cargo', [
    'test', 'redact::tests::redaction_hides_names_and_definition_content_deterministically', '--', '--exact',
  ], { cwd, encoding: 'utf8' });
  assert.match(output, /1 passed/);
});

test('@claim:catalog-only-capture enforces read-only catalog access', async () => {
  const source = await readFile(new URL('../../src/capture.rs', import.meta.url), 'utf8');
  assert.match(source, /SET SESSION CHARACTERISTICS AS TRANSACTION READ ONLY/);
  assert.match(source, /SET SESSION TRANSACTION READ ONLY/);
  assert.match(source, /information_schema\.tables/);
  assert.match(source, /information_schema\.COLUMNS/);
  assert.doesNotMatch(source, /(?:query|query_drop)\(\s*"(?:DELETE|INSERT|UPDATE)\s/im);
});

test('@claim:free-compare-needs-no-license runs from the bundled sample', () => {
  const output = sds([
    'compare', '--before', 'examples/fixtures/expected.sds.json', '--after',
    'examples/fixtures/observed.sds.json', '--json',
  ]);
  assert.equal(JSON.parse(output).summary.total, 5);
});

test('@claim:cli-no-telemetry has no telemetry or analytics client', async () => {
  const manifest = await readFile(new URL('../../Cargo.toml', import.meta.url), 'utf8');
  const sources = await Promise.all([
    'capture.rs', 'diff.rs', 'lib.rs', 'license.rs', 'main.rs', 'model.rs', 'redact.rs', 'report.rs',
  ].map((name) => readFile(new URL(`../../src/${name}`, import.meta.url), 'utf8')));
  const searchable = `${manifest}\n${sources.join('\n')}`;
  assert.doesNotMatch(searchable, /analytics|telemetry|sentry|datadog|segment|mixpanel/i);
});

test('@claim:database-url-support recognizes the documented PostgreSQL and MySQL URL schemes', () => {
  const output = execFileSync('cargo', [
    'test', 'capture::tests::detects_documented_database_urls', '--', '--exact',
  ], { cwd, encoding: 'utf8' });
  assert.match(output, /1 passed/);
});

test('@claim:credential-hygiene keeps database URLs out of snapshot documents', async () => {
  const model = await readFile(new URL('../../src/model.rs', import.meta.url), 'utf8');
  const capture = await readFile(new URL('../../src/capture.rs', import.meta.url), 'utf8');
  const snapshotFields = model.match(/pub struct Snapshot \{([\s\S]*?)\n\}/)?.[1] ?? '';
  assert.doesNotMatch(snapshotFields, /url|password|credential/i);
  assert.match(capture, /source: "database catalog"\.to_owned\(\)/);
  assert.doesNotMatch(capture, /source:\s*url/);
});

test('@claim:cli-exit-codes returns the documented automation statuses without prompting', async () => {
  const config = await mkdtemp(path.join(tmpdir(), 'sds-no-license-'));
  try {
    const run = (args) => spawnSync('cargo', ['run', '--quiet', '--', ...args], {
      cwd,
      encoding: 'utf8',
      env: { ...process.env, XDG_CONFIG_HOME: config },
    });
    assert.equal(run(['compare', '--before', 'examples/fixtures/expected.sds.json', '--after', 'examples/fixtures/expected.sds.json']).status, 0);
    assert.equal(run(['compare', '--before', 'missing.json', '--after', 'examples/fixtures/observed.sds.json']).status, 2);
    assert.equal(run(['check', '--before', 'examples/fixtures/expected.sds.json', '--after', 'examples/fixtures/observed.sds.json']).status, 3);

    const licensed = execFileSync('cargo', [
      'test', '--test', 'cli', 'pro_check_verifies_license_and_applies_threshold', '--', '--exact',
    ], { cwd, encoding: 'utf8' });
    assert.match(licensed, /1 passed/);
  } finally {
    await rm(config, { recursive: true, force: true });
  }
});

test('@claim:pro-ci-policy verifies a license and applies a configured risk threshold', () => {
  const output = execFileSync('cargo', [
    'test', '--test', 'cli', 'pro_check_verifies_license_and_applies_threshold', '--', '--exact',
  ], { cwd, encoding: 'utf8' });
  assert.match(output, /1 passed/);
});
