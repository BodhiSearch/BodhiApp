import { TokenForm } from '@/app/ui/tokens/TokenForm';
import { CREATE_TOKEN_ENDPOINT, TokenResponse } from '@/hooks/useCreateToken';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { describe, expect, it, vi } from 'vitest';

const mockToken: TokenResponse = {
  offline_token: 'test-token-123',
  name: 'Test Token',
  status: 'active',
  last_used: null,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

const server = setupServer();

const mockToast = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast }),
}));

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => {
  server.resetHandlers();
  mockToast.mockClear();
});

describe('TokenForm', () => {
  beforeEach(() => {
    server.use(
      rest.post(`*${CREATE_TOKEN_ENDPOINT}`, (_, res, ctx) => {
        return res(ctx.status(201), ctx.json(mockToken));
      })
    );
  });

  it('renders form fields correctly', () => {
    const onTokenCreated = vi.fn();
    render(<TokenForm onTokenCreated={onTokenCreated} />, {
      wrapper: createWrapper(),
    });

    expect(screen.getByLabelText('Token Name (Optional)')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Generate Token' })).toBeInTheDocument();
  });

  it('handles form submission with name', async () => {
    const user = userEvent.setup();
    const onTokenCreated = vi.fn();

    await act(async () => {
      render(<TokenForm onTokenCreated={onTokenCreated} />, {
        wrapper: createWrapper(),
      });
    })

    await user.type(screen.getByLabelText('Token Name (Optional)'), 'Test Token');
    await user.click(screen.getByRole('button', { name: 'Generate Token' }));

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith({
        title: 'Success',
        description: 'API token successfully generated',
        variant: 'default',
        duration: 5000,
      });
    });

    // Check if callback was called with token
    expect(onTokenCreated).toHaveBeenCalledWith(mockToken);

    // Check if form was reset
    expect(screen.getByLabelText('Token Name (Optional)')).toHaveValue('');
  });

  it('handles form submission without name', async () => {
    const user = userEvent.setup();
    const onTokenCreated = vi.fn();

    render(<TokenForm onTokenCreated={onTokenCreated} />, {
      wrapper: createWrapper(),
    });

    await user.click(screen.getByRole('button', { name: 'Generate Token' }));

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith({
        title: 'Success',
        description: 'API token successfully generated',
        variant: 'default',
        duration: 5000,
      });
      expect(onTokenCreated).toHaveBeenCalledWith(mockToken);
    });
  });
});

describe('TokenDialog', () => {
  it('disables form during submission', async () => {
    const user = userEvent.setup();
    const onTokenCreated = vi.fn();
    server.use(
      rest.post(`*${CREATE_TOKEN_ENDPOINT}`, (_, res, ctx) => {
        return res(ctx.delay(100), ctx.status(201), ctx.json({ message: 'Failed to generate token. Please try again.' }));
      })
    );

    await act(async () => {
      render(<TokenForm onTokenCreated={onTokenCreated} />, {
        wrapper: createWrapper(),
      });
    })

    const submitButton = screen.getByRole('button', { name: 'Generate Token' });
    const input = screen.getByLabelText('Token Name (Optional)');

    await user.click(submitButton);

    // Check if form elements are disabled during submission
    expect(submitButton).toBeDisabled();
    expect(input).toBeDisabled();
    expect(screen.getByText('Generating...')).toBeInTheDocument();
  });
});

describe('TokenDialog', () => {
  it('handles api error', async () => {
    const user = userEvent.setup();
    const onTokenCreated = vi.fn();
    server.use(
      rest.post(`*${CREATE_TOKEN_ENDPOINT}`, (_, res, ctx) => {
        return res(ctx.status(400), ctx.json({ message: 'Failed to generate token. Please try again.' }));
      })
    );
    render(<TokenForm onTokenCreated={onTokenCreated} />, {
      wrapper: createWrapper(),
    });

    await user.click(screen.getByRole('button', { name: 'Generate Token' }));

    // Wait for error toast and console error
    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith({
        title: 'Error',
        description: 'Failed to generate token. Please try again.',
        variant: 'destructive',
        duration: 5000,
      });
    });
    expect(onTokenCreated).not.toHaveBeenCalled();
  });
})