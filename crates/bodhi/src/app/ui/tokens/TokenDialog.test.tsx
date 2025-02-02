import { TokenDialog } from '@/app/ui/tokens/TokenDialog';
import { TokenResponse } from '@/hooks/useApiTokens';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

describe('TokenDialog', () => {
  const mockToken: TokenResponse = {
    offline_token: 'test-token-123',
  };

  let user: ReturnType<typeof userEvent.setup>;

  beforeEach(() => {
    user = userEvent.setup();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it('renders dialog with masked token', () => {
    const onClose = vi.fn();
    render(<TokenDialog token={mockToken} open={true} onClose={onClose} />, {
      wrapper: createWrapper(),
    });

    expect(screen.getByText('API Token Generated')).toBeInTheDocument();
    expect(screen.getByText('•'.repeat(40))).toBeInTheDocument();
    expect(screen.queryByText('test-token-123')).not.toBeInTheDocument();
  });

  it('toggles token visibility', async () => {
    const onClose = vi.fn();

    render(<TokenDialog token={mockToken} open={true} onClose={onClose} />, {
      wrapper: createWrapper(),
    });

    // Initially token is masked
    expect(screen.getByText('•'.repeat(40))).toBeInTheDocument();
    expect(screen.queryByText('test-token-123')).not.toBeInTheDocument();

    // Show token
    await user.click(screen.getByTestId('toggle-show-content'));
    expect(screen.getByText('test-token-123')).toBeInTheDocument();
    expect(screen.queryByText('•'.repeat(40))).not.toBeInTheDocument();

    // Hide token again
    await user.click(screen.getByTestId('toggle-show-content'));
    expect(screen.getByText('•'.repeat(40))).toBeInTheDocument();
    expect(screen.queryByText('test-token-123')).not.toBeInTheDocument();
  });

  it('copies token to clipboard', async () => {
    const onClose = vi.fn();

    await act(async () => {
      render(<TokenDialog token={mockToken} open={true} onClose={onClose} />, {
        wrapper: createWrapper(),
      });
    });

    // Click copy button
    await user.click(screen.getByTestId('copy-content'));

    // Verify clipboard content
    expect(await navigator.clipboard.readText()).toBe('test-token-123');

    // Check copy confirmation
    expect(screen.getByTestId('copied-content')).toBeInTheDocument();
  });

  it('closes dialog', async () => {
    const onClose = vi.fn();

    render(<TokenDialog token={mockToken} open={true} onClose={onClose} />, {
      wrapper: createWrapper(),
    });

    await user.click(screen.getByRole('button', { name: 'Done' }));
    expect(onClose).toHaveBeenCalled();
  });

  it('shows security warnings', () => {
    const onClose = vi.fn();
    render(<TokenDialog token={mockToken} open={true} onClose={onClose} />, {
      wrapper: createWrapper(),
    });

    expect(screen.getByText(/Copy your API token now/)).toBeInTheDocument();
    expect(screen.getByText(/Make sure to copy your token now/)).toBeInTheDocument();
    expect(screen.getByText(/For security reasons/)).toBeInTheDocument();
  });
});
