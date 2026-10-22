import { defineConfig } from '@playwright/test';
const deployed = process.env.PLAYWRIGHT_BASE_URL;
export default defineConfig({
  testDir: './tests/browser',
  timeout: 120_000,
  expect: { timeout: 60_000 },
  workers: 1,
  retries: 1,
  reporter: [['list'], ['html', { open: 'never' }]],
  use: {
    baseURL: deployed || 'http://127.0.0.1:4173/lord-of-towers/',
    viewport: { width: 1440, height: 900 },
    screenshot: 'only-on-failure',
    trace: 'retain-on-failure',
  },
  projects: [
    { name: 'chromium', testIgnore: '**/mobile.spec.mjs', use: { browserName: 'chromium', launchOptions: { args: ['--enable-webgl', '--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'] } } },
    { name: 'firefox', testIgnore: '**/mobile.spec.mjs', use: { headless: !process.env.CI, browserName: 'firefox', launchOptions: { firefoxUserPrefs: { 'webgl.force-enabled': true, 'webgl.disabled': false } } } },
    { name: 'chromium-mobile', testMatch: '**/mobile.spec.mjs', timeout: 180_000, use: { browserName: 'chromium', viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true, deviceScaleFactor: 2, launchOptions: { args: ['--enable-webgl', '--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'] } } },
  ],
  webServer: deployed ? undefined : { command: 'python3 scripts/serve-web.py', port: 4173, reuseExistingServer: !process.env.CI },
});
