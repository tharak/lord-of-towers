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
    { name: 'chromium', use: { browserName: 'chromium', launchOptions: { args: ['--enable-webgl', '--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'] } } },
    { name: 'firefox', use: { browserName: 'firefox', launchOptions: { firefoxUserPrefs: { 'webgl.force-enabled': true, 'webgl.disabled': false } } } },
  ],
  webServer: deployed ? undefined : { command: 'python3 scripts/serve-web.py', port: 4173, reuseExistingServer: !process.env.CI },
});
