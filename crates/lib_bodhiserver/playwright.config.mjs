import { dirname, join } from 'path';
import { fileURLToPath } from 'url';
import { defineConfig, devices } from '@playwright/test';
import { config } from 'dotenv';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
config({ path: join(__dirname, 'tests-js', '.env.test') });

const devProxyPort = process.env.BODHI_DEV_PROXY_UI_PORT || '3000';

const testTimeout = 120000;
const navigationTimeout = 30000;
const actionTimeout = 30000;

const isScheduledRun = process.argv.includes('--grep') && process.argv.some(arg => arg.includes('@scheduled'));

// Dual-project support: standalone (SQLite, port 51135) and multi_tenant (PostgreSQL, port 41135)
// multi_tenant requires docker/docker-compose.test.yml containers running

/**
 * @see https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  testDir: './tests-js',
  testMatch: '**/*.spec.mjs',
  /* Exclude scheduled tests from regular runs, unless explicitly running with --grep @scheduled */
  ...(isScheduledRun ? {} : { grepInvert: /@scheduled/ }),
  fullyParallel: false, // Sequential execution for server tests
  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,
  /* No test-level retries: external auth-server flakiness is absorbed at the POM
     action level (see LoginPage/SetupResourceAdminPage login retries). */
  retries: 0,
  workers: 1, // Single worker to avoid port conflicts
  reporter: process.env.CI
    ? [
      ['github'],
      ['html', { open: 'never' }],
      ['junit', { outputFile: 'test-results/junit.xml' }],
    ]
    : 'list',
  timeout: process.env.PLAYWRIGHT_TIMEOUT ? Number.parseInt(process.env.PLAYWRIGHT_TIMEOUT) : testTimeout,
  use: {
    // baseURL will be set dynamically by tests based on server configuration

    trace: 'on-first-retry',

    screenshot: 'only-on-failure',

    video: 'retain-on-failure',

    navigationTimeout,

    actionTimeout,

    waitForLoadState: 'domcontentloaded',

    extraHTTPHeaders: {
      Accept: 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8',
    },
  },

  projects: [
    {
      name: 'standalone',
      testIgnore: [
        '**/multi-tenant/**',               // Multi-tenant tests are multi_tenant-only
      ],
      use: {
        ...devices['Desktop Chrome'],
        headless: !!process.env.CI || process.env.HEADLESS === 'true' || process.env.PLAYWRIGHT_HEADLESS === 'true',
      },
    },
    {
      name: 'multi_tenant',
      testIgnore: [
        '**/setup/**',                    // Setup flow is standalone-only
        '**/models/**',                   // Local model alias + metadata require GGUF files
        '**/request-access/**',           // Uses createServerManager() — standalone-specific
        '**/chat/local-models.spec.mjs',  // Standalone-only GGUF testing
      ],
      use: {
        ...devices['Desktop Chrome'],
        headless: !!process.env.CI || process.env.HEADLESS === 'true' || process.env.PLAYWRIGHT_HEADLESS === 'true',
      },
    },
  ],

  /* Run your local dev server before starting the tests */
  webServer: [
    {
      command: 'npm run e2e:server:vite',
      url: `http://localhost:${devProxyPort}/ui/@vite/client`,
      reuseExistingServer: false,
      timeout: 180000,
    },
    {
      command: 'npm run e2e:server:standalone',
      url: 'http://localhost:51135/ping',
      reuseExistingServer: false,
      timeout: 60000,
    },
    {
      command: 'npm run e2e:server:multi_tenant',
      url: 'http://localhost:41135/ping',
      reuseExistingServer: false,
      timeout: 60000,
    },
    {
      command: 'npm run e2e:server:test-app-oauth',
      url: 'http://localhost:55173/',
      reuseExistingServer: false,
      timeout: 30000,
    },
    {
      command: 'npm run e2e:server:test-app-mcp',
      url: 'http://localhost:55174/ping',
      reuseExistingServer: false,
      timeout: 30000,
    },
    {
      command: 'npm run e2e:server:test-app-mcp-dcr',
      url: 'http://localhost:55175/ping',
      reuseExistingServer: false,
      timeout: 30000,
      env: {
        ...process.env,
        TEST_MCP_OAUTH_PORT: '55175',
      },
    },
    {
      command: 'cd test-mcp-auth-server && npm run build && node dist/index.js --header Authorization="Bearer test-header-key" --port 55176',
      url: 'http://localhost:55176/ping',
      timeout: 30 * 1000,
      reuseExistingServer: false,
    },
    {
      command: 'cd test-mcp-auth-server && npm run build && node dist/index.js --query api_key=test-query-key --port 55177',
      url: 'http://localhost:55177/ping',
      timeout: 30 * 1000,
      reuseExistingServer: false,
    },
    {
      command: 'cd test-mcp-auth-server && npm run build && node dist/index.js --header X-Auth-1=header-val-1 --header X-Auth-2=header-val-2 --query q_key_1=query-val-1 --query q_key_2=query-val-2 --port 55178',
      url: 'http://localhost:55178/ping',
      timeout: 30 * 1000,
      reuseExistingServer: false,
    },
    {
      command: 'npm run e2e:server:everything-mcp',
      url: 'http://localhost:55180/mcp',
      timeout: 30 * 1000,
      reuseExistingServer: false,
    },
    {
      command: 'npm run e2e:server:mcp-inspector',
      url: 'http://localhost:6277/health',
      timeout: 30 * 1000,
      reuseExistingServer: false,
    },
  ],
});
