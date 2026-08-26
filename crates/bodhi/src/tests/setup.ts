import 'fake-indexeddb/auto';
import '@testing-library/jest-dom';
import { notifyManager } from '@tanstack/react-query';
import { beforeAll, afterAll, vi } from 'vitest';

import apiClient from '@/lib/apiClient';

// TanStack Query v5: queueMicrotask scheduler keeps updates inside act() without full sync's infinite-loop risk.
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

vi.mock('@/hooks/useMediaQuery', () => ({
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  useMediaQuery: (_query: string) => {
    return true;
  },
}));

class MockResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}

global.ResizeObserver = MockResizeObserver;

// Pointer Events polyfill for Radix UI
Element.prototype.hasPointerCapture = vi.fn(() => false);
Element.prototype.setPointerCapture = vi.fn();
Element.prototype.releasePointerCapture = vi.fn();

// scrollIntoView polyfill for Radix UI
Element.prototype.scrollIntoView = vi.fn();

const originalError = console.error;
beforeAll(() => {
  apiClient.defaults.baseURL = 'http://localhost:3000';
  console.error = (...args) => {
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
