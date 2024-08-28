'use client'

import { render, screen, waitFor } from '@testing-library/react';
import { describe, it, vi, expect, beforeEach } from 'vitest';
import ResourceAdminPage from './page';
import { BodhiBackend } from '@/services/BodhiBackend';

// Mock the BodhiBackend
vi.mock('@/services/BodhiBackend');

// Mock the router
const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  useSearchParams: () => ({
    get: () => null,
  }),
}));

// Mock the Image component
vi.mock('next/image', () => ({
  default: () => <img alt="mocked image" />,
}));

describe('ResourceAdminPage', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
  });

  it('renders the resource admin page when status is resource-admin', async () => {
    vi.mocked(BodhiBackend.prototype.getAppInfo).mockResolvedValueOnce({ status: 'resource-admin' });

    render(<ResourceAdminPage />);

    await waitFor(() => {
      expect(screen.getByText('Resource Admin Setup')).toBeInTheDocument();
      expect(screen.getByText('Log In')).toBeInTheDocument();
    });
  });

  it('redirects to /ui/setup when status is setup', async () => {
    vi.mocked(BodhiBackend.prototype.getAppInfo).mockResolvedValueOnce({ status: 'setup' });

    render(<ResourceAdminPage />);

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup');
    });
  });

  it('redirects to /ui/home when status is ready', async () => {
    vi.mocked(BodhiBackend.prototype.getAppInfo).mockResolvedValueOnce({ status: 'ready' });

    render(<ResourceAdminPage />);

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/home');
    });
  });
});
