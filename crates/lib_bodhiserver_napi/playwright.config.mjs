import { dirname, join } from 'path';
import { fileURLToPath } from 'url';
import { defineConfig, devices } from '@playwright/test';
import { config } from 'dotenv';

// Load test environment variables globally
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
config({ path: join(__dirname, 'tests-js', '.env.test') });

const testTimeout = 120000;
const navigationTimeout = 30000;
const actionTimeout = 30000;

// Check if running scheduled tests via --grep flag
const isScheduledRun = process.argv.includes('--grep') && process.argv.some(arg => arg.includes('@scheduled'));

// Dual-DB support: SQLite (port 51135) and PostgreSQL (port 51136)
// PostgreSQL requires docker-compose-test-deps.yml containers running

/**
 * @see https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  testDir: './tests-js',
  testMatch: '**/*.spec.mjs',
  /* Exclude scheduled tests from regular runs, unless explicitly running with --grep @scheduled */
  ...(isScheduledRun ? {} : { grepInvert: /@scheduled/ }),
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
  timeout: process.env.PLAYWRIGHT_TIMEOUT ? Number.parseInt(process.env.PLAYWRIGHT_TIMEOUT) : testTimeout, // Configurable timeout
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
    navigationTimeout,

    /* Action timeout */
    actionTimeout,

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
      name: 'sqlite',
      use: {
        ...devices['Desktop Chrome'],
        // Use headless mode in CI or when explicitly set
        headless: !!process.env.CI || process.env.HEADLESS === 'true' || process.env.PLAYWRIGHT_HEADLESS === 'true',
      },
    },
    {
      name: 'postgres',
      use: {
        ...devices['Desktop Chrome'],
        headless: !!process.env.CI || process.env.HEADLESS === 'true' || process.env.PLAYWRIGHT_HEADLESS === 'true',
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
  webServer: [
    {
      command: 'node tests-js/scripts/start-shared-server.mjs --port 51135 --db-type sqlite',
      url: 'http://localhost:51135/ping',
      reuseExistingServer: false,  // Always start fresh
      timeout: 60000,
    },
    {
      command: 'node tests-js/scripts/start-shared-server.mjs --port 41135 --db-type postgres',
      url: 'http://localhost:41135/ping',
      reuseExistingServer: false,
      timeout: 60000,
    },
    {
      command: 'cd test-oauth-app && npm run build && npx serve dist -s -l 55173',
      url: 'http://localhost:55173/',
      reuseExistingServer: false,
      timeout: 30000,
    },
    {
      command: 'cd test-mcp-oauth-server && npm run build && npm start',
      url: 'http://localhost:55174/ping',
      reuseExistingServer: false,
      timeout: 30000,
    },
    {
      command: 'cd test-mcp-oauth-server && npm run build && node dist/index.js --dcr',
      url: 'http://localhost:55175/ping',
      reuseExistingServer: false,
      timeout: 30000,
      env: {
        ...process.env,
        TEST_MCP_OAUTH_PORT: '55175',
      },
    },
  ],
});
