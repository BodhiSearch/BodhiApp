'use client';

import { render, screen, waitFor } from '@testing-library/react';
import { describe, it, vi, expect, beforeEach } from 'vitest';
import UiPage from './page';
import { BodhiBackend } from '@/services/BodhiBackend';

vi.mock('@/services/BodhiBackend');

const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  useSearchParams: () => ({
    get: () => null,
  }),
}));

describe('UiPage', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
  });

  it('redirects to /ui/setup when status is setup', async () => {
    vi.mocked(BodhiBackend.prototype.getAppInfo).mockResolvedValueOnce({
      status: 'setup',
    });

    render(<UiPage />);

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup');
    });
  });

  it('redirects to /ui/home when status is ready', async () => {
    vi.mocked(BodhiBackend.prototype.getAppInfo).mockResolvedValueOnce({
      status: 'ready',
    });

    render(<UiPage />);

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/home');
    });
  });

  it('redirects to /ui/setup/resource-admin when status is resource-admin', async () => {
    vi.mocked(BodhiBackend.prototype.getAppInfo).mockResolvedValueOnce({
      status: 'resource-admin',
    });

    render(<UiPage />);

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup/resource-admin');
    });
  });

  it('displays error message for unexpected status', async () => {
    vi.mocked(BodhiBackend.prototype.getAppInfo).mockResolvedValueOnce({
      status: 'unexpected',
    });

    render(<UiPage />);

    await waitFor(() => {
      expect(
        screen.getByText('Unexpected response from server')
      ).toBeInTheDocument();
    });
  });

  it('displays error message when API call fails', async () => {
    vi.mocked(BodhiBackend.prototype.getAppInfo).mockRejectedValueOnce(
      new Error('API Error')
    );

    render(<UiPage />);

    await waitFor(() => {
      expect(
        screen.getByText(/Unable to connect to backend/)
      ).toBeInTheDocument();
    });
  });
});
