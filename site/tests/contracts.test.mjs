import assert from 'node:assert/strict';
import { readFile, stat } from 'node:fs/promises';
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

test('design record contains product-specific tokens and provenance', async () => {
  const design = await readFile(new URL('.factory/design.md', root), 'utf8');
  assert.match(design, /paper-cut incident diorama/i);
  assert.match(design, /provenance/i);
  assert.match(design, /prefers-reduced-motion/);
});
