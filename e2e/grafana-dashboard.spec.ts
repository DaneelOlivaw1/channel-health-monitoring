import { test, expect, Page } from '@playwright/test';

const GRAFANA_URL = process.env.GRAFANA_URL || 'http://localhost:3000';
const GRAFANA_USER = process.env.GRAFANA_USER || 'admin';
const GRAFANA_PASSWORD = process.env.GRAFANA_PASSWORD || 'admin';
const DASHBOARD_UID = 'channel-health-prometheus';

function authHeaders() {
  return {
    'Authorization': 'Basic ' + Buffer.from(`${GRAFANA_USER}:${GRAFANA_PASSWORD}`).toString('base64'),
  };
}

test.describe('Grafana Dashboard E2E', () => {
  test.beforeEach(async ({ page }) => {
    await page.setExtraHTTPHeaders(authHeaders());
  });

  test('dashboard loads successfully', async ({ page }) => {
    await page.goto(`${GRAFANA_URL}/d/${DASHBOARD_UID}`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    await expect(page).toHaveTitle(/渠道健康状况/);

    await page.screenshot({
      path: 'test-results/screenshots/dashboard-full.png',
      fullPage: true,
    });
  });

  test('availability panel - solo view', async ({ page }) => {
    await page.goto(`${GRAFANA_URL}/d-solo/${DASHBOARD_UID}?panelId=21`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(3000);

    await page.screenshot({
      path: 'test-results/screenshots/panel-availability.png',
    });
  });

  test('cache panel - solo view', async ({ page }) => {
    await page.goto(`${GRAFANA_URL}/d-solo/${DASHBOARD_UID}?panelId=22`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(3000);

    await page.screenshot({
      path: 'test-results/screenshots/panel-cache.png',
    });
  });

  test('cost panel - solo view', async ({ page }) => {
    await page.goto(`${GRAFANA_URL}/d-solo/${DASHBOARD_UID}?panelId=23`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(3000);

    await page.screenshot({
      path: 'test-results/screenshots/panel-cost.png',
    });
  });

  test('table panel - solo view', async ({ page }) => {
    await page.goto(`${GRAFANA_URL}/d-solo/${DASHBOARD_UID}?panelId=31`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(3000);

    await page.screenshot({
      path: 'test-results/screenshots/panel-table.png',
    });
  });
});
