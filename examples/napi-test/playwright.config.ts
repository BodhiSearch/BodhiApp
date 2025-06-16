import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for BodhiApp UI testing
 * 
 * This configuration is optimized for testing the BodhiApp server
 * launched via NAPI-RS bindings.
 */
export default defineConfig({
  testDir: './src',
  testMatch: '**/ui-test.ts',
  
  /* Run tests in files in parallel */
  fullyParallel: false, // Disable parallel execution for server tests
  
  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,
  
  /* Retry on CI only */
  retries: process.env.CI ? 2 : 0,
  
  /* Opt out of parallel tests on CI. */
  workers: 1, // Single worker to avoid port conflicts
  
  /* Reporter to use. See https://playwright.dev/docs/test-reporters */
  reporter: [
    ['html', { outputFolder: 'test-results/html-report' }],
    ['list']
  ],
  
  /* Shared settings for all the projects below. See https://playwright.dev/docs/api/class-testoptions. */
  use: {
    /* Base URL to use in actions like `await page.goto('/')`. */
    baseURL: 'http://127.0.0.1:1135',
    
    /* Collect trace when retrying the failed test. See https://playwright.dev/docs/trace-viewer */
    trace: 'on-first-retry',
    
    /* Take screenshot on failure */
    screenshot: 'only-on-failure',
    
    /* Record video on failure */
    video: 'retain-on-failure',
  },

  /* Configure projects for major browsers */
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    
    // Uncomment to test on other browsers
    // {
    //   name: 'firefox',
    //   use: { ...devices['Desktop Firefox'] },
    // },
    // {
    //   name: 'webkit',
    //   use: { ...devices['Desktop Safari'] },
    // },
  ],

  /* Output directory for test artifacts */
  outputDir: 'test-results/artifacts',
  
  /* Global setup and teardown */
  globalSetup: undefined,
  globalTeardown: undefined,
  
  /* Test timeout */
  timeout: 30000, // 30 seconds per test
  expect: {
    timeout: 5000, // 5 seconds for assertions
  },
});
