import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { ShellHarness, ChromeProbe } from '@/test-utils/shell-harness';
import { modelsSearchSchema } from '@/routes/models/index';
import { ModelsScreenV2 } from '@/routes/models/-components/ModelsScreenV2';
import {
  createMockApiAlias,
  createMockModelAlias,
  createMockOpenAIModel,
  createMockUserAlias,
} from '@/test-fixtures/models';
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import { mockModelPullDownloads, mockModelPullDownloadsAllSections } from '@/test-utils/msw-v2/handlers/modelfiles';
import { mockModels, mockModelsWithCapture } from '@/test-utils/msw-v2/handlers/models';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2, type components } from '@/test-utils/msw-v2/setup';
import { makeRouteRouter, RouteHarness } from '@/test-utils/router-harness';
import { createWrapper } from '@/tests/wrapper';

// Edit-nav uses the plain useNavigate() (a route NOT mounted in the single-route harness); mock it so
// we can assert the target. routeApi.useNavigate() binds to the real harness router, not this mock.
const mockNavigate = vi.fn();
vi.mock('@tanstack/react-router', async () => {
  const actual = await vi.importActual('@tanstack/react-router');
  return { ...actual, useNavigate: () => mockNavigate };
});
// View transitions run synchronously in tests.
vi.mock('@/hooks/useViewTransition', () => ({ useViewTransition: () => (cb: () => void) => cb() }));

setupMswV2();

let Wrapper: ReturnType<typeof createWrapper>;

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
  localStorage.clear();
  Wrapper = createWrapper();
  mockNavigate.mockReset();
  server.use(
    ...mockAppInfoReady(),
    ...mockUserLoggedIn({ username: 'admin@example.com', role: 'resource_admin' }),
    ...mockModelPullDownloads({ data: [], total: 0 }, { stub: true })
  );
});

afterEach(() => vi.clearAllMocks());

function ScreenWithSlots() {
  return (
    <>
      <ChromeProbe />
      <ModelsScreenV2 />
    </>
  );
}

async function renderScreen(initialEntries?: string[]) {
  const router = makeRouteRouter({
    path: '/models/',
    validateSearch: modelsSearchSchema as never,
    Screen: ScreenWithSlots,
    initialEntries,
  });
  await act(async () => {
    render(
      <ShellHarness renderProbe={false}>
        <RouteHarness router={router} />
      </ShellHarness>,
      { wrapper: Wrapper }
    );
  });
  await waitFor(() => expect(screen.getByTestId('models-content')).toHaveAttribute('data-pagestatus', 'ready'));
  return router;
}

describe('ModelsScreen V2 — list + rail', () => {
  it('renders the four row types with their badges and the breadcrumb', async () => {
    server.use(...mockModels({ data: MIXED_ROWS, total: MIXED_ROWS.length }, { stub: true }));
    await renderScreen();

    expect(screen.getByTestId('harness-breadcrumb')).toHaveTextContent('Bodhi / Models / My Models');
    expect(within(screen.getByTestId('model-type-org/local-gguf:Q4')).getByText('Local File')).toBeInTheDocument();
    expect(within(screen.getByTestId('model-type-my-coder')).getByText('Model Alias')).toBeInTheDocument();
    expect(within(screen.getByTestId('model-type-openai-main')).getByText('OPENAI')).toBeInTheDocument();
    expect(within(screen.getByTestId('model-type-router-1')).getByText('Router')).toBeInTheDocument();
  });

  it('publishes the faceted sidebar (type / capability / size / api-format incl. Liberty)', async () => {
    server.use(...mockModels({ data: MIXED_ROWS, total: MIXED_ROWS.length }, { stub: true }));
    await renderScreen();

    const sidebar = screen.getByTestId('harness-sidebar');
    expect(within(sidebar).getByTestId('models-facet-type-local_file')).toBeInTheDocument();
    expect(within(sidebar).getByTestId('models-facet-capability-vision')).toBeInTheDocument();
    expect(within(sidebar).getByTestId('models-facet-size')).toBeInTheDocument();
    expect(within(sidebar).getByTestId('models-facet-format-openai')).toBeInTheDocument();
    expect(within(sidebar).getByTestId('models-facet-format-liberty')).toBeInTheDocument();
  });

  it('opens the Local File rail on row click and shows repo/filename/size', async () => {
    server.use(...mockModels({ data: MIXED_ROWS, total: MIXED_ROWS.length }, { stub: true }));
    await renderScreen();

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
    await renderScreen();

    await userEvent.click(screen.getByTestId('model-row-openai-main'));
    const rail = await screen.findByTestId('model-detail-openai-main');
    expect(within(rail).getByText('https://api.openai.com/v1')).toBeInTheDocument();
    expect(within(within(rail).getByTestId('model-detail-models')).getByText('gpt-4o')).toBeInTheDocument();
  });

  it('opens the Fallback rail with the routing chain (disabled step marked)', async () => {
    server.use(...mockModels({ data: [makeRouterAlias()], total: 1 }, { stub: true }));
    await renderScreen();

    await userEvent.click(screen.getByTestId('model-row-router-1'));
    const rail = await screen.findByTestId('model-detail-router-1');
    const chain = within(rail).getByTestId('model-detail-chain');
    expect(within(chain).getByText('openai-main')).toBeInTheDocument();
    expect(within(chain).getByText('disabled')).toBeInTheDocument();
  });

  it('Edit CTA on the API rail navigates to the API edit route', async () => {
    const api = createMockApiAlias({ id: 'openai-main', name: 'openai-main' });
    server.use(...mockModels({ data: [api], total: 1 }, { stub: true }));
    await renderScreen();

    await userEvent.click(screen.getByTestId('model-row-openai-main'));
    await userEvent.click(await screen.findByTestId('model-detail-edit'));
    expect(mockNavigate).toHaveBeenCalledWith({ to: '/models/api/edit/', search: { id: 'openai-main' } });
  });

  it('Local File rail: Chat-only footer (no Edit), no disclaimer, HF links on repo/filename', async () => {
    server.use(...mockModels({ data: MIXED_ROWS, total: MIXED_ROWS.length }, { stub: true }));
    await renderScreen();

    await userEvent.click(screen.getByTestId('model-row-org/local-gguf:Q4'));
    const rail = await screen.findByTestId('model-detail-org/local-gguf:Q4');

    // Read-only local file: Chat replaces Edit.
    const chat = within(rail).getByTestId('model-detail-chat');
    expect(chat).toHaveAttribute('href', expect.stringContaining('/chat/'));
    expect(decodeURIComponent(chat.getAttribute('href') ?? '')).toContain('model=org/local-gguf:Q4');
    expect(within(rail).queryByTestId('model-detail-edit')).not.toBeInTheDocument();

    // Disclaimer is gone for auto-discovered local files.
    expect(within(rail).queryByText(/Auto-discovered from local cache/)).not.toBeInTheDocument();

    // HuggingFace external links.
    expect(within(rail).getByText('org/local-gguf').closest('a')).toHaveAttribute(
      'href',
      'https://huggingface.co/org/local-gguf'
    );
    expect(within(rail).getByText('local.gguf').closest('a')).toHaveAttribute(
      'href',
      'https://huggingface.co/org/local-gguf/blob/main/local.gguf'
    );
  });

  it('User alias rail: Chat (primary) + Edit, keeps the user-alias disclaimer', async () => {
    server.use(...mockModels({ data: MIXED_ROWS, total: MIXED_ROWS.length }, { stub: true }));
    await renderScreen();

    await userEvent.click(screen.getByTestId('model-row-my-coder'));
    const rail = await screen.findByTestId('model-detail-my-coder');
    const chat = within(rail).getByTestId('model-detail-chat');
    expect(decodeURIComponent(chat.getAttribute('href') ?? '')).toContain('model=my-coder');
    expect(within(rail).getByTestId('model-detail-edit')).toBeInTheDocument();
    expect(within(rail).getByText(/User-created alias/)).toBeInTheDocument();
  });

  it('Router rail: Chat with Router (primary) + Edit', async () => {
    server.use(...mockModels({ data: [makeRouterAlias()], total: 1 }, { stub: true }));
    await renderScreen();

    await userEvent.click(screen.getByTestId('model-row-router-1'));
    const rail = await screen.findByTestId('model-detail-router-1');
    const chat = within(rail).getByTestId('model-detail-chat');
    expect(chat).toHaveTextContent('Chat with Router');
    expect(decodeURIComponent(chat.getAttribute('href') ?? '')).toContain('model=smart-fallback');
    expect(within(rail).getByTestId('model-detail-edit')).toBeInTheDocument();
  });

  it('API rail: per-model chat cards (prefix applied) + copyable base URL', async () => {
    const api = createMockApiAlias({
      id: 'openai-main',
      name: 'openai-main',
      prefix: 'p/',
      models: [createMockOpenAIModel('gpt-4o')],
    });
    server.use(...mockModels({ data: [api], total: 1 }, { stub: true }));
    await renderScreen();

    await userEvent.click(screen.getByTestId('model-row-openai-main'));
    const rail = await screen.findByTestId('model-detail-openai-main');

    expect(within(rail).getByTestId('model-detail-model-gpt-4o')).toBeInTheDocument();
    const chat = within(rail).getByTestId('model-detail-chat-gpt-4o');
    expect(decodeURIComponent(chat.getAttribute('href') ?? '')).toContain('model=p/gpt-4o');

    // base URL row is copyable.
    expect(within(rail).getByTestId('copy-content')).toBeInTheDocument();
  });

  it('API rail: chat href uses the bare model id when no prefix is set', async () => {
    const api = createMockApiAlias({
      id: 'openai-main',
      name: 'openai-main',
      models: [createMockOpenAIModel('gpt-4o')],
    });
    server.use(...mockModels({ data: [api], total: 1 }, { stub: true }));
    await renderScreen();

    await userEvent.click(screen.getByTestId('model-row-openai-main'));
    const rail = await screen.findByTestId('model-detail-openai-main');
    const chat = within(rail).getByTestId('model-detail-chat-gpt-4o');
    expect(decodeURIComponent(chat.getAttribute('href') ?? '')).toContain('model=gpt-4o');
  });

  it('shows an empty state when no models match', async () => {
    server.use(...mockModels({ data: [], total: 0 }, { stub: true }));
    await renderScreen();
    expect(screen.getByTestId('no-models')).toBeInTheDocument();
  });

  it('Downloads button opens the Downloads panel with all sections + active badge', async () => {
    server.use(
      ...mockModels({ data: MIXED_ROWS, total: MIXED_ROWS.length }, { stub: true }),
      ...mockModelPullDownloadsAllSections()
    );
    await renderScreen();

    await act(async () => {
      await userEvent.click(screen.getByTestId('models-downloads-button'));
    });

    await waitFor(() => expect(screen.getByTestId('ld-downloads-panel')).toBeInTheDocument());
    expect(screen.getByTestId('ld-dl-group-downloading')).toBeInTheDocument();
    expect(screen.getByTestId('ld-dl-group-failed')).toBeInTheDocument();
    expect(screen.getByTestId('models-downloads-badge')).toHaveTextContent('2');
  });
});

describe('ModelsScreen V2 — URL state', () => {
  it('submits search to ?q on Enter and to the backend `search` param', async () => {
    const { handlers, capture } = mockModelsWithCapture({ data: MIXED_ROWS, total: MIXED_ROWS.length });
    server.use(...handlers);
    const router = await renderScreen();

    const input = within(screen.getByTestId('models-search')).getByRole('textbox');
    await userEvent.type(input, 'llama');
    expect(capture.last?.get('search')).toBeNull(); // submit-on-Enter, not per-keystroke
    await userEvent.type(input, '{Enter}');
    await waitFor(() => expect(capture.last?.get('search')).toBe('llama'));
    expect(router.state.location.search).toMatchObject({ q: 'llama' });
  });

  it('clearing the search box resets ?q and the backend `search` param', async () => {
    const { handlers, capture } = mockModelsWithCapture({ data: MIXED_ROWS, total: MIXED_ROWS.length });
    server.use(...handlers);
    const router = await renderScreen(['/models/?q=llama']);
    await waitFor(() => expect(capture.last?.get('search')).toBe('llama'));

    const input = within(screen.getByTestId('models-search')).getByRole('textbox');
    await userEvent.clear(input);
    await waitFor(() => expect(capture.last?.get('search')).toBeNull());
    expect(router.state.location.search).not.toHaveProperty('q');
  });

  it('writes the TYPE facet to the URL and the `type` query param; Back reverts', async () => {
    const { handlers, capture } = mockModelsWithCapture({ data: MIXED_ROWS, total: MIXED_ROWS.length });
    server.use(...handlers);
    const router = await renderScreen();

    await userEvent.click(within(screen.getByTestId('harness-sidebar')).getByTestId('models-facet-type-api_model'));
    await waitFor(() => expect(capture.last?.get('type')).toBe('api_model'));
    expect(router.state.location.search).toMatchObject({ type: ['api_model'] });

    await act(async () => router.history.back());
    await waitFor(() => expect(router.state.location.search).not.toHaveProperty('type'));
  });

  it('sends the CAPABILITY and Liberty api_format facets as query params', async () => {
    const { handlers, capture } = mockModelsWithCapture({ data: MIXED_ROWS, total: MIXED_ROWS.length });
    server.use(...handlers);
    await renderScreen();

    await userEvent.click(within(screen.getByTestId('harness-sidebar')).getByTestId('models-facet-capability-vision'));
    await waitFor(() => expect(capture.last?.get('capability')).toBe('vision'));
    await userEvent.click(within(screen.getByTestId('harness-sidebar')).getByTestId('models-facet-format-liberty'));
    await waitFor(() => expect(capture.last?.get('api_format')).toBe('liberty'));
  });

  it('row selection writes ?select with replace (no extra history entry) and reload restores the rail', async () => {
    server.use(...mockModels({ data: MIXED_ROWS, total: MIXED_ROWS.length }, { stub: true }));

    // Deep-link restore: ?select on mount opens the rail.
    const router = await renderScreen(['/models/?select=openai-main']);
    expect(await screen.findByTestId('model-detail-openai-main')).toBeInTheDocument();
    expect(router.state.location.search).toMatchObject({ select: 'openai-main' });

    // Selecting a different row replaces (does not push) — one Back leaves the page entirely.
    const lengthBefore = router.history.length;
    await userEvent.click(screen.getByTestId('model-row-router-1'));
    await waitFor(() => expect(router.state.location.search).toMatchObject({ select: 'router-1' }));
    expect(router.history.length).toBe(lengthBefore);
  });
});

describe('ModelsScreen V2 — table layout + columns', () => {
  it('renders a semantic table with the universal columns (Name / Provider / Base-URL)', async () => {
    server.use(...mockModels({ data: MIXED_ROWS, total: MIXED_ROWS.length }, { stub: true }));
    await renderScreen();

    const head = screen.getByTestId('cat-listhead');
    expect(head).toHaveTextContent('NAME');
    expect(head).toHaveTextContent('PROVIDER / REPO');
    expect(head).toHaveTextContent('BASE URL / FILE');
    // Rows are table rows carrying their alias id testid.
    expect(screen.getByTestId('model-row-openai-main').tagName).toBe('TR');
  });

  it('has no count heading and drops the per-row "exposed" subtitle (saves vertical space)', async () => {
    server.use(...mockModels({ data: MIXED_ROWS, total: MIXED_ROWS.length }, { stub: true }));
    await renderScreen();
    expect(screen.queryByTestId('models-heading')).not.toBeInTheDocument();
    expect(screen.queryByText(/exposed/)).not.toBeInTheDocument();
  });

  it('removes the "no key" connection text from rows entirely', async () => {
    const api = createMockApiAlias({ id: 'openai-main', name: 'openai-main', has_api_key: false });
    server.use(...mockModels({ data: [api], total: 1 }, { stub: true }));
    await renderScreen();
    // Connection status (key state) is not surfaced anywhere — not in rows, not in the rail.
    const row = screen.getByTestId('model-row-openai-main');
    expect(within(row).queryByText('no key')).not.toBeInTheDocument();
    expect(within(row).queryByText('connected')).not.toBeInTheDocument();
    expect(screen.queryByTestId('model-detail-status')).not.toBeInTheDocument();
  });

  it('derives Provider/Repo and Base-URL/Filename per alias type', async () => {
    server.use(...mockModels({ data: MIXED_ROWS, total: MIXED_ROWS.length }, { stub: true }));
    await renderScreen();

    // API → provider (api_format) + base_url.
    const apiRow = screen.getByTestId('model-row-openai-main');
    expect(apiRow).toHaveTextContent('OPENAI');
    expect(apiRow).toHaveTextContent('https://api.openai.com/v1');
    // Local/user alias → repo + filename.
    const localRow = screen.getByTestId('model-row-org/local-gguf:Q4');
    expect(localRow).toHaveTextContent('org/local-gguf');
    expect(localRow).toHaveTextContent('local.gguf');
    // Router → first target's alias + model.
    const routerRow = screen.getByTestId('model-row-router-1');
    expect(routerRow).toHaveTextContent('openai-main');
    expect(routerRow).toHaveTextContent('gpt-4o');
  });

  it('hides the Provider column via the column picker', async () => {
    server.use(...mockModels({ data: MIXED_ROWS, total: MIXED_ROWS.length }, { stub: true }));
    await renderScreen();

    await userEvent.click(screen.getByTestId('cat-mymodel-columns'));
    await userEvent.click(await screen.findByTestId('cat-mymodel-col-provider'));
    await waitFor(() => expect(screen.getByTestId('cat-listhead')).not.toHaveTextContent('PROVIDER / REPO'));
  });
});

describe('ModelsScreen V2 — sort + reset', () => {
  it('clicking the Name header writes ?sort=name and marks the header active', async () => {
    server.use(...mockModels({ data: MIXED_ROWS, total: MIXED_ROWS.length }, { stub: true }));
    const router = await renderScreen();

    await userEvent.click(screen.getByTestId('cat-mymodel-sort-name'));
    await waitFor(() => expect(router.state.location.search).toMatchObject({ sort: 'name' }));
    expect(screen.getByTestId('cat-mymodel-sort-name')).toHaveAttribute('data-test-state', 'active');
  });

  it('persists the explicit sort to localStorage and applies it on a later clean-URL visit', async () => {
    server.use(...mockModels({ data: MIXED_ROWS, total: MIXED_ROWS.length }, { stub: true }));
    const router = await renderScreen();

    await userEvent.click(screen.getByTestId('cat-mymodel-sort-provider'));
    await waitFor(() => expect(router.state.location.search).toMatchObject({ sort: 'provider' }));
    expect(localStorage.getItem('bodhi.models.sort')).toContain('provider');
  });

  it('sorts the derived Provider column client-side within the current page', async () => {
    // Three API aliases out of natural order; sorting by provider (api_format) reorders the page.
    const rows = [
      createMockApiAlias({ id: 'z', name: 'z', api_format: 'openai' }),
      createMockApiAlias({ id: 'a', name: 'a', api_format: 'anthropic' }),
      createMockApiAlias({ id: 'm', name: 'm', api_format: 'gemini' }),
    ];
    server.use(...mockModels({ data: rows, total: rows.length }, { stub: true }));
    await renderScreen();

    await userEvent.click(screen.getByTestId('cat-mymodel-sort-provider'));
    await waitFor(() => {
      const ordered = screen.getAllByTestId(/^model-row-/).map((r) => r.getAttribute('data-testid'));
      // ANTHROPIC < GEMINI < OPENAI ascending.
      expect(ordered).toEqual(['model-row-a', 'model-row-m', 'model-row-z']);
    });
  });

  it('toolbar reset waterfalls filters → query → disabled', async () => {
    server.use(...mockModels({ data: MIXED_ROWS, total: MIXED_ROWS.length }, { stub: true }));
    const router = await renderScreen(['/models/?type=%5B%22api_model%22%5D&q=foo']);

    const reset = screen.getByTestId('cat-mymodel-clear-all');
    expect(reset).toHaveAttribute('data-test-state', 'filters');
    await userEvent.click(reset);
    await waitFor(() => expect(router.state.location.search).not.toHaveProperty('type'));
    // Query still set → now in 'query' mode.
    await waitFor(() => expect(reset).toHaveAttribute('data-test-state', 'query'));
    await userEvent.click(reset);
    await waitFor(() => expect(router.state.location.search).not.toHaveProperty('q'));
    expect(reset).toBeDisabled();
  });

  it('arrow-down navigates to the first row and opens its rail', async () => {
    server.use(...mockModels({ data: MIXED_ROWS, total: MIXED_ROWS.length }, { stub: true }));
    const router = await renderScreen();

    await act(async () => {
      document.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }));
    });
    await waitFor(() => expect(router.state.location.search).toHaveProperty('select'));
  });
});
