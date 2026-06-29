import { TokenForm, toCreateTokenRequest } from '@/routes/tokens/-components/TokenForm';
import { TokenCreated } from '@bodhiapp/ts-client';
import { showSuccessParams } from '@/lib/utils.test';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { setupMswV2, server } from '@/test-utils/msw-v2/setup';
import { mockCreateToken, mockCreateTokenError } from '@/test-utils/msw-v2/handlers/tokens';
import { mockModelsDefault } from '@/test-utils/msw-v2/handlers/models';
import { mockListMcps } from '@/test-utils/msw-v2/handlers/mcps';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { describe, expect, it, vi } from 'vitest';

const mockToken: TokenCreated = {
  token: 'test-token-123',
};

const mockToast = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast }),
}));

setupMswV2();

afterEach(() => {
  mockToast.mockClear();
});

describe('toCreateTokenRequest', () => {
  it('builds all-access grants', () => {
    expect(
      toCreateTokenRequest({
        name: 'x',
        scope: 'scope_token_user',
        listModels: true,
        modelMode: 'all',
        models: [],
        listMcps: false,
        mcpMode: 'all',
        mcps: [],
      })
    ).toEqual({
      name: 'x',
      scope: 'scope_token_user',
      grants: { version: '1', list_models: true, models: { type: 'all' }, list_mcps: false, mcps: { type: 'all' } },
    });
  });

  it('builds specific models + empty mcps (and drops empty name)', () => {
    expect(
      toCreateTokenRequest({
        name: '',
        scope: 'scope_token_user',
        listModels: false,
        modelMode: 'specific',
        models: ['a', 'b'],
        listMcps: false,
        mcpMode: 'specific',
        mcps: [],
      })
    ).toEqual({
      name: undefined,
      scope: 'scope_token_user',
      grants: {
        version: '1',
        list_models: false,
        models: { type: 'specific', ids: ['a', 'b'] },
        list_mcps: false,
        mcps: { type: 'specific', ids: [] },
      },
    });
  });

  it('builds specific mcps', () => {
    const req = toCreateTokenRequest({
      name: 'k',
      scope: 'scope_token_power_user',
      listModels: false,
      modelMode: 'all',
      models: [],
      listMcps: true,
      mcpMode: 'specific',
      mcps: ['mcp-1'],
    });
    expect(req.grants).toEqual({
      version: '1',
      list_models: false,
      models: { type: 'all' },
      list_mcps: true,
      mcps: { type: 'specific', ids: ['mcp-1'] },
    });
  });
});

describe('TokenForm', () => {
  beforeEach(() => {
    server.use(
      ...mockCreateToken({ token: mockToken.token }),
      ...mockUserLoggedIn({}, { stub: true }),
      ...mockModelsDefault(),
      mockListMcps()
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
    });

    await user.type(screen.getByLabelText('Token Name (Optional)'), 'Test Token');
    await user.click(screen.getByRole('button', { name: 'Generate Token' }));

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'API token successfully generated'));
    });

    expect(onTokenCreated).toHaveBeenCalledWith(mockToken);
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
      expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'API token successfully generated'));
      expect(onTokenCreated).toHaveBeenCalledWith(mockToken);
    });
  });
});

describe('TokenDialog', () => {
  it('disables form during submission', async () => {
    const user = userEvent.setup();
    const onTokenCreated = vi.fn();
    server.use(...mockCreateToken({ token: 'test-token-123' }, { delayMs: 100 }));

    await act(async () => {
      render(<TokenForm onTokenCreated={onTokenCreated} />, {
        wrapper: createWrapper(),
      });
    });

    const submitButton = screen.getByRole('button', { name: 'Generate Token' });
    const input = screen.getByLabelText('Token Name (Optional)');

    await user.click(submitButton);

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
      ...mockCreateTokenError({
        message: 'Failed to generate token. Please try again.',
        type: 'invalid_request_error',
      })
    );
    render(<TokenForm onTokenCreated={onTokenCreated} />, {
      wrapper: createWrapper(),
    });

    await user.click(screen.getByRole('button', { name: 'Generate Token' }));

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
});
