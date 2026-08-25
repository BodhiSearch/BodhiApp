import { test as base } from '@playwright/test';
import { resetDatabase } from '@/test-helpers.mjs';
import { getServerUrl } from '@/utils/db-config.mjs';

export const test = base.extend({
  sharedServerUrl: [
    async ({}, use, testInfo) => {
      const url = getServerUrl(testInfo.project.name);
      await use(url);
    },
    { scope: 'test' },
  ],

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
