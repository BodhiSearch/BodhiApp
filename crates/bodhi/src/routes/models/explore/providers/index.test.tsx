import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { ShellChromeProvider, useShellSlots } from '@/components/shell';
import { ExploreProvidersScreen } from '@/routes/models/explore/providers/-components/ExploreProvidersScreen';
import { exploreProvidersSearchSchema } from '@/routes/models/explore/providers/index';
import { createProviderListResponse, createProviderSummary } from '@/test-fixtures/catalog-providers';
import { makeRouteRouter, RouteHarness } from '@/test-utils/router-harness';
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import {
  mockCatalogError,
  mockCatalogProviderDetail,
  mockCatalogProviderModels,
  mockCatalogProviders,
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
      <ExploreProvidersScreen />
    </>
  );
}

function buildRouter(initialEntries?: string[]) {
  return makeRouteRouter({
    path: '/models/explore/providers/',
    validateSearch: exploreProvidersSearchSchema as never,
    Screen: ScreenWithSlots,
    initialEntries,
  });
}

async function renderScreen(initialEntries?: string[]) {
  const router = buildRouter(initialEntries);
  await act(async () => {
    render(
      <ShellChromeProvider>
        <RouteHarness router={router} />
      </ShellChromeProvider>,
      { wrapper: Wrapper }
    );
  });
  await waitFor(() =>
    expect(screen.getByTestId('explore-providers-content')).toHaveAttribute('data-pagestatus', 'ready')
  );
  return router;
}

describe('ExploreProvidersScreen (B1 — list)', () => {
  it('renders provider rows + api_format column from the catalog', async () => {
    server.use(...mockCatalogProviders());
    await renderScreen();

    const list = screen.getByTestId('cat-prov-list');
    expect(within(list).getAllByRole('option').length).toBe(3);
    expect(screen.getByTestId('cat-prov-row-nano-gpt')).toHaveTextContent('NanoGPT');
    expect(screen.getByTestId('cat-prov-row-nano-gpt')).toHaveTextContent('617');
    expect(screen.getByTestId('cat-prov-row-nano-gpt')).toHaveTextContent('Reasoning');
    // The api_format column header is present (FORMAT) and rows carry the format hint.
    expect(screen.getByTestId('cat-listhead')).toHaveTextContent('FORMAT');
  });

  it('has no result bar (the count lives in the pager)', async () => {
    server.use(...mockCatalogProviders());
    await renderScreen();
    expect(screen.queryByTestId('cat-prov-resultbar')).not.toBeInTheDocument();
  });

  it('reads the catalog anonymously — no Authorization header', async () => {
    let seenAuth: string | null = 'unset';
    let sawRequest = false;
    server.use(
      ...mockCatalogProviders({
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

  it('first load with a clean URL and no saved pref requests natural order (no sort param)', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogProviders({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();
    await waitFor(() => expect(seen.length).toBeGreaterThan(0));
    expect(seen[0].searchParams.has('sort')).toBe(false);
  });

  it('renders a numbered pager and navigates to page 2', async () => {
    const items = Array.from({ length: 31 }, (_, i) =>
      createProviderSummary({ slug: `prov-${i}`, name: `Provider ${i}`, rank: i + 1 })
    );
    const seen: URL[] = [];
    server.use(
      ...mockCatalogProviders({ response: createProviderListResponse(items), onRequest: ({ url }) => seen.push(url) })
    );
    await renderScreen();

    expect(screen.getByTestId('pagination')).toBeInTheDocument();
    expect(screen.queryByTestId('cat-prov-load-more')).not.toBeInTheDocument();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('pagination-next'));

    await waitFor(() => expect(seen.some((u) => u.searchParams.get('page') === '2')).toBe(true));
    await waitFor(() => expect(within(screen.getByTestId('cat-prov-list')).getAllByRole('option').length).toBe(1));
  });

  it('renders the empty state when the catalog has no providers', async () => {
    server.use(...mockCatalogProviders({ response: createProviderListResponse([]) }));
    await renderScreen();
    expect(screen.getByTestId('cat-prov-empty')).toBeInTheDocument();
  });

  it('renders an error page when the catalog list fails', async () => {
    server.use(...mockCatalogError('providers', { status: 500, error: 'internal' }));
    const router = buildRouter();
    await act(async () => {
      render(
        <ShellChromeProvider>
          <RouteHarness router={router} />
        </ShellChromeProvider>,
        { wrapper: Wrapper }
      );
    });
    await waitFor(() => expect(screen.getByText(/Reference API error 500/i)).toBeInTheDocument());
  });
});

describe('ExploreProvidersScreen (B2 — detail rail)', () => {
  it('opens the rail with connection meta + the provider models on row select', async () => {
    server.use(...mockCatalogProviders(), ...mockCatalogProviderDetail(), ...mockCatalogProviderModels());
    await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-prov-row-nano-gpt'));

    await waitFor(() => expect(screen.getByTestId('cat-prov-detail-nano-gpt')).toBeInTheDocument());
    const meta = await screen.findByTestId('cat-prov-detail-meta');
    expect(meta).toHaveTextContent('NANO_GPT_API_KEY');
    expect(meta).toHaveTextContent('https://nano-gpt.com/api/v1');
    // SDK row and the Documentation link were removed.
    expect(meta).not.toHaveTextContent('@ai-sdk/openai-compatible');
    expect(screen.queryByTestId('cat-prov-doc-link')).not.toBeInTheDocument();

    const models = await screen.findByTestId('cat-prov-models');
    expect(models).toHaveTextContent('Claude Sonnet 4.5');
  });

  it('rail links: See All Models from Provider → API Models filtered by provider; Add API Model prefills the form', async () => {
    server.use(...mockCatalogProviders(), ...mockCatalogProviderDetail(), ...mockCatalogProviderModels());
    await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-prov-row-nano-gpt'));
    await waitFor(() => expect(screen.getByTestId('cat-prov-detail-nano-gpt')).toBeInTheDocument());

    const allModels = screen.getByTestId('cat-prov-allmodels-nano-gpt');
    expect(allModels).toHaveAttribute('href', expect.stringContaining('/models/explore/api/'));
    expect(decodeURIComponent(allModels.getAttribute('href') ?? '')).toContain('provider=["nano-gpt"]');

    const add = await screen.findByTestId('cat-prov-add-nano-gpt');
    const addHref = decodeURIComponent(add.getAttribute('href') ?? '');
    expect(addHref).toContain('/models/api/new/');
    expect(addHref).toContain('api_format=openai');
    expect(addHref).toContain('base_url=https://nano-gpt.com/api/v1');
    expect(addHref).toContain('name=NanoGPT');
  });

  it('per-model + link prefills the create form with the model id, provider format/name/base_url', async () => {
    server.use(...mockCatalogProviders(), ...mockCatalogProviderDetail(), ...mockCatalogProviderModels());
    await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-prov-row-nano-gpt'));
    await waitFor(() => expect(screen.getByTestId('cat-prov-models')).toBeInTheDocument());

    const add = await screen.findByTestId('cat-prov-model-add-anthropic/claude-sonnet-4.5');
    const href = decodeURIComponent(add.getAttribute('href') ?? '');
    expect(href).toContain('/models/api/new/');
    expect(href).toContain('model=anthropic/claude-sonnet-4.5');
    expect(href).toContain('api_format=openai');
    expect(href).toContain('name=NanoGPT');
    expect(href).toContain('base_url=https://nano-gpt.com/api/v1');
  });

  it('does not fetch detail until a provider is selected (gated)', async () => {
    let detailRequested = false;
    server.use(
      ...mockCatalogProviders(),
      ...mockCatalogProviderDetail({ onRequest: () => (detailRequested = true) }),
      ...mockCatalogProviderModels()
    );
    await renderScreen();
    expect(detailRequested).toBe(false);
    expect(screen.queryByTestId('cat-prov-detail-nano-gpt')).not.toBeInTheDocument();
  });

  it('opens a provider rail on mount from the ?select cross-link', async () => {
    server.use(...mockCatalogProviders(), ...mockCatalogProviderDetail(), ...mockCatalogProviderModels());
    await renderScreen(['/models/explore/providers/?select=nano-gpt']);
    await waitFor(() => expect(screen.getByTestId('cat-prov-detail-nano-gpt')).toBeInTheDocument());
  });

  it('closes the rail via the header close button', async () => {
    server.use(...mockCatalogProviders(), ...mockCatalogProviderDetail(), ...mockCatalogProviderModels());
    await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-prov-row-nano-gpt'));
    await waitFor(() => expect(screen.getByTestId('cat-prov-detail-nano-gpt')).toBeInTheDocument());

    await user.click(screen.getByTestId('cat-prov-detail-close'));
    await waitFor(() => expect(screen.queryByTestId('cat-prov-detail-nano-gpt')).not.toBeInTheDocument());
  });

  it('the rail no longer has per-model context/price/name sort controls', async () => {
    server.use(...mockCatalogProviders(), ...mockCatalogProviderDetail(), ...mockCatalogProviderModels());
    await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-prov-row-nano-gpt'));
    await waitFor(() => expect(screen.getByTestId('cat-prov-models')).toBeInTheDocument());
    expect(screen.queryByTestId('cat-prov-models-sort-context')).not.toBeInTheDocument();
    expect(screen.queryByTestId('cat-prov-models-sort-price')).not.toBeInTheDocument();
    expect(screen.queryByTestId('cat-prov-models-sort-name')).not.toBeInTheDocument();
  });
});

describe('ExploreProvidersScreen (B4 — column picker)', () => {
  it('hides and restores the Format column via the column picker', async () => {
    server.use(...mockCatalogProviders());
    await renderScreen();

    expect(screen.getByTestId('cat-listhead')).toHaveTextContent('FORMAT');

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-prov-columns'));
    await user.click(screen.getByTestId('cat-prov-col-api_format'));
    await waitFor(() => expect(screen.getByTestId('cat-listhead')).not.toHaveTextContent('FORMAT'));

    await user.click(screen.getByTestId('cat-prov-col-api_format'));
    await waitFor(() => expect(screen.getByTestId('cat-listhead')).toHaveTextContent('FORMAT'));
  });
});

describe('ExploreProvidersScreen (B5 — ?select URL sync for the rail)', () => {
  it('clicking a row writes ?select and opens the rail; closing strips it', async () => {
    server.use(...mockCatalogProviders(), ...mockCatalogProviderDetail(), ...mockCatalogProviderModels());
    const router = await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-prov-row-nano-gpt'));
    await waitFor(() => expect(router.state.location.search).toMatchObject({ select: 'nano-gpt' }));
    await waitFor(() => expect(screen.getByTestId('cat-prov-detail-nano-gpt')).toBeInTheDocument());

    await user.click(screen.getByTestId('cat-prov-detail-close'));
    await waitFor(() => expect(router.state.location.search).toEqual({}));
    await waitFor(() => expect(screen.queryByTestId('cat-prov-detail-nano-gpt')).not.toBeInTheDocument());
  });

  it('deep-link ?select= opens the rail on mount', async () => {
    server.use(...mockCatalogProviders(), ...mockCatalogProviderDetail(), ...mockCatalogProviderModels());
    await renderScreen(['/models/explore/providers/?select=nano-gpt']);
    await waitFor(() => expect(screen.getByTestId('cat-prov-detail-nano-gpt')).toBeInTheDocument());
  });

  it('Back restores the pre-selection state and Forward re-applies the selection', async () => {
    server.use(...mockCatalogProviders(), ...mockCatalogProviderDetail(), ...mockCatalogProviderModels());
    const router = await renderScreen();

    const user = userEvent.setup();
    // Apply a facet (push) so there is a history entry to return to, then select a row (replace).
    await user.click(screen.getByTestId('cat-prov-cap-reasoning'));
    await waitFor(() => expect(router.state.location.search).toMatchObject({ capability: ['reasoning'] }));
    await user.click(screen.getByTestId('cat-prov-row-nano-gpt'));
    await waitFor(() => expect(router.state.location.search).toMatchObject({ select: 'nano-gpt' }));

    // Back returns to the facet state with no selection (replace collapsed the selection into the
    // facet entry, so the rail closes); Forward re-applies the whole replaced entry.
    await act(async () => router.history.back());
    await waitFor(() => expect(router.state.location.search).toEqual({}));
    await waitFor(() => expect(screen.queryByTestId('cat-prov-detail-nano-gpt')).not.toBeInTheDocument());

    await act(async () => router.history.forward());
    await waitFor(() => expect(router.state.location.search).toMatchObject({ select: 'nano-gpt' }));
  });

  it('selection uses replace: one Back after several selections skips them entirely', async () => {
    const items = [
      createProviderSummary({ slug: 'nano-gpt', name: 'NanoGPT' }),
      createProviderSummary({ slug: 'openrouter', name: 'OpenRouter' }),
    ];
    server.use(
      ...mockCatalogProviders({ response: createProviderListResponse(items) }),
      ...mockCatalogProviderDetail(),
      ...mockCatalogProviderModels()
    );
    const router = await renderScreen();

    const user = userEvent.setup();
    // Apply a facet (push), then select two different rows (replace each → collapse into the facet entry).
    await user.click(screen.getByTestId('cat-prov-cap-reasoning'));
    await waitFor(() => expect(router.state.location.search).toMatchObject({ capability: ['reasoning'] }));
    await user.click(screen.getByTestId('cat-prov-row-nano-gpt'));
    await waitFor(() => expect(router.state.location.search).toMatchObject({ select: 'nano-gpt' }));
    await user.click(screen.getByTestId('cat-prov-row-openrouter'));
    await waitFor(() => expect(router.state.location.search).toMatchObject({ select: 'openrouter' }));

    // A single Back skips ALL selections (they never pushed), landing before the facet entry.
    await act(async () => router.history.back());
    await waitFor(() => expect(router.state.location.search).toEqual({}));
  });

  it('re-selecting the already-selected row is a no-op (dedup)', async () => {
    server.use(...mockCatalogProviders(), ...mockCatalogProviderDetail(), ...mockCatalogProviderModels());
    const router = await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-prov-row-nano-gpt'));
    await waitFor(() => expect(router.state.location.search).toMatchObject({ select: 'nano-gpt' }));
    const before = router.state.location.search;
    await user.click(screen.getByTestId('cat-prov-row-nano-gpt'));
    expect(router.state.location.search).toBe(before);
  });

  it('changing a facet keeps the selection in the URL', async () => {
    server.use(...mockCatalogProviders(), ...mockCatalogProviderDetail(), ...mockCatalogProviderModels());
    const router = await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-prov-row-nano-gpt'));
    await waitFor(() => expect(router.state.location.search).toMatchObject({ select: 'nano-gpt' }));
    await user.click(screen.getByTestId('cat-prov-cap-reasoning'));
    await waitFor(() =>
      expect(router.state.location.search).toMatchObject({ select: 'nano-gpt', capability: ['reasoning'] })
    );
  });
});

describe('ExploreProvidersScreen (B3 — URL sync: search + sort + facets)', () => {
  it('keeps the URL clean at defaults and writes only non-defaults', async () => {
    server.use(...mockCatalogProviders());
    const router = await renderScreen();
    expect(router.state.location.search).toEqual({});
  });

  it('search submits q on Enter, writes the URL, and resets to page 1', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogProviders({ onRequest: ({ url }) => seen.push(url) }));
    const router = await renderScreen();

    const user = userEvent.setup();
    const input = screen.getByTestId('cat-prov-search').querySelector('input')!;
    await user.click(input);
    await user.type(input, 'nano{Enter}');

    await waitFor(() => expect(router.state.location.search).toMatchObject({ q: 'nano' }));
    await waitFor(() => expect(seen.some((u) => u.searchParams.get('q') === 'nano')).toBe(true));
    expect((router.state.location.search as { page?: number }).page).toBeUndefined();
  });

  it('seeds the search box from the ?q deep-link (the "View" cross-link) and requests it', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogProviders({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen(['/models/explore/providers/?q=NanoGPT']);

    const input = screen.getByTestId('cat-prov-search').querySelector('input')! as HTMLInputElement;
    expect(input.value).toBe('NanoGPT');
    await waitFor(() => expect(seen.some((u) => u.searchParams.get('q') === 'NanoGPT')).toBe(true));
  });

  it('the FORMAT/MODELS column headers sort, mark active, and toggle direction', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogProviders({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-prov-sort-model_count'));
    await waitFor(() => {
      const last = seen[seen.length - 1];
      expect(last.searchParams.get('sort')).toBe('model_count');
      expect(last.searchParams.get('order')).toBe('desc');
    });
    expect(screen.getByTestId('cat-prov-sort-model_count')).toHaveAttribute('data-test-state', 'active');

    await user.click(screen.getByTestId('cat-prov-sort-model_count'));
    await waitFor(() => expect(seen[seen.length - 1].searchParams.get('order')).toBe('asc'));
  });

  it('sorts by api_format from its column header', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogProviders({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-prov-sort-api_format'));
    await waitFor(() => expect(seen.some((u) => u.searchParams.get('sort') === 'api_format')).toBe(true));
  });

  it('rank and cheapest sorts are gone', async () => {
    server.use(...mockCatalogProviders());
    await renderScreen();
    expect(screen.queryByTestId('cat-prov-sort-rank')).not.toBeInTheDocument();
    expect(screen.queryByTestId('cat-prov-sort-pricing')).not.toBeInTheDocument();
  });

  it('a deep-link ?sort= drives the request and Back re-applies the prior state', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogProviders({ onRequest: ({ url }) => seen.push(url) }));
    const router = await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-prov-cap-reasoning'));
    await waitFor(() => expect(router.state.location.search).toMatchObject({ capability: ['reasoning'] }));

    await act(async () => {
      router.history.back();
    });
    await waitFor(() => expect(router.state.location.search).toEqual({}));
    await waitFor(() => expect(seen[seen.length - 1].searchParams.getAll('capability')).toHaveLength(0));
  });

  it('capability + api_format facets send repeated-key params; available values render enabled', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogProviders({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    expect(screen.getByTestId('cat-prov-cap-reasoning')).toBeEnabled();
    // Only api_format values present in the API's facet array are offered (no synthetic options).
    expect(screen.queryByTestId('cat-prov-fmt-openai_responses')).not.toBeInTheDocument();
    expect(screen.queryByTestId('cat-prov-fmt-anthropic_oauth')).not.toBeInTheDocument();
    expect(screen.getByTestId('cat-prov-fmt-anthropic')).toBeEnabled();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-prov-cap-reasoning'));
    await user.click(screen.getByTestId('cat-prov-fmt-anthropic'));

    await waitFor(() => {
      const last = seen[seen.length - 1];
      return (
        last.searchParams.getAll('capability').includes('reasoning') &&
        last.searchParams.getAll('api_format').includes('anthropic')
      );
    });
  });

  it('Labs-only toggle sends is_lab=true and is off by default', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogProviders({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    expect(screen.getByTestId('cat-prov-labs')).toHaveAttribute('aria-pressed', 'false');
    expect(seen[0].searchParams.has('is_lab')).toBe(false);

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-prov-labs'));
    await waitFor(() => expect(seen[seen.length - 1].searchParams.get('is_lab')).toBe('true'));
  });

  it('free/paid toggle sends pricing= and is single-select; no price slider', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogProviders({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    expect(screen.queryByTestId('cat-prov-pricing-range')).not.toBeInTheDocument();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-prov-pricing-free'));
    await waitFor(() => expect(seen[seen.length - 1].searchParams.get('pricing')).toBe('free'));
    expect(screen.getByTestId('cat-prov-pricing-free')).toHaveAttribute('aria-pressed', 'true');

    await user.click(screen.getByTestId('cat-prov-pricing-paid'));
    await waitFor(() => expect(seen[seen.length - 1].searchParams.get('pricing')).toBe('paid'));

    await user.click(screen.getByTestId('cat-prov-pricing-paid'));
    await waitFor(() => expect(seen[seen.length - 1].searchParams.has('pricing')).toBe(false));
  });

  it('reset lives in the toolbar (not the sidebar) and is always visible with three states', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogProviders({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    const user = userEvent.setup();
    const reset = await screen.findByTestId('cat-prov-clear-all');
    expect(screen.getByTestId('cat-prov-facets').contains(reset)).toBe(false);
    expect(reset).toHaveAttribute('data-test-state', 'none');
    expect(reset).toBeDisabled();

    const input = screen.getByTestId('cat-prov-search').querySelector('input')!;
    await user.click(input);
    await user.type(input, 'nano{Enter}');
    await waitFor(() => expect(seen[seen.length - 1].searchParams.get('q')).toBe('nano'));
    await user.click(screen.getByTestId('cat-prov-cap-reasoning'));

    // State 1 (filters): reset clears facets only, keeping the search.
    await waitFor(() => expect(reset).toHaveAttribute('data-test-state', 'filters'));
    await user.click(reset);
    await waitFor(() => {
      const last = seen[seen.length - 1];
      expect(last.searchParams.getAll('capability')).toHaveLength(0);
      expect(last.searchParams.get('q')).toBe('nano');
    });

    // State 2 (query): next click clears the query.
    await waitFor(() => expect(reset).toHaveAttribute('data-test-state', 'query'));
    await user.click(reset);
    await waitFor(() => expect(seen[seen.length - 1].searchParams.get('q')).toBeNull());

    // Back to inert.
    await waitFor(() => expect(reset).toHaveAttribute('data-test-state', 'none'));
    expect(reset).toBeDisabled();
  });

  it('a saved sort preference drives the request on a clean-URL load without writing the URL', async () => {
    localStorage.setItem('bodhi.explore.providers.sort', JSON.stringify({ sort: 'name', order: 'asc' }));
    const seen: URL[] = [];
    server.use(...mockCatalogProviders({ onRequest: ({ url }) => seen.push(url) }));
    const router = await renderScreen();

    await waitFor(() => expect(seen.some((u) => u.searchParams.get('sort') === 'name')).toBe(true));
    expect(router.state.location.search).toEqual({});
  });
});
