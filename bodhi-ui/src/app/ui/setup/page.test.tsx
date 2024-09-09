'use client';

import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, vi, expect, beforeEach } from 'vitest';
import Setup from './page';
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
  // eslint-disable-next-line @next/next/no-img-element
  default: () => <img alt="mocked image" />,
}));

describe('Setup Page', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    pushMock.mockClear();
  });

  it('renders the setup page when status is setup', async () => {
    vi.mocked(BodhiBackend.prototype.getAppInfo).mockResolvedValueOnce({
      status: 'setup',
    });

    render(<Setup />);

    await waitFor(() => {
      expect(screen.getByText('Bodhi App Setup')).toBeInTheDocument();
    });
  });

  it('redirects to /ui/home when status is ready', async () => {
    vi.mocked(BodhiBackend.prototype.getAppInfo).mockResolvedValueOnce({
      status: 'ready',
    });

    render(<Setup />);

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/home');
    });
  });

  it('redirects to /ui/setup/resource-admin when status is resource-admin', async () => {
    vi.mocked(BodhiBackend.prototype.getAppInfo).mockResolvedValueOnce({
      status: 'resource-admin',
    });

    render(<Setup />);

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup/resource-admin');
    });
  });

  it('sets up authenticated instance and redirects to /ui/home', async () => {
    vi.mocked(BodhiBackend.prototype.getAppInfo).mockResolvedValue({
      status: 'setup',
    });
    vi.mocked(BodhiBackend.prototype.setupApp).mockResolvedValueOnce({
      status: 'ready',
    });

    render(<Setup />);

    const authButton = await screen.findByText(
      'Setup Authenticated Instance →'
    );
    fireEvent.click(authButton);

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/home');
    });
  });

  it('sets up unauthenticated instance and redirects to /ui/setup/resource-admin', async () => {
    vi.mocked(BodhiBackend.prototype.getAppInfo).mockResolvedValue({
      status: 'setup',
    });
    vi.mocked(BodhiBackend.prototype.setupApp).mockResolvedValueOnce({
      status: 'resource-admin',
    });

    render(<Setup />);

    const unauthButton = await screen.findByText(
      'Setup Unauthenticated Instance →'
    );
    fireEvent.click(unauthButton);

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup/resource-admin');
    });
  });

  it('displays error message when setup fails', async () => {
    vi.mocked(BodhiBackend.prototype.getAppInfo).mockResolvedValue({
      status: 'setup',
    });
    vi.mocked(BodhiBackend.prototype.setupApp).mockRejectedValueOnce(
      new Error('Setup failed')
    );

    render(<Setup />);

    const authButton = await screen.findByText(
      'Setup Authenticated Instance →'
    );
    fireEvent.click(authButton);

    await waitFor(() => {
      expect(
        screen.getByText('An unexpected error occurred')
      ).toBeInTheDocument();
    });
  });
});
