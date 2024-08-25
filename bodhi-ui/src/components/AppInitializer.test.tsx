'use client'

import { render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, it, vi, expect } from 'vitest';
import AppInitializer from './AppInitializer';

// Mock Next.js navigation
const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

// Mock fetch
global.fetch = vi.fn();

describe('AppInitializer', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear(); // Clear the pushMock between tests
  });

  it('displays error message when API call fails', async () => {
    (global.fetch as unknown as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error('API Error'));

    render(<AppInitializer />);

    await waitFor(() => {
      expect(screen.getByText(/Unable to connect to backend: 'http:\/\/localhost:1135'/)).toBeInTheDocument();
    });
  });

  it('redirects to /ui/setup when status is setup', async () => {
    (global.fetch as unknown as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve({ status: 'setup' }),
    });

    render(<AppInitializer />);

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup');
    });
  });

  it('redirects to /ui/home when status is ready', async () => {
    (global.fetch as unknown as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve({ status: 'ready' }),
    });

    render(<AppInitializer />);

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/home');
    });
  });
});