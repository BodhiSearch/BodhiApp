import '@testing-library/jest-dom';
import { notifyManager } from '@tanstack/react-query';
import { beforeAll, afterAll, vi } from 'vitest';

import apiClient from '@/lib/apiClient';

// TanStack Query v5 uses setTimeout(cb, 0) to schedule state notifications by default.
// This causes mutation/query state updates to be deferred outside of act() blocks in tests.
// Using queueMicrotask ensures state transitions are flushed within act() blocks
// (act flushes microtasks) while avoiding infinite re-render loops that happen with
// fully synchronous scheduling.
notifyManager.setScheduler((cb) => queueMicrotask(cb));

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

vi.mock('@/hooks/use-media-query', () => ({
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  useMediaQuery: (_query: string) => {
    return true;
  },
}));

// Mock ResizeObserver
class MockResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}

global.ResizeObserver = MockResizeObserver;

// Polyfill for Pointer Events API (not supported in JSDOM)
// Required for Radix UI Select and other pointer-based components
Element.prototype.hasPointerCapture = vi.fn(() => false);
Element.prototype.setPointerCapture = vi.fn();
Element.prototype.releasePointerCapture = vi.fn();

// Polyfill for scrollIntoView (not supported in JSDOM)
// Required for Radix UI Select and other scroll-based components
Element.prototype.scrollIntoView = vi.fn();

// Suppress console errors for specific messages
const originalError = console.error;
beforeAll(() => {
  apiClient.defaults.baseURL = 'http://localhost:3000';
  console.error = (...args) => {
    // Check if any of the arguments contain our expected error messages
    const errorString = args
      .map((arg) => (typeof arg === 'string' ? arg : arg instanceof Error ? arg.message : arg?.toString?.()))
      .join(' ');

    if (errorString.includes('Request failed with status code ') || errorString.includes('Network Error')) {
      return;
    }
    originalError.call(console, ...args);
  };
});

afterAll(() => {
  console.error = originalError;
});
