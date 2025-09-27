import { Button } from '@/components/ui/button';
import { useLogoutHandler } from '@/hooks/useLogoutHandler';
import { createWrapper } from '@/tests/wrapper';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { mockLogout, mockLogoutError } from '@/test-utils/msw-v2/handlers/auth';
import React from 'react';
import { afterAll, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

setupMswV2();

// Simple component that uses the useLogoutHandler hook
const LogoutButton: React.FC<{ onSuccess?: (response: any) => void; onError?: (message: string) => void }> = ({
  onSuccess,
  onError,
}) => {
  const { logout, isLoading: isLoggingOut } = useLogoutHandler({ onSuccess, onError });
  return (
    <Button onClick={() => logout()} disabled={isLoggingOut}>
      {isLoggingOut ? 'Logging out...' : 'Log Out'}
    </Button>
  );
};

describe('useLogoutHandler', () => {
  it('calls onSuccess callback when logout succeeds', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

    server.use(...mockLogout({ location: 'http://localhost:1135/ui/login', delay: 100 }));

    render(<LogoutButton onSuccess={mockOnSuccess} onError={mockOnError} />, { wrapper: createWrapper() });

    const logoutButton = screen.getByRole('button', { name: 'Log Out' });
    expect(logoutButton).toBeInTheDocument();

    await userEvent.click(logoutButton);

    expect(screen.getByRole('button', { name: 'Logging out...' })).toBeInTheDocument();
    expect(logoutButton).toBeDisabled();

    await waitFor(() => {
      expect(mockOnSuccess).toHaveBeenCalledWith(
        expect.objectContaining({
          status: 200,
          data: expect.objectContaining({ location: 'http://localhost:1135/ui/login' }),
        })
      );
      expect(screen.getByRole('button', { name: 'Log Out' })).toBeInTheDocument();
    });

    expect(logoutButton).not.toBeDisabled();
    expect(mockOnError).not.toHaveBeenCalled();
  });

  it('calls onError callback when logout fails', async () => {
    const mockOnSuccess = vi.fn();
    const mockOnError = vi.fn();

    server.use(...mockLogoutError({ status: 500, message: 'Internal Server Error' }));

    render(<LogoutButton onSuccess={mockOnSuccess} onError={mockOnError} />, { wrapper: createWrapper() });

    const logoutButton = screen.getByRole('button', { name: 'Log Out' });
    await userEvent.click(logoutButton);

    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Log Out' })).toBeInTheDocument();
    });

    expect(logoutButton).not.toBeDisabled();
    expect(mockOnSuccess).not.toHaveBeenCalled();
    expect(mockOnError).toHaveBeenCalledWith('Internal Server Error');
  });

  it('handles logout without callbacks', async () => {
    server.use(...mockLogout({ location: 'http://localhost:1135/ui/login' }));

    render(<LogoutButton />, { wrapper: createWrapper() });

    const logoutButton = screen.getByRole('button', { name: 'Log Out' });
    await userEvent.click(logoutButton);

    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Log Out' })).toBeInTheDocument();
    });

    expect(logoutButton).not.toBeDisabled();
  });
});
