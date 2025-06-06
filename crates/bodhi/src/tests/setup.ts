import '@testing-library/jest-dom';
import { vi, beforeAll, afterAll } from 'vitest';

// Mock ResizeObserver
class MockResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}

global.ResizeObserver = MockResizeObserver;

// Mock React Router navigation functions
const mockNavigate = vi.fn();
const mockLocation = { pathname: '/', search: '', hash: '', state: null, key: 'default' };

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => mockNavigate,
    useLocation: () => mockLocation,
    useSearchParams: () => [new URLSearchParams(), vi.fn()],
  };
});

// Export mocks for use in tests
export { mockNavigate, mockLocation };

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

    if (
      errorString.includes('Request failed with status code ') ||
      errorString.includes('Network Error')
    ) {
      return;
    }
    originalError.call(console, ...args);
  };
});

afterAll(() => {
  console.error = originalError;
});
