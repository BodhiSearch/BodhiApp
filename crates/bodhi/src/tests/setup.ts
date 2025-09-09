import { vi } from 'vitest';

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

// Mock framer-motion to avoid animation and browser API issues in tests
vi.mock('framer-motion', async () => {
  const React = await import('react');
  return {
    motion: {
      div: ({ children, ...props }: any) => {
        // eslint-disable-line @typescript-eslint/no-explicit-any
        // Filter out framer-motion specific props to avoid React warnings
        const {
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          animate,
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          initial,
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          exit,
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          variants,
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          transition,
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          whileHover,
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          whileTap,
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          whileFocus,
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          whileInView,
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          drag,
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          dragConstraints,
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          dragElastic,
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          dragMomentum,
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          dragTransition,
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          onDrag,
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          onDragStart,
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          onDragEnd,
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          layout,
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
          layoutId,
          ...filteredProps
        } = props;
        return React.createElement('div', filteredProps, children);
      },
    },
    AnimatePresence: ({ children }: { children?: React.ReactNode }) =>
      React.createElement(React.Fragment, null, children),
    useAnimation: () => ({}),
  };
});

vi.mock('@/hooks/use-media-query', () => ({
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  useMediaQuery: (query: string) => {
    return true;
  },
}));

import '@testing-library/jest-dom';
import { beforeAll, afterAll } from 'vitest';
import apiClient from '@/lib/apiClient';

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
