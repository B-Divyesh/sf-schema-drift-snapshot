import { expect, test } from '@playwright/test';

test('@claim:browser-demo-local keeps edited snapshot text on this origin', async ({ page }) => {
  const requests: string[] = [];
  page.on('request', (request) => requests.push(request.url()));
  await page.addInitScript(() => {
    localStorage.setItem('sb_license:schema-drift-snapshot', 'real-mode-token');
    localStorage.setItem('real:schema-draft', 'unchanged');
  });
  await page.goto('/demo/');
  await page.locator('#before-snapshot').fill(JSON.stringify({
    schema_version: 1,
    dialect: 'postgresql',
    captured_at: '2026-08-30T00:00:00Z',
    objects: [],
  }));
  await page.locator('#after-snapshot').fill(JSON.stringify({
    schema_version: 1,
    dialect: 'postgresql',
    captured_at: '2026-08-30T00:01:00Z',
    objects: [],
  }));
  await page.getByRole('button', { name: 'Compare locally' }).click();
  await expect(page.locator('#review-summary')).toHaveText('No drift detected');
  expect(requests.every((url) => new URL(url).origin === 'http://127.0.0.1:4173')).toBe(true);
  expect(await page.evaluate(() => localStorage.getItem('sb_license:schema-drift-snapshot'))).toBe('real-mode-token');
  expect(await page.evaluate(() => localStorage.getItem('real:schema-draft'))).toBe('unchanged');
});

test('@claim:sample-review opens with four classified browser changes', async ({ page }) => {
  await page.goto('/demo/');
  await expect(page.locator('.demo-banner')).toContainText('Demo — sample data, nothing is saved');
  await expect(page.locator('#review-summary')).toHaveText('4 differences found');
  await expect(page.locator('.change-list li')).toHaveCount(4);
  await page.locator('[data-reset-demo]').first().click();
  await expect(page.locator('.change-list li')).toHaveCount(4);
});

test('@claim:offline-reload reopens the demo after the first visit', async ({ browser }) => {
  const context = await browser.newContext();
  const page = await context.newPage();
  await page.goto('/demo/');
  await page.evaluate(() => navigator.serviceWorker.ready);
  await page.reload();
  await expect.poll(() => page.evaluate(() => navigator.serviceWorker.controller !== null)).toBe(true);
  await context.setOffline(true);
  await page.reload();
  await expect(page).toHaveTitle('Demo — Schema Drift Snapshot');
  await expect(page.locator('#review-summary')).toHaveText('4 differences found');
  await context.close();
});

test('@claim:daily-license-check keeps an invalid notice without a second request', async ({ page }) => {
  let requests = 0;
  await page.route('https://api.sociobot.in/**', async (route) => {
    requests += 1;
    await route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ valid: false, reason: 'invalid', expires_at: null }) });
  });
  await page.goto('/?license=invalid-token#pricing');
  await expect(page.locator('#license-status')).toContainText('License no longer active (invalid)');
  await page.reload();
  await expect(page.locator('#license-status')).toContainText('License no longer active (invalid)');
  expect(requests).toBe(1);
});

test('@claim:no-analytics sends no tracking requests during the sample flow', async ({ page }) => {
  const thirdParty: string[] = [];
  page.on('request', (request) => {
    const url = new URL(request.url());
    if (url.origin !== 'http://127.0.0.1:4173') thirdParty.push(url.href);
  });
  await page.goto('/demo/');
  await page.getByRole('button', { name: 'Compare locally' }).click();
  expect(thirdParty).toEqual([]);
  expect(await page.locator('script[src]').evaluateAll((scripts) => scripts.map((script) => (script as HTMLScriptElement).src).every((url) => new URL(url).origin === location.origin))).toBe(true);
});

test('@claim:price-copy shows the same one-time price in product and legal copy', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('#pricing')).toContainText('$49');
  await expect(page.locator('#pricing')).toContainText('one time');
  await page.goto('/terms/');
  await expect(page.locator('main')).toContainText('$49 one-time Pro purchase');
});
