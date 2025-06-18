import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    environment: 'node',
    testTimeout: 30000,
    hookTimeout: 10000,
    teardownTimeout: 10000,
    include: ['tests-js/**/*.test.js', 'tests-js/**/*.spec.js'],
    exclude: [
      'node_modules/**',
      'tests-js/playwright/**/*.spec.js',
      'tests-js/playwright/**/*.test.js',
    ],
    reporter: ['verbose'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: ['node_modules/**', 'tests-js/**', '**/*.config.*'],
    },
    pool: 'threads',
    poolOptions: {
      threads: {
        singleThread: true, // Important for server tests that use the same ports
      },
    },
    silent: false,
    bail: 1,
  },
});
