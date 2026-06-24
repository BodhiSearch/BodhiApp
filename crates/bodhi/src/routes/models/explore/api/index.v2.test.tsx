import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { ShellSlotsProvider, useShellSlots } from '@/components/shell';
import { ExploreApiScreen } from '@/routes/models/explore/api/-components/ExploreApiScreen';
import { createModelDetail, createModelLite, createModelsListResponse } from '@/test-fixtures/catalog-models';
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import {
  mockCatalogError,
  mockCatalogModelDetail,
  mockCatalogModels,
} from '@/test-utils/msw-v2/handlers/reference-catalog';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

vi.mock('@/hooks/useViewTransition', () => ({ useViewTransition: () => (cb: () => void) => cb() }));

// The rail uses TanStack <Link> (no router in the RTL wrapper). Render it as a plain anchor that
// encodes `to` + `search` so tests can assert the Configure-bridge / cross-link targets.
vi.mock('@tanstack/react-router', async () => {
  const actual = await vi.importActual<typeof import('@tanstack/react-router')>('@tanstack/react-router');
  return {
    ...actual,
    Link: ({
      to,
      search,
      children,
      ...rest
    }: {
      to: string;
      search?: Record<string, unknown>;
      children: React.ReactNode;
    }) => {
      const qs = search ? new URLSearchParams(search as Record<string, string>).toString() : '';
      return (
        <a href={qs ? `${to}?${qs}` : to} {...rest}>
          {children}
        </a>
      );
    },
  };
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
        <ExploreApiScreen />
      </ShellSlotsProvider>,
      { wrapper: Wrapper }
    );
  });
  await waitFor(() => expect(screen.getByTestId('explore-api-content')).toHaveAttribute('data-pagestatus', 'ready'));
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
    await renderScreen();

    // Page 1: 30 of 31 rows, pager visible (no Load More).
    expect(screen.getByTestId('cat-model-resultbar')).toHaveTextContent('Showing 30 of 31');
    expect(screen.getByTestId('pagination')).toBeInTheDocument();
    expect(screen.queryByTestId('cat-model-load-more')).not.toBeInTheDocument();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('pagination-page-2'));

    // Page 2: the request carries page=2 and the single remaining row renders.
    await waitFor(() => expect(seen.some((u) => u.searchParams.get('page') === '2')).toBe(true));
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
    await act(async () => {
      render(
        <ShellSlotsProvider>
          <ExploreApiScreen />
        </ShellSlotsProvider>,
        { wrapper: Wrapper }
      );
    });
    await waitFor(() => expect(screen.getByText(/Reference API error 500/i)).toBeInTheDocument());
  });
});

describe('ExploreApiScreen (A2 — detail rail + Configure bridge)', () => {
  it('opens the rail with spec grid + Served-by on row select', async () => {
    server.use(...mockCatalogModels(), ...mockCatalogModelDetail());
    await renderScreen();

    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-model-row-anthropic-claude-sonnet-4.5'));

    await waitFor(() => expect(screen.getByTestId('cat-model-detail-anthropic-claude-sonnet-4.5')).toBeInTheDocument());
    const specs = await screen.findByTestId('cat-model-detail-specs');
    expect(specs).toHaveTextContent('Context');
    expect(specs).toHaveTextContent('200K');
    // Served-by from the detail fetch, deep-linking into the Providers page.
    const servedBy = await screen.findByTestId('cat-model-servedby');
    expect(servedBy).toHaveTextContent('Anthropic');
    expect(screen.getByTestId('cat-model-servedby-openrouter')).toHaveAttribute(
      'href',
      expect.stringContaining('/models/explore/api-providers/?select=openrouter')
    );
  });

  it('synthesizes "Stable" status for a null-status model', async () => {
    server.use(...mockCatalogModels(), ...mockCatalogModelDetail());
    await renderScreen();
    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-model-row-anthropic-claude-sonnet-4.5'));
    const specs = await screen.findByTestId('cat-model-detail-specs');
    expect(specs).toHaveTextContent('Stable');
  });

  it('Configure CTA targets /models/api/new with the bridge api_format + base_url + model', async () => {
    server.use(...mockCatalogModels(), ...mockCatalogModelDetail());
    await renderScreen();
    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-model-row-anthropic-claude-sonnet-4.5'));

    const cta = await screen.findByTestId('cat-model-configure-cta');
    const href = cta.getAttribute('href') ?? '';
    expect(href).toContain('/models/api/new/');
    expect(href).toContain('api_format=anthropic');
    expect(href).toContain('base_url=');
    expect(href).toContain('model=claude-sonnet-4.5');
  });

  it('omits base_url from the bridge when the catalog returns null', async () => {
    server.use(
      ...mockCatalogModels(),
      ...mockCatalogModelDetail({
        detail: createModelDetail({
          bridge: {
            api_format: 'anthropic',
            base_url: null,
            base_url_source: 'user_required',
            base_url_requires_substitution: false,
          },
        }),
      })
    );
    await renderScreen();
    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-model-row-anthropic-claude-sonnet-4.5'));
    const cta = await screen.findByTestId('cat-model-configure-cta');
    const href = cta.getAttribute('href') ?? '';
    expect(href).toContain('api_format=anthropic');
    expect(href).not.toContain('base_url=');
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

  it('Configure shows a substitution note when the bridge base_url needs editing', async () => {
    server.use(
      ...mockCatalogModels(),
      ...mockCatalogModelDetail({
        detail: createModelDetail({
          bridge: {
            api_format: 'openai',
            base_url: 'https://bedrock.{AWS_REGION}.amazonaws.com',
            base_url_source: 'modelsdev_api',
            base_url_requires_substitution: true,
          },
        }),
      })
    );
    await renderScreen();
    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-model-row-anthropic-claude-sonnet-4.5'));
    expect(await screen.findByTestId('cat-model-configure-subst')).toBeInTheDocument();
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
    // The frontend forwards the raw input untouched (no quoting/escaping) + sort=relevance; the
    // catalog FTS5 trigram index does the typo/substring matching server-side.
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

    // Non-zero buckets show their count and stay enabled.
    expect(screen.getByTestId('cat-model-cap-reasoning')).toHaveTextContent('1292');
    expect(screen.getByTestId('cat-model-cap-reasoning')).toBeEnabled();
    // Zero-count buckets are disabled (can't select an empty facet).
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
