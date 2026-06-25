import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { ShellSlotsProvider, useShellSlots } from '@/components/shell';
import { ExploreApiScreen } from '@/routes/models/explore/api/-components/ExploreApiScreen';
import { exploreApiSearchSchema } from '@/routes/models/explore/api/index';
import { createModelDetail, createModelLite, createModelsListResponse } from '@/test-fixtures/catalog-models';
import { makeRouteRouter, RouteHarness } from '@/test-utils/router-harness';
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import {
  mockCatalogError,
  mockCatalogModelDetail,
  mockCatalogModels,
  mockCatalogProviderDetail,
} from '@/test-utils/msw-v2/handlers/reference-catalog';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

vi.mock('@/hooks/useViewTransition', () => ({ useViewTransition: () => (cb: () => void) => cb() }));

setupMswV2();

let Wrapper: ReturnType<typeof createWrapper>;

beforeEach(() => {
  Wrapper = createWrapper();
  server.use(
    ...mockAppInfoReady(),
    ...mockUserLoggedIn({ username: 'admin@example.com', role: 'resource_admin', id_token: 'test-id-token' })
  );
});

// The screen reads its filter/sort/search/page state from the URL via getRouteApi('/models/explore/api/'),
// so tests mount it behind a real in-memory router carrying the route's validateSearch schema. The
// SlotsConsumer surfaces the sidebar/rail shell slots the screen publishes.
function SlotsConsumer() {
  const { sidebar, rail, railHeader } = useShellSlots();
  return (
    <>
      <div data-testid="harness-sidebar">{sidebar}</div>
      <div data-testid="harness-rail-header">{railHeader}</div>
      <div data-testid="harness-rail">{rail}</div>
    </>
  );
}

function ScreenWithSlots() {
  return (
    <>
      <SlotsConsumer />
      <ExploreApiScreen />
    </>
  );
}

function buildRouter(initialEntries?: string[]) {
  return makeRouteRouter({
    path: '/models/explore/api/',
    validateSearch: exploreApiSearchSchema as never,
    Screen: ScreenWithSlots,
    initialEntries,
  });
}

async function renderScreen(initialEntries?: string[]) {
  const router = buildRouter(initialEntries);
  await act(async () => {
    render(
      <ShellSlotsProvider>
        <RouteHarness router={router} />
      </ShellSlotsProvider>,
      { wrapper: Wrapper }
    );
  });
  await waitFor(() => expect(screen.getByTestId('explore-api-content')).toHaveAttribute('data-pagestatus', 'ready'));
  return router;
}

describe('ExploreApiScreen (A1 — list)', () => {
  it('renders model rows with "Showing X of TOTAL", context/pricing/caps/providers', async () => {
    server.use(...mockCatalogModels());
    await renderScreen();

    const list = screen.getByTestId('cat-model-list');
    expect(within(list).getAllByRole('option').length).toBe(3);
    expect(screen.getByTestId('cat-model-resultbar')).toHaveTextContent('Showing 3 of 3');

    const claude = screen.getByTestId('cat-model-row-anthropic-claude-sonnet-4.5');
    expect(claude).toHaveTextContent('Claude Sonnet 4.5');
    expect(claude).toHaveTextContent('200K'); // context
    expect(claude).toHaveTextContent('Reasoning'); // capability chip
    expect(claude).toHaveTextContent('4'); // provider_count
  });

  it('renders Free for zero-priced models and a deprecated status badge', async () => {
    server.use(...mockCatalogModels());
    await renderScreen();
    // Llama 3.3 70B is open-weights, free, deprecated in the fixture.
    const llama = screen.getByTestId('cat-model-row-meta-llama-llama-3.3-70b-instruct');
    expect(llama).toHaveTextContent('Free');
    expect(llama).toHaveTextContent('Deprecated');
  });

  it('reads the catalog anonymously — no Authorization header', async () => {
    let seenAuth: string | null = 'unset';
    let sawRequest = false;
    server.use(
      ...mockCatalogModels({
        onRequest: ({ authorization }) => {
          sawRequest = true;
          seenAuth = authorization;
        },
      })
    );
    await renderScreen();
    await waitFor(() => expect(sawRequest).toBe(true));
    expect(seenAuth).toBeNull();
  });

  it('renders a numbered pager and navigates to page 2', async () => {
    const items = Array.from({ length: 31 }, (_, i) =>
      createModelLite({ slug: 'p', model_id: `m-${i}`, name: `Model ${i}` })
    );
    const seen: URL[] = [];
    server.use(
      ...mockCatalogModels({ response: createModelsListResponse(items), onRequest: ({ url }) => seen.push(url) })
    );
    const router = await renderScreen();

    // Page 1: 30 of 31 rows, pager visible (no Load More).
    expect(screen.getByTestId('cat-model-resultbar')).toHaveTextContent('Showing 30 of 31');
    expect(screen.getByTestId('pagination')).toBeInTheDocument();
    expect(screen.queryByTestId('cat-model-load-more')).not.toBeInTheDocument();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('pagination-page-2'));

    // Page 2: the request carries page=2, the URL reflects it, and the single remaining row renders.
    await waitFor(() => expect(seen.some((u) => u.searchParams.get('page') === '2')).toBe(true));
    expect(router.state.location.search).toMatchObject({ page: 2 });
    await waitFor(() => expect(within(screen.getByTestId('cat-model-list')).getAllByRole('option').length).toBe(1));
  });

  it('resets to page 1 when a filter changes', async () => {
    const items = Array.from({ length: 31 }, (_, i) =>
      createModelLite({ slug: 'p', model_id: `m-${i}`, name: `Model ${i}` })
    );
    const seen: URL[] = [];
    server.use(
      ...mockCatalogModels({ response: createModelsListResponse(items), onRequest: ({ url }) => seen.push(url) })
    );
    await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('pagination-page-2'));
    await waitFor(() => expect(seen.some((u) => u.searchParams.get('page') === '2')).toBe(true));

    // Toggling a facet must reset paging to page 1.
    await user.click(screen.getByTestId('cat-model-cap-reasoning'));
    await waitFor(() => {
      const last = seen[seen.length - 1];
      expect(last.searchParams.getAll('capability')).toContain('reasoning');
      expect(last.searchParams.get('page')).toBe('1');
    });
  });

  it('renders the empty state when the catalog has no models', async () => {
    server.use(...mockCatalogModels({ response: createModelsListResponse([]) }));
    await renderScreen();
    expect(screen.getByTestId('cat-model-empty')).toBeInTheDocument();
  });

  it('renders an error page when the list fails', async () => {
    server.use(...mockCatalogError('models', { status: 500, error: 'internal' }));
    const router = buildRouter();
    await act(async () => {
      render(
        <ShellSlotsProvider>
          <RouteHarness router={router} />
        </ShellSlotsProvider>,
        { wrapper: Wrapper }
      );
    });
    await waitFor(() => expect(screen.getByText(/Reference API error 500/i)).toBeInTheDocument());
  });
});

describe('ExploreApiScreen (URL sync — back/forward + deep link)', () => {
  it('deep-links ?provider=<slug> into an active provider filter and a request param', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogModels({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen(['/models/explore/api/?provider=nano-gpt']);

    // The request carries the deep-linked provider...
    await waitFor(() => expect(seen.some((u) => u.searchParams.getAll('provider').includes('nano-gpt'))).toBe(true));
    // ...and the sidebar shows it as an active removable chip.
    expect(screen.getByTestId('cat-model-provider-chip-nano-gpt')).toBeInTheDocument();
  });

  it('keeps the URL clean at defaults (no sort/order/page) and writes only non-defaults', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogModels({ onRequest: ({ url }) => seen.push(url) }));
    const router = await renderScreen();

    // Initial state: no sort/order/page in the URL.
    expect(router.state.location.search).toEqual({});

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-model-sort-price'));
    await waitFor(() => expect(router.state.location.search).toMatchObject({ sort: 'price' }));
    // price's natural order is asc → asc !== default desc, so order is written.
    expect(router.state.location.search).toMatchObject({ sort: 'price', order: 'asc' });
    expect((router.state.location.search as { page?: number }).page).toBeUndefined();
  });

  it('browser Back re-applies the previous filter set', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogModels({ onRequest: ({ url }) => seen.push(url) }));
    const router = await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-model-cap-reasoning'));
    await waitFor(() => expect(router.state.location.search).toMatchObject({ capability: ['reasoning'] }));

    await act(async () => {
      router.history.back();
    });
    await waitFor(() => expect(router.state.location.search).toEqual({}));
    // The facet chip is gone and the request reverts to no capability.
    await waitFor(() => {
      const last = seen[seen.length - 1];
      expect(last.searchParams.getAll('capability')).toHaveLength(0);
    });
    expect(screen.getByTestId('cat-model-cap-reasoning')).toHaveAttribute('aria-pressed', 'false');
  });
});

describe('ExploreApiScreen (A2 — detail rail + cross-links)', () => {
  it('opens the rail with spec grid + Served-by on row select', async () => {
    server.use(...mockCatalogModels(), ...mockCatalogModelDetail(), ...mockCatalogProviderDetail());
    await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-model-row-anthropic-claude-sonnet-4.5'));

    await waitFor(() => expect(screen.getByTestId('cat-model-detail-anthropic-claude-sonnet-4.5')).toBeInTheDocument());
    const specs = await screen.findByTestId('cat-model-detail-specs');
    expect(specs).toHaveTextContent('Context');
    expect(specs).toHaveTextContent('200K');
    // Served-by from the detail fetch. The per-row Add icon targets the create-API-model form.
    const servedBy = await screen.findByTestId('cat-model-servedby');
    expect(servedBy).toHaveTextContent('Anthropic');
    const add = screen.getByTestId('cat-model-servedby-add-openrouter');
    const addHref = add.getAttribute('href') ?? '';
    expect(addHref).toContain('/models/api/new/');
    expect(addHref).toContain('api_format=openai');
    expect(addHref).toContain('model=claude-sonnet-4.5');
    // Clicking the provider row reveals its connection detail inline (no route change).
    await user.click(screen.getByTestId('cat-model-servedby-toggle-openrouter'));
    expect(await screen.findByTestId('cat-model-servedby-detail-openrouter')).toBeInTheDocument();
  });

  it('the Configure-in-Bodhi CTA is removed (per-provider Add is the configure path)', async () => {
    server.use(...mockCatalogModels(), ...mockCatalogModelDetail());
    await renderScreen();
    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-model-row-anthropic-claude-sonnet-4.5'));
    await screen.findByTestId('cat-model-detail-specs');
    expect(screen.queryByTestId('cat-model-configure-cta')).not.toBeInTheDocument();
    expect(screen.queryByTestId('cat-model-configure-subst')).not.toBeInTheDocument();
  });

  it('served-by detail shows "All Models from Provider" (?provider=slug) and "View" (?q=name) links', async () => {
    server.use(...mockCatalogModels(), ...mockCatalogModelDetail(), ...mockCatalogProviderDetail());
    await renderScreen();
    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-model-row-anthropic-claude-sonnet-4.5'));
    await user.click(await screen.findByTestId('cat-model-servedby-toggle-openrouter'));

    const allModels = await screen.findByTestId('cat-model-servedby-allmodels-openrouter');
    const allHref = allModels.getAttribute('href') ?? '';
    expect(allHref).toContain('/models/explore/api/');
    // provider is an array param → the default stringifier JSON-encodes it (?provider=["openrouter"]).
    expect(decodeURIComponent(allHref)).toContain('provider=["openrouter"]');

    const view = screen.getByTestId('cat-model-servedby-view-openrouter');
    const viewHref = view.getAttribute('href') ?? '';
    expect(viewHref).toContain('/models/explore/api-providers/');
    // q carries the provider's display name (URL-encoded).
    expect(decodeURIComponent(viewHref)).toContain('q=OpenRouter');
  });

  it('synthesizes "Stable" status for a null-status model', async () => {
    server.use(...mockCatalogModels(), ...mockCatalogModelDetail());
    await renderScreen();
    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-model-row-anthropic-claude-sonnet-4.5'));
    const specs = await screen.findByTestId('cat-model-detail-specs');
    expect(specs).toHaveTextContent('Stable');
  });

  it('detail fetch is gated until a model is selected', async () => {
    let detailRequested = false;
    server.use(...mockCatalogModels(), ...mockCatalogModelDetail({ onRequest: () => (detailRequested = true) }));
    await renderScreen();
    expect(detailRequested).toBe(false);
  });

  it('served-by rows show per-provider $in / $out for cross-provider cost comparison', async () => {
    server.use(
      ...mockCatalogModels(),
      ...mockCatalogModelDetail({
        detail: createModelDetail({
          served_by: [
            {
              slug: 'anthropic',
              name: 'Anthropic',
              logo_url: null,
              base_url: 'https://api.anthropic.com/v1',
              pricing: { input_per_m: 3, output_per_m: 15, cache_read_per_m: null, cache_write_per_m: null },
            },
            {
              slug: 'openrouter',
              name: 'OpenRouter',
              logo_url: null,
              base_url: 'https://openrouter.ai/api/v1',
              pricing: { input_per_m: 3.2, output_per_m: 16, cache_read_per_m: null, cache_write_per_m: null },
            },
          ],
        }),
      })
    );
    await renderScreen();
    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-model-row-anthropic-claude-sonnet-4.5'));

    const anthropic = await screen.findByTestId('cat-model-servedby-anthropic');
    expect(anthropic).toHaveTextContent('$3 / $15');
    const openrouter = await screen.findByTestId('cat-model-servedby-openrouter');
    expect(openrouter).toHaveTextContent('$3.20 / $16');
  });
});

describe('ExploreApiScreen (A3 — search + facets + sort)', () => {
  it('search submits q on Enter and keeps the numbered pager (inverse of Local)', async () => {
    const items = Array.from({ length: 31 }, (_, i) =>
      createModelLite({ slug: 'p', model_id: `m-${i}`, name: `Model ${i}` })
    );
    const seen: URL[] = [];
    server.use(
      ...mockCatalogModels({ response: createModelsListResponse(items), onRequest: ({ url }) => seen.push(url) })
    );
    await renderScreen();

    const user = userEvent.setup();
    const input = screen.getByTestId('cat-model-search').querySelector('input')!;
    await user.click(input);
    await user.type(input, 'Model{Enter}');

    await waitFor(() => expect(seen.some((u) => u.searchParams.get('q') === 'Model')).toBe(true));
    // Search does NOT disable pagination here (unlike Local's cursor model).
    expect(screen.getByTestId('pagination')).toBeInTheDocument();
  });

  it('column sort headers send the chosen sort + natural order, and toggle direction on re-click', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogModels({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    const user = userEvent.setup();
    // First click: price adopts its natural ascending direction.
    await user.click(screen.getByTestId('cat-model-sort-price'));
    await waitFor(() => {
      const last = seen[seen.length - 1];
      expect(last.searchParams.get('sort')).toBe('price');
      expect(last.searchParams.get('order')).toBe('asc');
    });
    expect(screen.getByTestId('cat-model-sort-price')).toHaveAttribute('data-test-state', 'active');

    // Re-click the active column toggles to descending.
    await user.click(screen.getByTestId('cat-model-sort-price'));
    await waitFor(() => {
      const last = seen[seen.length - 1];
      expect(last.searchParams.get('sort')).toBe('price');
      expect(last.searchParams.get('order')).toBe('desc');
    });
  });

  it('output-price column is sortable (price_out)', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogModels({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-model-sort-price_out'));
    await waitFor(() => expect(seen.some((u) => u.searchParams.get('sort') === 'price_out')).toBe(true));
  });

  it('search auto-applies sort=relevance and reverts to updated when cleared', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogModels({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    const user = userEvent.setup();
    const input = screen.getByTestId('cat-model-search').querySelector('input')!;
    await user.click(input);
    await user.type(input, 'claude{Enter}');
    await waitFor(() => {
      const last = seen[seen.length - 1];
      expect(last.searchParams.get('q')).toBe('claude');
      expect(last.searchParams.get('sort')).toBe('relevance');
    });

    await user.clear(input);
    await waitFor(() => {
      const last = seen[seen.length - 1];
      expect(last.searchParams.has('q')).toBe(false);
      expect(last.searchParams.get('sort')).toBe('updated');
    });
  });

  it('sends a typo/raw query verbatim with sort=relevance (server handles typo tolerance)', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogModels({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    const user = userEvent.setup();
    const input = screen.getByTestId('cat-model-search').querySelector('input')!;
    await user.click(input);
    await user.type(input, 'clade{Enter}');
    await waitFor(() => {
      const last = seen[seen.length - 1];
      expect(last.searchParams.get('q')).toBe('clade');
      expect(last.searchParams.get('sort')).toBe('relevance');
    });
  });

  it('Free chip sends pricing=free and pins; re-click clears it', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogModels({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-model-pricing-free'));
    await waitFor(() => expect(seen[seen.length - 1].searchParams.get('pricing')).toBe('free'));
    expect(screen.getByTestId('cat-model-pricing-free')).toHaveAttribute('aria-pressed', 'true');

    await user.click(screen.getByTestId('cat-model-pricing-free'));
    await waitFor(() => expect(seen[seen.length - 1].searchParams.has('pricing')).toBe(false));
  });

  it('provider autocomplete selects from facet options, sends provider=, and removes via chip', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogModels({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-model-provider-trigger'));
    // Options come from facets.provider; pick by accessible name (the slug).
    await user.click(await screen.findByRole('option', { name: 'nano-gpt' }));
    await waitFor(() => expect(seen[seen.length - 1].searchParams.getAll('provider')).toContain('nano-gpt'));

    // Selecting renders a removable chip; clicking it clears the filter.
    const chip = await screen.findByTestId('cat-model-provider-chip-nano-gpt');
    await user.click(chip);
    await waitFor(() => expect(seen[seen.length - 1].searchParams.has('provider')).toBe(false));
  });

  it('family autocomplete sends family= from facet options', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogModels({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-model-family-trigger'));
    await user.click(await screen.findByRole('option', { name: 'claude' }));
    await waitFor(() => expect(seen[seen.length - 1].searchParams.getAll('family')).toContain('claude'));
  });

  it('multi-select facets send repeated-key params; Stable maps to status=stable', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogModels({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    // Counts come from the response facets.
    expect(screen.getByTestId('cat-model-cap-reasoning')).toHaveTextContent('1292');

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-model-cap-reasoning'));
    await user.click(screen.getByTestId('cat-model-mod-image'));
    await user.click(screen.getByTestId('cat-model-status-stable'));

    await waitFor(() => {
      const last = seen[seen.length - 1];
      return (
        last.searchParams.getAll('capability').includes('reasoning') &&
        last.searchParams.getAll('modality').includes('image') &&
        last.searchParams.getAll('status').includes('stable')
      );
    });
  });

  it('renders live facet counts and disables zero-count buckets', async () => {
    const zeroed = createModelsListResponse(undefined, {
      facets: {
        capability: { reasoning: 1292, tool_call: 1762, structured_output: 0, attachment: 1123, vision: 1117 },
        modality: { text: 2565, audio: 0, image: 1168, video: 302, pdf: 350 },
        status: { stable: 2477, alpha: 0, beta: 0, deprecated: 66 },
        provider: { 'nano-gpt': 617 },
        family: { claude: 25 },
        open_weights: { open: 900, closed: 1665 },
      },
    });
    server.use(...mockCatalogModels({ response: zeroed }));
    await renderScreen();

    expect(screen.getByTestId('cat-model-cap-reasoning')).toHaveTextContent('1292');
    expect(screen.getByTestId('cat-model-cap-reasoning')).toBeEnabled();
    expect(screen.getByTestId('cat-model-cap-structured_output')).toBeDisabled();
    expect(screen.getByTestId('cat-model-mod-audio')).toBeDisabled();
    expect(screen.getByTestId('cat-model-status-beta')).toBeDisabled();
  });

  it('open_weights is tri-state: unset → open → unset', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogModels({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-model-ow-open'));
    await waitFor(() => expect(seen.some((u) => u.searchParams.get('open_weights') === 'open')).toBe(true));
    expect(screen.getByTestId('cat-model-ow-open')).toHaveAttribute('aria-pressed', 'true');

    await user.click(screen.getByTestId('cat-model-ow-open'));
    await waitFor(() => {
      const last = seen[seen.length - 1];
      return !last.searchParams.has('open_weights');
    });
  });

  it('clear-all resets every facet param', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogModels({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-model-cap-reasoning'));
    await waitFor(() => expect(screen.getByTestId('cat-model-clear-all')).toBeInTheDocument());

    await user.click(screen.getByTestId('cat-model-clear-all'));
    await waitFor(() => {
      const last = seen[seen.length - 1];
      return last.searchParams.getAll('capability').length === 0;
    });
    expect(screen.queryByTestId('cat-model-clear-all')).not.toBeInTheDocument();
  });
});

describe('ExploreApiScreen (A4 — columns + four-param pricing)', () => {
  it('renders the Family column value and a human-readable Updated date', async () => {
    server.use(...mockCatalogModels());
    await renderScreen();
    const claude = screen.getByTestId('cat-model-row-anthropic-claude-sonnet-4.5');
    expect(claude).toHaveTextContent('claude');
    expect(claude.textContent ?? '').toMatch(/\d+(d|mo) ago|[A-Z][a-z]{2} \d{4}/);
  });

  it('Name and Family column headers are sortable', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogModels({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();
    const user = userEvent.setup();

    await user.click(screen.getByTestId('cat-model-sort-name'));
    await waitFor(() => expect(seen.some((u) => u.searchParams.get('sort') === 'name')).toBe(true));

    await user.click(screen.getByTestId('cat-model-sort-family'));
    await waitFor(() => expect(seen.some((u) => u.searchParams.get('sort') === 'family')).toBe(true));
  });

  it('column picker hides and restores an optional column', async () => {
    server.use(...mockCatalogModels());
    await renderScreen();
    const user = userEvent.setup();

    expect(screen.getByTestId('cat-model-sort-family')).toBeInTheDocument();
    await user.click(screen.getByTestId('cat-model-columns'));
    await user.click(await screen.findByTestId('cat-model-col-family'));
    await waitFor(() => expect(screen.queryByTestId('cat-model-sort-family')).not.toBeInTheDocument());

    await user.click(screen.getByTestId('cat-model-col-family'));
    await waitFor(() => expect(screen.getByTestId('cat-model-sort-family')).toBeInTheDocument());
  });

  it('input + output price sliders send pricing_in_* / pricing_out_* independently', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogModels({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();
    const user = userEvent.setup();

    // Sliders are radix primitives; drive the underlying input via keyboard for a deterministic commit.
    const inMin = within(screen.getByTestId('cat-model-pricing-in-slider')).getAllByRole('slider')[0];
    inMin.focus();
    await user.keyboard('{ArrowRight}');
    await waitFor(() => expect(seen.some((u) => u.searchParams.has('pricing_in_min'))).toBe(true));

    const outMin = within(screen.getByTestId('cat-model-pricing-out-slider')).getAllByRole('slider')[0];
    outMin.focus();
    await user.keyboard('{ArrowRight}');
    await waitFor(() => expect(seen.some((u) => u.searchParams.has('pricing_out_min'))).toBe(true));
  });

  it('Free disables the price sliders and clears any range bounds', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogModels({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();
    const user = userEvent.setup();

    await user.click(screen.getByTestId('cat-model-pricing-free'));
    await waitFor(() => expect(seen[seen.length - 1].searchParams.get('pricing')).toBe('free'));
    within(screen.getByTestId('cat-model-pricing-in-slider'))
      .getAllByRole('slider')
      .forEach((s) => expect(s).toHaveAttribute('data-disabled'));
    const last = seen[seen.length - 1].searchParams;
    expect(last.has('pricing_in_min')).toBe(false);
    expect(last.has('pricing_out_min')).toBe(false);
  });
});
