import { Route as ModelsRoute } from '@/routes/models/index';
import { ShellSlotsProvider, useShellSlots } from '@/components/shell';
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { mockModels, mockModelsWithCapture } from '@/test-utils/msw-v2/handlers/models';
import {
  createMockApiAlias,
  createMockModelAlias,
  createMockOpenAIModel,
  createMockUserAlias,
} from '@/test-fixtures/models';
import { server, setupMswV2, type components } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const mockNavigate = vi.fn();
vi.mock('@tanstack/react-router', async () => {
  const actual = await vi.importActual('@tanstack/react-router');
  return {
    ...actual,
    useNavigate: () => mockNavigate,
    useLocation: () => ({ pathname: '/models' }),
  };
});

setupMswV2();

const ModelsPage = ModelsRoute.options.component as React.ComponentType;

function SlotsConsumer() {
  const { sidebar, rail, railHeader, breadcrumb } = useShellSlots();
  const crumbs = Array.isArray(breadcrumb) ? breadcrumb.map((b) => b.label).join(' / ') : '';
  return (
    <>
      <div data-testid="harness-sidebar">{sidebar}</div>
      <div data-testid="harness-rail-header">{railHeader}</div>
      <div data-testid="harness-rail">{rail}</div>
      <div data-testid="harness-breadcrumb">{crumbs}</div>
    </>
  );
}

function makeRouterAlias(): components['schemas']['ModelRouterResponse'] {
  return {
    source: 'model_router',
    id: 'router-1',
    alias: 'smart-fallback',
    targets: [
      { alias: 'openai-main', model: 'gpt-4o', enabled: true },
      { alias: 'anthropic-main', model: 'claude-sonnet-4-5', enabled: false },
      { alias: 'local-coder', model: 'qwen', enabled: true },
    ],
    strategy: { strategy: 'fallback', cooldown_secs: 30, max_attempts: 0, honor_retry_after: true },
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  };
}

const MIXED_ROWS: components['schemas']['AliasResponse'][] = [
  createMockModelAlias({
    alias: 'org/local-gguf:Q4',
    repo: 'org/local-gguf',
    filename: 'local.gguf',
    size: 5 * 1024 ** 3,
  }),
  createMockUserAlias({ id: 'u1', alias: 'my-coder', repo: 'org/coder', filename: 'coder.gguf' }),
  createMockApiAlias({ id: 'openai-main', name: 'openai-main', api_format: 'openai', has_api_key: true }),
  makeRouterAlias(),
];

beforeEach(() => {
  localStorage.setItem('bodhi.ui-v2.models', 'true'); // opt into the V2 screen
  server.use(...mockAppInfoReady(), ...mockUserLoggedIn({ username: 'admin@example.com', role: 'resource_admin' }));
  mockNavigate.mockReset();
});

afterEach(() => {
  localStorage.clear();
  vi.clearAllMocks();
});

async function renderReady() {
  await act(async () => {
    render(
      <ShellSlotsProvider>
        <SlotsConsumer />
        <ModelsPage />
      </ShellSlotsProvider>,
      { wrapper: createWrapper() }
    );
  });
  await waitFor(() => {
    expect(screen.getByTestId('models-content')).toHaveAttribute('data-pagestatus', 'ready');
  });
}

describe('ModelsScreen V2', () => {
  it('renders the four row types with their badges and the breadcrumb', async () => {
    server.use(...mockModels({ data: MIXED_ROWS, total: MIXED_ROWS.length }, { stub: true }));
    await renderReady();

    expect(screen.getByTestId('harness-breadcrumb')).toHaveTextContent('Bodhi / Models / My Models');
    expect(within(screen.getByTestId('model-type-org/local-gguf:Q4')).getByText('Local File')).toBeInTheDocument();
    expect(within(screen.getByTestId('model-type-my-coder')).getByText('Model Alias')).toBeInTheDocument();
    expect(within(screen.getByTestId('model-type-openai-main')).getByText('API Model')).toBeInTheDocument();
    expect(within(screen.getByTestId('model-type-router-1')).getByText('Fallback')).toBeInTheDocument();
  });

  it('publishes the faceted sidebar (type / capability / size / api-format incl. Liberty)', async () => {
    server.use(...mockModels({ data: MIXED_ROWS, total: MIXED_ROWS.length }, { stub: true }));
    await renderReady();

    const sidebar = screen.getByTestId('harness-sidebar');
    expect(within(sidebar).getByTestId('models-facet-type-local_file')).toBeInTheDocument();
    expect(within(sidebar).getByTestId('models-facet-capability-vision')).toBeInTheDocument();
    expect(within(sidebar).getByTestId('models-facet-size')).toBeInTheDocument();
    // API-FORMAT incl. the newly-added Liberty bucket.
    expect(within(sidebar).getByTestId('models-facet-format-openai')).toBeInTheDocument();
    expect(within(sidebar).getByTestId('models-facet-format-liberty')).toBeInTheDocument();
  });

  it('sends the TYPE facet as a server-side `type` query param', async () => {
    const { handlers, capture } = mockModelsWithCapture({ data: MIXED_ROWS, total: MIXED_ROWS.length });
    server.use(...handlers);
    await renderReady();

    await userEvent.click(within(screen.getByTestId('harness-sidebar')).getByTestId('models-facet-type-api_model'));
    await waitFor(() => expect(capture.last?.get('type')).toBe('api_model'));
  });

  it('sends the API-FORMAT Liberty facet as `api_format=liberty`', async () => {
    const { handlers, capture } = mockModelsWithCapture({ data: MIXED_ROWS, total: MIXED_ROWS.length });
    server.use(...handlers);
    await renderReady();

    await userEvent.click(within(screen.getByTestId('harness-sidebar')).getByTestId('models-facet-format-liberty'));
    await waitFor(() => expect(capture.last?.get('api_format')).toBe('liberty'));
  });

  it('sends the CAPABILITY facet as `capability=vision`', async () => {
    const { handlers, capture } = mockModelsWithCapture({ data: MIXED_ROWS, total: MIXED_ROWS.length });
    server.use(...handlers);
    await renderReady();

    await userEvent.click(within(screen.getByTestId('harness-sidebar')).getByTestId('models-facet-capability-vision'));
    await waitFor(() => expect(capture.last?.get('capability')).toBe('vision'));
  });

  it('opens the Local File rail on row click and shows repo/filename/snapshot/size', async () => {
    server.use(...mockModels({ data: MIXED_ROWS, total: MIXED_ROWS.length }, { stub: true }));
    await renderReady();

    await userEvent.click(screen.getByTestId('model-row-org/local-gguf:Q4'));
    const rail = await screen.findByTestId('model-detail-org/local-gguf:Q4');
    expect(within(rail).getByText('org/local-gguf')).toBeInTheDocument();
    expect(within(rail).getByText('local.gguf')).toBeInTheDocument();
    expect(within(rail).getByText('5.00 GB')).toBeInTheDocument();
  });

  it('opens the API rail with connection + models', async () => {
    const api = createMockApiAlias({
      id: 'openai-main',
      name: 'openai-main',
      models: [createMockOpenAIModel('gpt-4o'), createMockOpenAIModel('gpt-4o-mini')],
    });
    server.use(...mockModels({ data: [api], total: 1 }, { stub: true }));
    await renderReady();

    await userEvent.click(screen.getByTestId('model-row-openai-main'));
    const rail = await screen.findByTestId('model-detail-openai-main');
    expect(within(rail).getByText('https://api.openai.com/v1')).toBeInTheDocument();
    expect(within(within(rail).getByTestId('model-detail-models')).getByText('gpt-4o')).toBeInTheDocument();
  });

  it('opens the Fallback rail with the routing chain (disabled step marked)', async () => {
    server.use(...mockModels({ data: [makeRouterAlias()], total: 1 }, { stub: true }));
    await renderReady();

    await userEvent.click(screen.getByTestId('model-row-router-1'));
    const rail = await screen.findByTestId('model-detail-router-1');
    const chain = within(rail).getByTestId('model-detail-chain');
    expect(within(chain).getByText('openai-main')).toBeInTheDocument();
    expect(within(chain).getByText('disabled')).toBeInTheDocument();
  });

  it('Edit CTA on the API rail navigates to the API edit route', async () => {
    const api = createMockApiAlias({ id: 'openai-main', name: 'openai-main' });
    server.use(...mockModels({ data: [api], total: 1 }, { stub: true }));
    await renderReady();

    await userEvent.click(screen.getByTestId('model-row-openai-main'));
    await userEvent.click(await screen.findByTestId('model-detail-edit'));
    expect(mockNavigate).toHaveBeenCalledWith({ to: '/models/api/edit/', search: { id: 'openai-main' } });
  });

  it('shows an empty state when no models match', async () => {
    server.use(...mockModels({ data: [], total: 0 }, { stub: true }));
    await renderReady();
    expect(screen.getByTestId('no-models')).toBeInTheDocument();
  });

  it('falls back to the V1 screen when the models flag is off', async () => {
    localStorage.setItem('bodhi.ui-v2.models', 'false');
    server.use(...mockModels({ data: [createMockUserAlias()], total: 1 }, { stub: true }));
    await act(async () => {
      render(
        <ShellSlotsProvider>
          <SlotsConsumer />
          <ModelsPage />
        </ShellSlotsProvider>,
        { wrapper: createWrapper() }
      );
    });
    // V1 renders the legacy table testid; V2 sidebar facets are absent.
    await waitFor(() => expect(screen.getByTestId('models-content')).toBeInTheDocument());
    expect(screen.queryByTestId('models-facets')).not.toBeInTheDocument();
  });
});
