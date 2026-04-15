import { test as base } from '@playwright/test';
import { resetDatabase } from '@/test-helpers.mjs';
import { getServerUrl } from '@/utils/db-config.mjs';

// Create extended test with project-aware fixtures
export const test = base.extend({
  // Fixture providing project-aware shared server URL
  sharedServerUrl: [
    async ({}, use, testInfo) => {
      const url = getServerUrl(testInfo.project.name);
      await use(url);
    },
    { scope: 'test' },
  ],

  // Auto-fixture that resets DB before each test (project-aware)
  autoResetDb: [
    async ({}, use, testInfo) => {
      const url = getServerUrl(testInfo.project.name);
      await resetDatabase(url);
      await use();
    },
    { auto: true, scope: 'test' },
  ],
});

export { expect } from '@playwright/test';
