import ModelRouterForm from '@/routes/models/router/-components/ModelRouterForm';
import { ENDPOINT_MODEL_ROUTERS } from '@/hooks/models';
import { ShellHarness } from '@/test-utils/shell-harness';
import { createMockOpenAIModel } from '@/test-fixtures/models';
import { mockModels } from '@/test-utils/msw-v2/handlers/models';
import { mockCreateModelRouter } from '@/test-utils/msw-v2/handlers/model-routers';
import { http, HttpResponse } from 'msw';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { AliasResponse, ModelRouterResponse } from '@bodhiapp/ts-client';
import { render, screen, waitFor, within } from '@testing-library/react';
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
      access: true,
    } as ModelRouterResponse;
    render(<ModelRouterForm mode="edit" initialData={initialData} />, { wrapper: createWrapper() });

    expect(screen.getByTestId('cooldown-secs-input')).toHaveValue(120);
    expect(screen.getByTestId('max-attempts-input')).toHaveValue(3);
    expect(screen.getByTestId('honor-retry-after-switch')).not.toBeChecked();
  });
});

// ── V2 rebuild: richer step cards / combobox / route-to-model / rail / validation contract ──
const apiSelectedAlias: AliasResponse = {
  source: 'api',
  id: 'openai-main',
  name: 'OpenAI Main',
  api_format: 'openai',
  base_url: 'https://api.openai.com/v1',
  has_api_key: true,
  models: [createMockOpenAIModel('gpt-4o'), createMockOpenAIModel('gpt-4o-mini')],
  forward_all_with_prefix: false,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
} as AliasResponse;

const apiForwardAllAlias: AliasResponse = {
  source: 'api',
  id: 'openrouter-all',
  name: 'OpenRouter',
  api_format: 'openai',
  base_url: 'https://openrouter.ai/api/v1',
  has_api_key: true,
  models: [],
  prefix: 'or/',
  forward_all_with_prefix: true,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
} as AliasResponse;

describe('ModelRouterForm — V2 rebuild', () => {
  function renderWithAliases() {
    server.use(
      ...mockModels(
        { data: [localAlias, apiSelectedAlias, apiForwardAllAlias], total: 3, page: 1, page_size: 30 },
        { stub: true }
      ),
      ...mockCreateModelRouter({ alias: 'my-stack' })
    );
    return userEvent.setup();
  }

  /** Open the cmdk combobox for target `idx` and pick the option whose accessible name is `id`. */
  async function pickAlias(user: ReturnType<typeof userEvent.setup>, idx: number, id: string) {
    await user.click(screen.getByTestId(`target-alias-${idx}`));
    await user.click(await screen.findByRole('option', { name: id }));
  }

  it('shows the type + provider badges in the alias-meta line for an API target', async () => {
    const user = renderWithAliases();
    render(<ModelRouterForm mode="create" />, { wrapper: createWrapper() });

    await user.click(screen.getByTestId('add-target'));
    await pickAlias(user, 0, 'openai-main');

    const meta = await screen.findByTestId('target-alias-meta-0');
    expect(within(meta).getByText('API Model')).toBeInTheDocument();
    expect(within(meta).getByText('OPENAI')).toBeInTheDocument();
  });

  it('renders a constrained model dropdown for a selected-subset API target', async () => {
    const user = renderWithAliases();
    render(<ModelRouterForm mode="create" />, { wrapper: createWrapper() });

    await user.click(screen.getByTestId('add-target'));
    await pickAlias(user, 0, 'openai-main');

    // selected-subset → first matchable model auto-pins; route-to-model is a select trigger.
    await waitFor(() => expect(screen.getByText('pre-configured only')).toBeInTheDocument());
    expect(screen.getByTestId('target-model-0')).toHaveTextContent('gpt-4o');
  });

  it('renders a free-text model input for a forward-all API target and keeps Create enabled when empty', async () => {
    const user = renderWithAliases();
    render(<ModelRouterForm mode="create" />, { wrapper: createWrapper() });

    await user.type(screen.getByTestId('router-alias-input'), 'my-stack');
    await user.click(screen.getByTestId('add-target'));
    await pickAlias(user, 0, 'openrouter-all');

    // forward-all → free-text input (empty model is allowed; server is the authority).
    await waitFor(() => expect(screen.getByText('any model · forward-all')).toBeInTheDocument());
    const modelInput = screen.getByTestId('target-model-0');
    expect(modelInput).toHaveValue('');
    // CONTRACT (decision 3): an empty forward-all model must NOT disable Create.
    expect(screen.getByTestId('router-submit')).toBeEnabled();
  });

  it('toggles a step off and marks it skipped in the rail chain', async () => {
    const user = renderWithAliases();
    render(
      <ShellHarness>
        <ModelRouterForm mode="create" />
      </ShellHarness>,
      { wrapper: createWrapper() }
    );

    await user.click(screen.getByTestId('add-target'));
    await pickAlias(user, 0, 'llama3:instruct');

    await user.click(screen.getByTestId('target-enabled-0'));

    const chain = await screen.findByTestId('router-rail-chain');
    expect(within(chain).getByText('skipped')).toBeInTheDocument();
  });

  it('reorders targets with the up/down arrows', async () => {
    const user = renderWithAliases();
    render(<ModelRouterForm mode="create" />, { wrapper: createWrapper() });

    await user.click(screen.getByTestId('add-target'));
    await pickAlias(user, 0, 'openai-main');
    await user.click(screen.getByTestId('add-target'));
    await pickAlias(user, 1, 'llama3:instruct');

    // Row 0 = OpenAI Main, row 1 = llama3:instruct. Move row 1 up → swap.
    await user.click(screen.getByTestId('target-up-1'));

    await waitFor(() => {
      // Trigger shows the human label (API alias name), not the opaque id.
      expect(screen.getByTestId('target-alias-0')).toHaveTextContent('llama3:instruct');
      expect(screen.getByTestId('target-alias-1')).toHaveTextContent('OpenAI Main');
    });
  });

  it('keeps Create enabled with no targets and an empty name (only invalid resilience disables it)', async () => {
    const user = renderWithAliases();
    render(<ModelRouterForm mode="create" />, { wrapper: createWrapper() });

    // 0 targets + empty name → still submittable (server authority).
    expect(screen.getByTestId('no-targets')).toBeInTheDocument();
    expect(screen.getByTestId('router-submit')).toBeEnabled();

    // Only an invalid resilience field disables it.
    await user.clear(screen.getByTestId('cooldown-secs-input'));
    expect(screen.getByTestId('router-submit')).toBeDisabled();
  });

  it('publishes the live rail chain reflecting the steps in order', async () => {
    const user = renderWithAliases();
    render(
      <ShellHarness>
        <ModelRouterForm mode="create" />
      </ShellHarness>,
      { wrapper: createWrapper() }
    );

    await user.click(screen.getByTestId('add-target'));
    await pickAlias(user, 0, 'openai-main');

    const chain = await screen.findByTestId('router-rail-chain');
    // The chain shows the human alias label (API alias name), not the opaque id.
    expect(within(chain).getByText('OpenAI Main')).toBeInTheDocument();
  });
});
