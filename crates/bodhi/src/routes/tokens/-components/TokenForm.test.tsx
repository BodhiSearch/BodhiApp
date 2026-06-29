import { http, HttpResponse } from 'msw';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { TokenCreated } from '@bodhiapp/ts-client';

import { BODHI_API_BASE } from '@/hooks/useQuery';
import { showSuccessParams } from '@/lib/utils.test';
import { TokenForm, toCreateTokenRequest } from '@/routes/tokens/-components/TokenForm';
import { mockListMcps } from '@/test-utils/msw-v2/handlers/mcps';
import { mockModelsDefault } from '@/test-utils/msw-v2/handlers/models';
import { mockCreateToken, mockCreateTokenError } from '@/test-utils/msw-v2/handlers/tokens';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { setupMswV2, server } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

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
  const renderForm = (onTokenCreated = vi.fn()) =>
    render(<TokenForm onTokenCreated={onTokenCreated} onCancel={vi.fn()} />, { wrapper: createWrapper() });

  beforeEach(() => {
    server.use(
      ...mockCreateToken({ token: mockToken.token }),
      ...mockUserLoggedIn({}, { stub: true }),
      ...mockModelsDefault(),
      mockListMcps()
    );
  });

  it('renders the four sections and the footer actions', () => {
    renderForm();
    expect(screen.getByTestId('token-name-input')).toBeInTheDocument();
    expect(screen.getByText('Token Identity')).toBeInTheDocument();
    expect(screen.getByText('Model Access')).toBeInTheDocument();
    expect(screen.getByText('MCP Access')).toBeInTheDocument();
    expect(screen.getByText('Token Scope')).toBeInTheDocument();
    expect(screen.getByTestId('list-models-switch')).toBeInTheDocument();
    expect(screen.getByTestId('list-mcps-switch')).toBeInTheDocument();
    expect(screen.getByTestId('scope-card-scope_token_user')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /Generate Token/ })).toBeInTheDocument();
    expect(screen.getByTestId('cancel-token-button')).toBeInTheDocument();
  });

  it('invokes onCancel from the Cancel button', async () => {
    const user = userEvent.setup();
    const onCancel = vi.fn();
    render(<TokenForm onTokenCreated={vi.fn()} onCancel={onCancel} />, { wrapper: createWrapper() });
    await user.click(screen.getByTestId('cancel-token-button'));
    expect(onCancel).toHaveBeenCalled();
  });

  it('handles form submission with name', async () => {
    const user = userEvent.setup();
    const onTokenCreated = vi.fn();

    await act(async () => {
      renderForm(onTokenCreated);
    });

    await user.type(screen.getByTestId('token-name-input'), 'Test Token');
    await user.click(screen.getByRole('button', { name: /Generate Token/ }));

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'API token successfully generated'));
    });

    expect(onTokenCreated).toHaveBeenCalledWith(mockToken);
    expect(screen.getByTestId('token-name-input')).toHaveValue('');
  });

  it('handles form submission without name', async () => {
    const user = userEvent.setup();
    const onTokenCreated = vi.fn();

    renderForm(onTokenCreated);

    await user.click(screen.getByRole('button', { name: /Generate Token/ }));

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'API token successfully generated'));
      expect(onTokenCreated).toHaveBeenCalledWith(mockToken);
    });
  });

  it('grants specific models and MCPs picked via the slide-in panels', async () => {
    const user = userEvent.setup();
    let captured: Record<string, unknown> | undefined;
    server.use(
      http.post(`${BODHI_API_BASE}/tokens`, async ({ request }) => {
        captured = (await request.json()) as Record<string, unknown>;
        return HttpResponse.json({ token: mockToken.token }, { status: 201 });
      })
    );

    const onTokenCreated = vi.fn();
    await act(async () => {
      renderForm(onTokenCreated);
    });

    // Model Access → Specific → pick the local alias in the panel
    await user.click(screen.getByTestId('model-access-mode-specific'));
    await user.click(await screen.findByTestId('model-access-panel-item-test-model'));
    await user.click(screen.getByTestId('model-access-panel-done'));
    expect(screen.getByTestId('model-access-selected-test-model')).toBeInTheDocument();

    // MCP Access → Specific → pick the MCP instance in the panel
    await user.click(screen.getByTestId('mcp-access-mode-specific'));
    await user.click(await screen.findByTestId('mcp-access-panel-item-mcp-uuid-1'));
    await user.click(screen.getByTestId('mcp-access-panel-done'));
    expect(screen.getByTestId('mcp-access-selected-mcp-uuid-1')).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: /Generate Token/ }));

    await waitFor(() => expect(onTokenCreated).toHaveBeenCalledWith(mockToken));
    expect(captured?.grants).toEqual({
      version: '1',
      list_models: false,
      models: { type: 'specific', ids: ['test-model'] },
      list_mcps: false,
      mcps: { type: 'specific', ids: ['mcp-uuid-1'] },
    });
  });

  it('removes a selected model from the picker', async () => {
    const user = userEvent.setup();
    await act(async () => {
      renderForm();
    });
    await user.click(screen.getByTestId('model-access-mode-specific'));
    await user.click(await screen.findByTestId('model-access-panel-item-test-model'));
    await user.click(screen.getByTestId('model-access-panel-done'));
    expect(screen.getByTestId('model-access-selected-test-model')).toBeInTheDocument();
    await user.click(screen.getByTestId('model-access-remove-test-model'));
    expect(screen.queryByTestId('model-access-selected-test-model')).not.toBeInTheDocument();
  });

  it('disables the form during submission', async () => {
    const user = userEvent.setup();
    server.use(...mockCreateToken({ token: mockToken.token }, { delayMs: 100 }));

    await act(async () => {
      renderForm();
    });

    const submitButton = screen.getByRole('button', { name: /Generate Token/ });
    await user.click(submitButton);

    expect(submitButton).toBeDisabled();
    expect(screen.getByTestId('token-name-input')).toBeDisabled();
    expect(screen.getByText('Generating...')).toBeInTheDocument();
  });

  it('shows an error toast on api failure', async () => {
    const user = userEvent.setup();
    const onTokenCreated = vi.fn();
    server.use(
      ...mockCreateTokenError({
        message: 'Failed to generate token. Please try again.',
        type: 'invalid_request_error',
      })
    );
    renderForm(onTokenCreated);

    await user.click(screen.getByRole('button', { name: /Generate Token/ }));

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
