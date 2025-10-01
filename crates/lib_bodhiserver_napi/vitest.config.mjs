import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    environment: 'node',
    testTimeout: 10000,
    hookTimeout: 10000,
    teardownTimeout: 10000,
    include: ['tests/**/*.test.js'],
    exclude: [
      'node_modules/**',
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
