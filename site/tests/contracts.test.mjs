import assert from 'node:assert/strict';
import { access, readFile, stat } from 'node:fs/promises';
import test from 'node:test';

const root = new URL('../../', import.meta.url);

test('landing page exposes one h1 and required landmarks', async () => {
  const html = await readFile(new URL('site/index.html', root), 'utf8');
  assert.equal((html.match(/<h1\b/g) ?? []).length, 1);
  assert.match(html, /<html lang="en">/);
  assert.match(html, /<main id="main">/);
  assert.match(html, /href="\/privacy\/"/);
  assert.match(html, /href="\/terms\/"/);
});

test('paid unlock uses the product-scoped storage and Sociobot API', async () => {
  const source = await readFile(new URL('site/src/main.ts', root), 'utf8');
  assert.match(source, /sb_license:/);
  assert.match(source, /api\.sociobot\.in/);
  assert.match(source, /history\.replaceState/);
  assert.match(source, /86_400_000/);
});

test('hero stays within the mobile image budget', async () => {
  const image = await stat(new URL('site/public/assets/schema-diorama.webp', root));
  assert.ok(image.size <= 300 * 1024, `hero is ${image.size} bytes`);
});

test('compact first render defers the nonessential hero artwork', async () => {
  const html = await readFile(new URL('site/index.html', root), 'utf8');
  const css = await readFile(new URL('site/src/style.css', root), 'utf8');
  assert.doesNotMatch(html, /rel="preload"[^>]+schema-diorama/);
  assert.match(html, /schema-diorama\.webp[^>]+loading="lazy"[^>]+decoding="async"[^>]+fetchpriority="low"/);
  assert.match(css, /@media \(max-width: 620px\)[\s\S]*?\.hero-figure \{ display: none; \}/);
});

test('design record contains product-specific tokens and provenance', async () => {
  const design = await readFile(new URL('.factory/design.md', root), 'utf8');
  assert.match(design, /paper-cut incident diorama/i);
  assert.match(design, /provenance/i);
  assert.match(design, /prefers-reduced-motion/);
});

test('Azure deployment policy preserves the security and cache contract', async () => {
  const declared = await readFile(new URL('site/public/_headers', root), 'utf8');
  const config = JSON.parse(await readFile(new URL('site/public/staticwebapp.config.json', root), 'utf8'));

  assert.match(declared, /Content-Security-Policy:/);
  assert.equal(
    config.globalHeaders['Content-Security-Policy'],
    "default-src 'self'; connect-src 'self' https://api.sociobot.in; img-src 'self'; style-src 'self'; script-src 'self'; object-src 'none'; base-uri 'self'; frame-ancestors 'none'",
  );
  assert.equal(config.globalHeaders['Permissions-Policy'], 'camera=(), microphone=(), geolocation=(), payment=()');
  assert.equal(config.globalHeaders['X-Frame-Options'], 'DENY');

  const headersFor = (route) => config.routes.find((entry) => entry.route === route)?.headers;
  assert.deepEqual(headersFor('/assets/*'), { 'Cache-Control': 'public, max-age=31536000, immutable' });
  assert.deepEqual(headersFor('/sw.js'), { 'Cache-Control': 'no-cache' });
});

test('generated offline shell precaches only publicly served build assets', async () => {
  const worker = await readFile(new URL('dist/site/sw.js', root), 'utf8');
  const shellMatch = worker.match(/const SHELL = (\[[^;]+\]);/);
  assert.ok(shellMatch, 'generated worker must declare its precache shell');
  const shell = JSON.parse(shellMatch[1]);

  assert.ok(shell.includes('/'), 'offline shell must include the app root');
  assert.ok(shell.includes('/privacy/'));
  assert.ok(shell.includes('/terms/'));
  assert.ok(!shell.includes('/staticwebapp.config.json'), 'Azure deployment metadata is not publicly served');
  assert.ok(!shell.includes('/_headers'), 'portable host metadata is not a runtime asset');
  assert.ok(!shell.includes('/sw.js'), 'the worker must not precache itself');

  for (const asset of shell.filter((entry) => entry !== '/' && !entry.endsWith('/'))) {
    await access(new URL(`dist/site${asset}`, root));
  }
});
