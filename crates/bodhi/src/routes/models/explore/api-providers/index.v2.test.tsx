import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { ShellSlotsProvider, useShellSlots } from '@/components/shell';
import { ExploreProvidersScreen } from '@/routes/models/explore/api-providers/-components/ExploreProvidersScreen';
import { createProviderListResponse, createProviderSummary } from '@/test-fixtures/catalog-providers';
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

// The screen reads ?select and ?q via useSearch (cross-link entries from the API Models page). No
// router in the RTL wrapper → mock useSearch over a controllable search object (default: empty), so
// each call's `select` projection runs against it. mockSearch lets a test set ?q=… / ?select=….
const mockSearch: Record<string, unknown> = {};
vi.mock('@tanstack/react-router', async () => {
  const actual = await vi.importActual<typeof import('@tanstack/react-router')>('@tanstack/react-router');
  return {
    ...actual,
    useSearch: (opts?: { select?: (s: Record<string, unknown>) => unknown }) =>
      opts?.select ? opts.select(mockSearch) : mockSearch,
  };
});

beforeEach(() => {
  for (const k of Object.keys(mockSearch)) delete mockSearch[k];
});

setupMswV2();

let Wrapper: ReturnType<typeof createWrapper>;

beforeEach(() => {
  Wrapper = createWrapper();
  server.use(
    ...mockAppInfoReady(),
    ...mockUserLoggedIn({ username: 'admin@example.com', role: 'resource_admin', id_token: 'test-id-token' })
  );
});

// Surfaces the published rail slots so the detail rail is in the DOM.
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

async function renderScreen() {
  await act(async () => {
    render(
      <ShellSlotsProvider>
        <SlotsConsumer />
        <ExploreProvidersScreen />
      </ShellSlotsProvider>,
      { wrapper: Wrapper }
    );
  });
  await waitFor(() =>
    expect(screen.getByTestId('explore-providers-content')).toHaveAttribute('data-pagestatus', 'ready')
  );
}

describe('ExploreProvidersScreen (B1 — list)', () => {
  it('renders provider rows with "Showing X of TOTAL" from the catalog', async () => {
    server.use(...mockCatalogProviders());
    await renderScreen();

    const list = screen.getByTestId('cat-prov-list');
    expect(within(list).getAllByRole('option').length).toBe(3);
    expect(screen.getByTestId('cat-prov-resultbar')).toHaveTextContent('Showing 3 of 3');
    expect(screen.getByTestId('cat-prov-row-nano-gpt')).toHaveTextContent('NanoGPT');
    // Model count + capability chips render.
    expect(screen.getByTestId('cat-prov-row-nano-gpt')).toHaveTextContent('617');
    expect(screen.getByTestId('cat-prov-row-nano-gpt')).toHaveTextContent('Reasoning');
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

  it('renders a numbered pager and navigates to page 2', async () => {
    // 31 providers, page_size 30 → page 1 returns 30, total 31, pager visible.
    const items = Array.from({ length: 31 }, (_, i) =>
      createProviderSummary({ slug: `prov-${i}`, name: `Provider ${i}`, rank: i + 1 })
    );
    const seen: URL[] = [];
    server.use(
      ...mockCatalogProviders({ response: createProviderListResponse(items), onRequest: ({ url }) => seen.push(url) })
    );
    await renderScreen();

    expect(screen.getByTestId('cat-prov-resultbar')).toHaveTextContent('Showing 30 of 31');
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
    await act(async () => {
      render(
        <ShellSlotsProvider>
          <ExploreProvidersScreen />
        </ShellSlotsProvider>,
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

    // Rail header names the provider; connection meta + models render from the gated fetches.
    await waitFor(() => expect(screen.getByTestId('cat-prov-detail-nano-gpt')).toBeInTheDocument());
    const meta = await screen.findByTestId('cat-prov-detail-meta');
    expect(meta).toHaveTextContent('NANO_GPT_API_KEY');
    expect(meta).toHaveTextContent('https://nano-gpt.com/api/v1');
    expect(meta).toHaveTextContent('@ai-sdk/openai-compatible');

    const models = await screen.findByTestId('cat-prov-models');
    expect(models).toHaveTextContent('Claude Sonnet 4.5');
    expect(screen.getByTestId('cat-prov-doc-link')).toHaveAttribute('href', 'https://docs.nano-gpt.com');
  });

  it('does not fetch detail until a provider is selected (gated)', async () => {
    let detailRequested = false;
    server.use(
      ...mockCatalogProviders(),
      ...mockCatalogProviderDetail({ onRequest: () => (detailRequested = true) }),
      ...mockCatalogProviderModels()
    );
    await renderScreen();
    // No selection yet → no detail call, no rail.
    expect(detailRequested).toBe(false);
    expect(screen.queryByTestId('cat-prov-detail-nano-gpt')).not.toBeInTheDocument();
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

  it('provider-rail model sort re-queries the served-models endpoint with sort=', async () => {
    const seen: URL[] = [];
    server.use(
      ...mockCatalogProviders(),
      ...mockCatalogProviderDetail(),
      ...mockCatalogProviderModels({ onRequest: ({ url }) => seen.push(url) })
    );
    await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-prov-row-nano-gpt'));
    await waitFor(() => expect(screen.getByTestId('cat-prov-models')).toBeInTheDocument());
    // Default sort is context.
    await waitFor(() => expect(seen.some((u) => u.searchParams.get('sort') === 'context')).toBe(true));

    await user.click(screen.getByTestId('cat-prov-models-sort-price'));
    await waitFor(() => expect(seen.some((u) => u.searchParams.get('sort') === 'price')).toBe(true));
  });
});

describe('ExploreProvidersScreen (B3 — search + sort + facets)', () => {
  it('search submits q on Enter and resets to page 1', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogProviders({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    const user = userEvent.setup();
    const input = screen.getByTestId('cat-prov-search').querySelector('input')!;
    await user.click(input);
    await user.type(input, 'nano{Enter}');

    await waitFor(() => expect(seen.some((u) => u.searchParams.get('q') === 'nano')).toBe(true));
    const last = seen[seen.length - 1];
    expect(last.searchParams.get('page')).toBe('1');
  });

  it('seeds the search box from ?q= (the "View" cross-link) and requests it', async () => {
    mockSearch.q = 'NanoGPT'; // the View link lands here as ?q=<provider name>
    const seen: URL[] = [];
    server.use(...mockCatalogProviders({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    // The search input is pre-filled and the request carries q.
    const input = screen.getByTestId('cat-prov-search').querySelector('input')! as HTMLInputElement;
    expect(input.value).toBe('NanoGPT');
    await waitFor(() => expect(seen.some((u) => u.searchParams.get('q') === 'NanoGPT')).toBe(true));
  });

  it('sort buttons send the chosen sort key + natural order, mark active, and toggle direction', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogProviders({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    const user = userEvent.setup();
    // model_count is naturally descending.
    await user.click(screen.getByTestId('cat-prov-sort-model_count'));
    await waitFor(() => {
      const last = seen[seen.length - 1];
      expect(last.searchParams.get('sort')).toBe('model_count');
      expect(last.searchParams.get('order')).toBe('desc');
    });
    expect(screen.getByTestId('cat-prov-sort-model_count')).toHaveAttribute('data-test-state', 'active');
    expect(screen.getByTestId('cat-prov-resultbar')).toHaveTextContent(/sorted by\s*Models/);

    // Re-click toggles to ascending.
    await user.click(screen.getByTestId('cat-prov-sort-model_count'));
    await waitFor(() => expect(seen[seen.length - 1].searchParams.get('order')).toBe('asc'));
  });

  it('exposes the new pricing + api_format sorts', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogProviders({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    const user = userEvent.setup();
    // pricing (cheapest) is naturally ascending.
    await user.click(screen.getByTestId('cat-prov-sort-pricing'));
    await waitFor(() => {
      const last = seen[seen.length - 1];
      expect(last.searchParams.get('sort')).toBe('pricing');
      expect(last.searchParams.get('order')).toBe('asc');
    });
  });

  it('capability + api_format facets send repeated-key params and counts render', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogProviders({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    // Counts come from the response facets.
    expect(screen.getByTestId('cat-prov-cap-reasoning')).toHaveTextContent('80');
    // api_format buckets absent from the facets map (openai_responses) render disabled.
    expect(screen.getByTestId('cat-prov-fmt-openai_responses')).toBeDisabled();
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

  it('free/paid toggle sends pricing= and is single-select', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogProviders({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-prov-pricing-free'));
    await waitFor(() => expect(seen[seen.length - 1].searchParams.get('pricing')).toBe('free'));
    expect(screen.getByTestId('cat-prov-pricing-free')).toHaveAttribute('aria-pressed', 'true');

    // Switching to paid replaces free (single-select).
    await user.click(screen.getByTestId('cat-prov-pricing-paid'));
    await waitFor(() => expect(seen[seen.length - 1].searchParams.get('pricing')).toBe('paid'));

    // Re-clicking the active value clears it.
    await user.click(screen.getByTestId('cat-prov-pricing-paid'));
    await waitFor(() => expect(seen[seen.length - 1].searchParams.has('pricing')).toBe(false));
  });

  it('clear-all resets every facet param', async () => {
    const seen: URL[] = [];
    server.use(...mockCatalogProviders({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-prov-cap-reasoning'));
    await waitFor(() => expect(screen.getByTestId('cat-prov-clear-all')).toBeInTheDocument());

    await user.click(screen.getByTestId('cat-prov-clear-all'));
    await waitFor(() => {
      const last = seen[seen.length - 1];
      return last.searchParams.getAll('capability').length === 0;
    });
    expect(screen.queryByTestId('cat-prov-clear-all')).not.toBeInTheDocument();
  });
});
