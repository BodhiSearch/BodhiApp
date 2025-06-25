import { dirname, join } from 'path';
import { fileURLToPath } from 'url';
import { defineConfig, devices } from '@playwright/test';
import { config } from 'dotenv';

// Load test environment variables globally
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
config({ path: join(__dirname, 'tests-js', 'playwright', '.env.test') });

/**
 * @see https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  testDir: './tests-js/playwright',
  testMatch: '**/*.spec.mjs',
  /* Run tests in files in parallel */
  fullyParallel: false, // Sequential execution for server tests
  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,
  /* Retry on CI only */
  retries: process.env.CI ? 2 : 0,
  /* Opt out of parallel tests on CI. */
  workers: 1, // Single worker to avoid port conflicts
  /* Reporter to use. See https://playwright.dev/docs/test-reporters */
  reporter: process.env.CI
    ? [
        ['github'], // GitHub Actions reporter for CI
        ['html', { open: 'never' }], // HTML report without auto-opening
        ['junit', { outputFile: 'test-results/junit.xml' }], // JUnit for test results
      ]
    : 'list', // Use list reporter locally
  /* Global timeout for each test */
  timeout: process.env.PLAYWRIGHT_TIMEOUT ? Number.parseInt(process.env.PLAYWRIGHT_TIMEOUT) : 10000, // Configurable timeout
  /* Shared settings for all the projects below. See https://playwright.dev/docs/api/class-testoptions. */
  use: {
    /* Base URL to use in actions like `await page.goto('/')`. */
    // baseURL will be set dynamically by tests based on server configuration

    /* Collect trace when retrying the failed test. See https://playwright.dev/docs/trace-viewer */
    trace: 'on-first-retry',

    /* Take screenshot on failure */
    screenshot: 'only-on-failure',

    /* Record video on failure */
    video: 'retain-on-failure',

    /* Navigation timeout */
    navigationTimeout: 10000, // Consistent with global timeout

    /* Action timeout */
    actionTimeout: 10000, // Consistent with global timeout

    /* Wait for load state */
    waitForLoadState: 'domcontentloaded',

    /* Default wait after navigation for SPA stability */
    extraHTTPHeaders: {
      Accept: 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8',
    },
  },

  /* Configure projects for major browsers */
  projects: [
    {
      name: 'chromium',
      use: {
        ...devices['Desktop Chrome'],
        // Use headless mode in CI or when explicitly set
        headless: !!process.env.CI || process.env.PLAYWRIGHT_HEADLESS === 'true',
      },
    },

    // Disable WebKit for now due to bus errors on this system
    // {
    //   name: 'webkit',
    //   use: { ...devices['Desktop Safari'] },
    // },

    /* Test against mobile viewports. */
    // {
    //   name: 'Mobile Chrome',
    //   use: { ...devices['Pixel 5'] },
    // },
    // {
    //   name: 'Mobile Safari',
    //   use: { ...devices['iPhone 12'] },
    // },

    /* Test against branded browsers. */
    // {
    //   name: 'Microsoft Edge',
    //   use: { ...devices['Desktop Edge'], channel: 'msedge' },
    // },
    // {
    //   name: 'Google Chrome',
    //   use: { ...devices['Desktop Chrome'], channel: 'chrome' },
    // },
  ],

  /* Run your local dev server before starting the tests */
  // webServer: {
  //   command: 'npm run start',
  //   url: 'http://127.0.0.1:3000',
  //   reuseExistingServer: !process.env.CI,
  // },
});
