import { test as base } from '@playwright/test';
import { resetDatabase } from '@/test-helpers.mjs';

// Create extended test with auto-reset fixture
export const test = base.extend({
  // Auto-fixture that resets DB before each test
  autoResetDb: [
    async ({}, use) => {
      await resetDatabase('http://localhost:51135');
      await use();
    },
    { auto: true, scope: 'test' },
  ],
});

export { expect } from '@playwright/test';
