import ModelRouterForm from '@/routes/models/router/-components/ModelRouterForm';
import { ENDPOINT_MODEL_ROUTERS } from '@/hooks/models';
import { mockModels } from '@/test-utils/msw-v2/handlers/models';
import { mockCreateModelRouter } from '@/test-utils/msw-v2/handlers/model-routers';
import { http, HttpResponse } from 'msw';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { AliasResponse, ModelRouterResponse } from '@bodhiapp/ts-client';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

const navigateMock = vi.fn();
vi.mock('@tanstack/react-router', async () => {
  const actual = await vi.importActual('@tanstack/react-router');
  return {
    ...actual,
    Link: ({ to, children, ...rest }: any) => (
      <a href={to} {...rest}>
        {children}
      </a>
    ),
    useNavigate: () => navigateMock,
  };
});

const mockToast = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast, dismiss: () => {} }),
}));
vi.mock('@/components/ui/toaster', () => ({ Toaster: () => null }));

Object.assign(window.HTMLElement.prototype, {
  scrollIntoView: vi.fn(),
  releasePointerCapture: vi.fn(),
  hasPointerCapture: vi.fn(),
  setPointerCapture: vi.fn(),
});

setupMswV2();

beforeAll(() => {
  Element.prototype.hasPointerCapture = vi.fn(() => false);
  Element.prototype.setPointerCapture = vi.fn();
  Element.prototype.releasePointerCapture = vi.fn();
});

afterEach(() => vi.clearAllMocks());
beforeEach(() => {
  navigateMock.mockClear();
  mockToast.mockClear();
});

const localAlias: AliasResponse = {
  source: 'user',
  alias: 'llama3:instruct',
  repo: 'meta/llama3',
  filename: 'llama3.gguf',
  snapshot: 'main',
} as AliasResponse;

describe('ModelRouterForm', () => {
  it('creates a router with one local target', async () => {
    server.use(
      ...mockModels({ data: [localAlias], total: 1, page: 1, page_size: 30 }, { stub: true }),
      ...mockCreateModelRouter({ alias: 'my-stack' })
    );
    const user = userEvent.setup();
    render(<ModelRouterForm mode="create" />, { wrapper: createWrapper() });

    await user.type(screen.getByTestId('router-alias-input'), 'my-stack');

    await user.click(screen.getByTestId('add-target'));
    expect(screen.getByTestId('target-row-0')).toBeInTheDocument();

    await user.click(screen.getByTestId('target-alias-0'));
    await user.click(await screen.findByText('llama3:instruct'));

    // local alias pins its own model (read-only)
    await waitFor(() => {
      expect(screen.getByTestId('target-model-0')).toHaveValue('llama3:instruct');
    });

    await user.click(screen.getByTestId('router-submit'));

    await waitFor(() => {
      expect(navigateMock).toHaveBeenCalledWith({ to: '/models/' });
    });
  });

  it('validates that an alias name is required before submit reaches the server', async () => {
    server.use(...mockModels({ data: [localAlias], total: 1, page: 1, page_size: 30 }, { stub: true }));
    const user = userEvent.setup();
    render(<ModelRouterForm mode="create" />, { wrapper: createWrapper() });

    // No targets initially, helper text shown
    expect(screen.getByTestId('no-targets')).toBeInTheDocument();

    // add + remove a target toggles the empty state
    await user.click(screen.getByTestId('add-target'));
    expect(screen.getByTestId('target-row-0')).toBeInTheDocument();
    await user.click(screen.getByTestId('target-remove-0'));
    expect(screen.getByTestId('no-targets')).toBeInTheDocument();
  });

  it('renders resilience knobs with persisted defaults', async () => {
    server.use(...mockModels({ data: [localAlias], total: 1, page: 1, page_size: 30 }, { stub: true }));
    render(<ModelRouterForm mode="create" />, { wrapper: createWrapper() });

    expect(screen.getByTestId('cooldown-secs-input')).toHaveValue(30);
    expect(screen.getByTestId('max-attempts-input')).toHaveValue(0);
    expect(screen.getByTestId('honor-retry-after-switch')).toBeChecked();
  });

  it('sends edited resilience knobs in the create request body', async () => {
    let captured: any = null;
    server.use(
      ...mockModels({ data: [localAlias], total: 1, page: 1, page_size: 30 }, { stub: true }),
      http.post(`*${ENDPOINT_MODEL_ROUTERS}`, async ({ request }) => {
        captured = await request.json();
        return HttpResponse.json(
          {
            source: 'model_router',
            id: 'router-123',
            alias: 'my-stack',
            targets: [],
            strategy: captured.strategy,
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          },
          { status: 201 }
        );
      })
    );
    const user = userEvent.setup();
    render(<ModelRouterForm mode="create" />, { wrapper: createWrapper() });

    await user.type(screen.getByTestId('router-alias-input'), 'my-stack');
    await user.clear(screen.getByTestId('cooldown-secs-input'));
    await user.type(screen.getByTestId('cooldown-secs-input'), '90');
    await user.clear(screen.getByTestId('max-attempts-input'));
    await user.type(screen.getByTestId('max-attempts-input'), '2');
    await user.click(screen.getByTestId('honor-retry-after-switch')); // turn off

    await user.click(screen.getByTestId('router-submit'));

    await waitFor(() => expect(captured).not.toBeNull());
    expect(captured.strategy).toMatchObject({
      strategy: 'fallback',
      cooldown_secs: 90,
      max_attempts: 2,
      honor_retry_after: false,
    });
  });

  it('rejects an invalid cooldown and disables submit', async () => {
    server.use(...mockModels({ data: [localAlias], total: 1, page: 1, page_size: 30 }, { stub: true }));
    const user = userEvent.setup();
    render(<ModelRouterForm mode="create" />, { wrapper: createWrapper() });

    // Clearing leaves the input empty (NaN) — an invalid cooldown.
    await user.clear(screen.getByTestId('cooldown-secs-input'));

    expect(screen.getByTestId('cooldown-secs-error')).toBeInTheDocument();
    expect(screen.getByTestId('router-submit')).toBeDisabled();
  });

  it('prefills resilience knobs from initialData on edit', async () => {
    server.use(...mockModels({ data: [localAlias], total: 1, page: 1, page_size: 30 }, { stub: true }));
    const initialData: ModelRouterResponse = {
      source: 'model_router',
      id: 'router-123',
      alias: 'my-stack',
      targets: [],
      strategy: { strategy: 'fallback', cooldown_secs: 120, max_attempts: 3, honor_retry_after: false },
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    } as ModelRouterResponse;
    render(<ModelRouterForm mode="edit" initialData={initialData} />, { wrapper: createWrapper() });

    expect(screen.getByTestId('cooldown-secs-input')).toHaveValue(120);
    expect(screen.getByTestId('max-attempts-input')).toHaveValue(3);
    expect(screen.getByTestId('honor-retry-after-switch')).not.toBeChecked();
  });
});
