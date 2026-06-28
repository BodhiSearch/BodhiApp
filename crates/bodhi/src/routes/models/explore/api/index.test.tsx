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
  localStorage.clear();
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
  it('renders model rows with context/pricing/caps/providers', async () => {
    server.use(...mockCatalogModels());
    await renderScreen();

    const list = screen.getByTestId('cat-model-list');
    expect(within(list).getAllByRole('option').length).toBe(3);

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
    expect(within(screen.getByTestId('cat-model-list')).getAllByRole('option').length).toBe(30);
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
    // price's natural order is asc → order matches natural, so it stays out of the URL.
    expect((router.state.location.search as { order?: string }).order).toBeUndefined();
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
    expect(viewHref).toContain('/models/explore/providers/');
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

describe('ExploreApiScreen (A2b — ?select URL sync for the rail)', () => {
  const ROW = 'cat-model-row-anthropic-claude-sonnet-4.5';
  const DETAIL = 'cat-model-detail-anthropic-claude-sonnet-4.5';
  const KEY = 'anthropic/claude-sonnet-4.5';

  it('clicking a row writes ?select and opens the rail; closing strips it', async () => {
    server.use(...mockCatalogModels(), ...mockCatalogModelDetail());
    const router = await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId(ROW));
    await waitFor(() => expect(router.state.location.search).toMatchObject({ select: KEY }));
    await waitFor(() => expect(screen.getByTestId(DETAIL)).toBeInTheDocument());

    await user.click(screen.getByTestId('cat-model-detail-close'));
    await waitFor(() => expect(router.state.location.search).toEqual({}));
    await waitFor(() => expect(screen.queryByTestId(DETAIL)).not.toBeInTheDocument());
  });

  it('deep-link ?select= opens the rail on mount', async () => {
    server.use(...mockCatalogModels(), ...mockCatalogModelDetail());
    await renderScreen([`/models/explore/api/?select=${encodeURIComponent(KEY)}`]);
    await waitFor(() => expect(screen.getByTestId(DETAIL)).toBeInTheDocument());
  });

  it('Back restores the pre-selection state and Forward re-applies the selection', async () => {
    server.use(...mockCatalogModels(), ...mockCatalogModelDetail());
    const router = await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-model-cap-reasoning'));
    await waitFor(() => expect(router.state.location.search).toMatchObject({ capability: ['reasoning'] }));
    await user.click(screen.getByTestId(ROW));
    await waitFor(() => expect(router.state.location.search).toMatchObject({ select: KEY }));

    await act(async () => router.history.back());
    await waitFor(() => expect(router.state.location.search).toEqual({}));
    await waitFor(() => expect(screen.queryByTestId(DETAIL)).not.toBeInTheDocument());

    await act(async () => router.history.forward());
    await waitFor(() => expect(router.state.location.search).toMatchObject({ select: KEY }));
  });

  it('re-selecting the already-selected row is a no-op (dedup)', async () => {
    server.use(...mockCatalogModels(), ...mockCatalogModelDetail());
    const router = await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId(ROW));
    await waitFor(() => expect(router.state.location.search).toMatchObject({ select: KEY }));
    const before = router.state.location.search;
    await user.click(screen.getByTestId(ROW));
    expect(router.state.location.search).toBe(before);
  });

  it('changing a facet keeps the selection in the URL', async () => {
    server.use(...mockCatalogModels(), ...mockCatalogModelDetail());
    const router = await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId(ROW));
    await waitFor(() => expect(router.state.location.search).toMatchObject({ select: KEY }));
    await user.click(screen.getByTestId('cat-model-cap-reasoning'));
    await waitFor(() => expect(router.state.location.search).toMatchObject({ select: KEY, capability: ['reasoning'] }));
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

  it('first load with no URL sort and no saved pref requests natural order (no sort param)', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogModels({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();
    await waitFor(() => expect(seen.length).toBeGreaterThan(0));
    expect(seen[0].searchParams.has('sort')).toBe(false);
  });

  it('clicking a sort persists the preference to localStorage and writes the URL', async () => {
    server.use(...mockCatalogModels());
    const router = await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-model-sort-name'));
    await waitFor(() => expect(localStorage.getItem('bodhi.explore.api.sort')).toContain('"sort":"name"'));
    expect(router.state.location.search).toMatchObject({ sort: 'name' });
  });

  it('a saved preference drives the request on a clean-URL load without writing the URL', async () => {
    localStorage.setItem('bodhi.explore.api.sort', JSON.stringify({ sort: 'name', order: 'asc' }));
    const seen: URL[] = [];
    server.use(...mockCatalogModels({ onRequest: ({ url }) => seen.push(url) }));
    const router = await renderScreen();

    await waitFor(() => expect(seen.some((u) => u.searchParams.get('sort') === 'name')).toBe(true));
    expect(router.state.location.search).toEqual({});
  });

  it('search auto-applies sort=relevance and reverts to no sort (natural order) when cleared', async () => {
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
      expect(last.searchParams.has('sort')).toBe(false);
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

    // The chip is available (its value is in the global facet array) and so enabled.
    expect(screen.getByTestId('cat-model-cap-reasoning')).toBeEnabled();

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

  it('disables facet values absent from the global facet arrays (no count badges)', async () => {
    const partial = createModelsListResponse(undefined, {
      facets: {
        // structured_output, audio, and beta are omitted → their chips disable.
        capability: ['reasoning', 'tool_call', 'attachment', 'vision'],
        modality: ['text', 'image', 'video', 'pdf'],
        status: ['stable', 'alpha', 'deprecated'],
        provider: ['nano-gpt'],
        family: ['claude'],
        open_weights: ['open', 'closed'],
      },
    });
    server.use(...mockCatalogModels({ response: partial }));
    await renderScreen();

    expect(screen.getByTestId('cat-model-cap-reasoning')).toBeEnabled();
    expect(screen.getByTestId('cat-model-cap-structured_output')).toBeDisabled();
    expect(screen.getByTestId('cat-model-mod-audio')).toBeDisabled();
    expect(screen.getByTestId('cat-model-status-beta')).toBeDisabled();
    // No count badge renders anymore.
    expect(screen.getByTestId('cat-model-cap-reasoning').querySelector('.cat-facet-count')).toBeNull();
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

  it('reset lives in the toolbar (not the sidebar) and is always visible with three states', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogModels({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    const user = userEvent.setup();

    // State 3 (none): with no query and no facets the reset is visible but inert (disabled).
    const reset = await screen.findByTestId('cat-model-clear-all');
    // It renders in the central toolbar, not inside the facet sidebar (no sidebar layout shift).
    expect(screen.getByTestId('cat-model-facets').contains(reset)).toBe(false);
    expect(reset).toHaveAttribute('data-test-state', 'none');
    expect(reset).toBeDisabled();
    expect(reset).toHaveAttribute('aria-label', 'Nothing to reset');

    // Establish a committed search, then a facet (so the precedence — filters before query — can be seen).
    const input = screen.getByTestId('cat-model-search').querySelector('input')!;
    await user.click(input);
    await user.type(input, 'claude{Enter}');
    await waitFor(() => expect(seen[seen.length - 1].searchParams.get('q')).toBe('claude'));
    await user.click(screen.getByTestId('cat-model-cap-reasoning'));

    // State 1 (filters): a facet is active → reset clears facets only, keeping search + its sort.
    await waitFor(() => expect(reset).toHaveAttribute('data-test-state', 'filters'));
    expect(reset).toHaveAttribute('aria-label', 'Clear all filters');
    await user.click(reset);
    await waitFor(() => {
      const last = seen[seen.length - 1];
      expect(last.searchParams.getAll('capability')).toHaveLength(0);
      expect(last.searchParams.get('q')).toBe('claude');
      expect(last.searchParams.get('sort')).toBe('relevance');
    });

    // State 2 (query): no facets left but the query remains → next click clears the query.
    await waitFor(() => expect(reset).toHaveAttribute('data-test-state', 'query'));
    expect(reset).toHaveAttribute('aria-label', 'Clear search');
    await user.click(reset);
    await waitFor(() => expect(seen[seen.length - 1].searchParams.get('q')).toBeNull());

    // Back to inert once nothing is active.
    await waitFor(() => expect(reset).toHaveAttribute('data-test-state', 'none'));
    expect(reset).toBeDisabled();
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

  it('column picker is icon-only with an accessible label, and hides/restores an optional column', async () => {
    server.use(...mockCatalogModels());
    await renderScreen();
    const user = userEvent.setup();

    // Icon-only trigger: no visible "Columns" text, but an accessible name for screen readers.
    const columnsBtn = screen.getByTestId('cat-model-columns');
    expect(columnsBtn).toHaveAccessibleName('Columns');
    expect(columnsBtn).not.toHaveTextContent('Columns');

    expect(screen.getByTestId('cat-model-sort-family')).toBeInTheDocument();
    await user.click(columnsBtn);
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

  it('range value labels are hidden at the default state and revealed while adjusting (no layout shift)', async () => {
    server.use(...mockCatalogModels());
    await renderScreen();
    const user = userEvent.setup();

    // At rest (default "Any"), the pricing + context value labels are present but visually hidden
    // (aria-hidden, no `visible` class) so they reserve space without shifting the sidebar.
    const inVal = screen.getByTestId('cat-model-pricing-in-val');
    const ctxVal = screen.getByTestId('cat-model-context-val');
    expect(inVal).toHaveAttribute('aria-hidden', 'true');
    expect(inVal).not.toHaveClass('visible');
    expect(ctxVal).toHaveAttribute('aria-hidden', 'true');

    // Adjusting a thumb reveals that axis's value.
    const inMin = within(screen.getByTestId('cat-model-pricing-in-slider')).getAllByRole('slider')[0];
    inMin.focus();
    await user.keyboard('{ArrowRight}');
    await waitFor(() => expect(screen.getByTestId('cat-model-pricing-in-val')).toHaveClass('visible'));
    await waitFor(() => expect(screen.getByTestId('cat-model-pricing-in-val')).toHaveAttribute('aria-hidden', 'false'));
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
