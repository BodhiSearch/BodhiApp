import path from 'path';

import react from '@vitejs/plugin-react';
import { defineConfig } from 'vitest/config';
// @ts-expect-error path is not typed in vitest config

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/tests/setup.ts'],
    include: ['src/**/*.{test,spec}.{js,jsx,ts,tsx}'],
    alias: {
      '@': path.resolve(__dirname, './src'),
      'framer-motion': path.resolve(__dirname, './src/tests/mocks/framer-motion.tsx'),
    },
    deps: {
      optimizer: {
        web: {
          include: ['@testing-library/jest-dom'],
        },
      },
    },
  },
});
