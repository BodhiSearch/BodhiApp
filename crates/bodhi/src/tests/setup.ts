import { config } from 'dotenv';
import { resolve } from 'path';
import '@testing-library/jest-dom';
import { vi, beforeAll, afterEach, afterAll } from 'vitest';

// Mock ResizeObserver
class MockResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}

global.ResizeObserver = MockResizeObserver;

// Mock window.matchMedia
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation((query) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

// Suppress console errors for specific messages
const originalError = console.error;
beforeAll(() => {
  console.error = (...args) => {
    // Check if any of the arguments contain our expected error messages
    const errorString = args
      .map((arg) =>
        typeof arg === 'string'
          ? arg
          : arg instanceof Error
            ? arg.message
            : arg?.toString?.()
      )
      .join(' ');

    if (errorString.includes('Request failed with status code 500')) {
      return;
    }
    originalError.call(console, ...args);
  };
});

afterAll(() => {
  console.error = originalError;
});

config({ path: resolve(__dirname, '../.env.test') });
