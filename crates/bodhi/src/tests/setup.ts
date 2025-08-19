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
vi.mock('framer-motion', () => {
  const React = require('react');
  return {
    motion: {
      div: ({ children, ...props }: any) => {
        // Filter out framer-motion specific props to avoid React warnings
        const {
          animate,
          initial,
          exit,
          variants,
          transition,
          whileHover,
          whileTap,
          whileFocus,
          whileInView,
          drag,
          dragConstraints,
          dragElastic,
          dragMomentum,
          dragTransition,
          onDrag,
          onDragStart,
          onDragEnd,
          layout,
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
