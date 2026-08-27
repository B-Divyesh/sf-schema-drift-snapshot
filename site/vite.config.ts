import { createHash } from 'node:crypto';
import { promises as fs } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig, type Plugin } from 'vite';

const siteRoot = fileURLToPath(new URL('.', import.meta.url));
const outputRoot = path.resolve(siteRoot, '../dist/site');

async function filesUnder(directory: string, base = directory): Promise<string[]> {
  const entries = await fs.readdir(directory, { withFileTypes: true });
  const files = await Promise.all(entries.map(async (entry) => {
    const fullPath = path.join(directory, entry.name);
    return entry.isDirectory() ? filesUnder(fullPath, base) : [`/${path.relative(base, fullPath).replaceAll(path.sep, '/')}`];
  }));
  return files.flat();
}

function offlineShell(): Plugin {
  return {
    name: 'sds-offline-shell',
    apply: 'build',
    async closeBundle() {
      const files = (await filesUnder(outputRoot))
        .filter((file) => file !== '/sw.js' && !file.endsWith('.map'))
        .sort();
      const version = createHash('sha256').update(files.join('\n')).digest('hex').slice(0, 12);
      const shell = [...new Set(['/', '/privacy/', '/terms/', ...files])];
      const worker = `const CACHE = 'sds-${version}';\nconst SHELL = ${JSON.stringify(shell)};\nself.addEventListener('install', event => event.waitUntil(caches.open(CACHE).then(cache => cache.addAll(SHELL)).then(() => self.skipWaiting())));\nself.addEventListener('activate', event => event.waitUntil(caches.keys().then(keys => Promise.all(keys.filter(key => key.startsWith('sds-') && key !== CACHE).map(key => caches.delete(key)))).then(() => self.clients.claim())));\nself.addEventListener('fetch', event => { if (event.request.method !== 'GET' || new URL(event.request.url).origin !== self.location.origin) return; event.respondWith(caches.match(event.request).then(hit => hit || fetch(event.request).then(response => { const copy = response.clone(); caches.open(CACHE).then(cache => cache.put(event.request, copy)); return response; }).catch(() => caches.match('/index.html')))); });\n`;
      await fs.writeFile(path.join(outputRoot, 'sw.js'), worker);
    },
  };
}

export default defineConfig({
  root: siteRoot,
  publicDir: path.join(siteRoot, 'public'),
  build: {
    outDir: outputRoot,
    emptyOutDir: true,
    target: 'es2022',
    cssCodeSplit: true,
    sourcemap: false,
    rollupOptions: {
      input: {
        main: path.join(siteRoot, 'index.html'),
        privacy: path.join(siteRoot, 'privacy/index.html'),
        terms: path.join(siteRoot, 'terms/index.html'),
      },
    },
  },
  plugins: [offlineShell()],
});
