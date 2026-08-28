import AxeBuilder from '@axe-core/playwright';
import { expect, test } from '@playwright/test';

test('landing page is accessible and the local review classifies drift', async ({ page }) => {
  const runtimeErrors: string[] = [];
  page.on('pageerror', (error) => runtimeErrors.push(error.message));
  page.on('console', (message) => { if (message.type() === 'error') runtimeErrors.push(message.text()); });
  await page.goto('/');
  await expect(page).toHaveTitle(/Schema Drift Snapshot/);
  await expect(page.locator('h1')).toHaveCount(1);
  const accessibility = await new AxeBuilder({ page }).analyze();
  expect(accessibility.violations.filter((item) => ['serious', 'critical'].includes(item.impact ?? ''))).toEqual([]);
  await page.locator('#compare-button').click();
  await expect(page.locator('#review-summary')).toContainText('4 differences');
  await expect(page.locator('.change-list li')).toHaveCount(4);
  expect(runtimeErrors).toEqual([]);
});

test('invalid input has a useful announced error and keyboard reset works', async ({ page }) => {
  await page.goto('/#demo');
  await page.locator('#before-snapshot').fill('{oops');
  await page.locator('#compare-button').focus();
  await page.keyboard.press('Enter');
  await expect(page.locator('#review-error')).toContainText('not valid JSON');
  await page.locator('#reset-demo').focus();
  await page.keyboard.press('Enter');
  await expect(page.locator('#review-empty')).toBeVisible();
});

test('license return token is stored, stripped, and verified', async ({ page }) => {
  await page.route('https://api.sociobot.in/**', async (route) => {
    await route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ valid: true, reason: 'ok', expires_at: null }) });
  });
  await page.goto('/?license=test-token#pricing');
  await expect(page).toHaveURL(/\/#pricing$/);
  await expect(page.locator('#license-status')).toContainText('unlocked');
  expect(await page.evaluate(() => localStorage.getItem('sb_license:schema-drift-snapshot'))).toBe('test-token');
});

test('mobile layout has no page-level horizontal overflow', async ({ page }, testInfo) => {
  test.skip(testInfo.project.name !== 'mobile');
  await page.goto('/');
  const widths = await page.evaluate(() => ({ scroll: document.documentElement.scrollWidth, client: document.documentElement.clientWidth }));
  expect(widths.scroll).toBeLessThanOrEqual(widths.client + 1);
  await expect(page.locator('.hero-actions .button')).toHaveCount(2);
});

test('legal routes have clear titles and one h1', async ({ page }) => {
  for (const route of ['/privacy/', '/terms/']) {
    await page.goto(route);
    await expect(page.locator('main h1')).toHaveCount(1);
    await expect(page).toHaveTitle(/Schema Drift Snapshot/);
  }
});

test('installed service worker controls an offline root reload', async ({ page }) => {
  await page.goto('/');
  await page.evaluate(() => navigator.serviceWorker.ready);
  await page.reload();
  await expect.poll(() => page.evaluate(() => navigator.serviceWorker.controller !== null)).toBe(true);

  await page.context().setOffline(true);
  await page.reload();
  await expect(page).toHaveTitle(/Schema Drift Snapshot/);
  await expect(page.locator('h1')).toHaveCount(1);
});
