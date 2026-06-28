import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { ShellHarness, ChromeProbe } from '@/test-utils/shell-harness';
import { LocalDiscoveryScreen } from '@/routes/models/explore/local/-components/LocalDiscoveryScreen';
import { localDiscoverySearchSchema } from '@/routes/models/explore/local/index';
import { createListModel } from '@/test-fixtures/discover-models';
import { makeRouteRouter, RouteHarness } from '@/test-utils/router-harness';
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import {
  mockDiscoverModelDetail,
  mockDiscoverModels,
  mockDiscoverModelsError,
} from '@/test-utils/msw-v2/handlers/reference-models';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { mockModelPullDownloads, mockModelPullDownloadsAllSections } from '@/test-utils/msw-v2/handlers/modelfiles';
import { http, HttpResponse, server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

vi.mock('@/hooks/useViewTransition', () => ({ useViewTransition: () => (cb: () => void) => cb() }));

const showSuccess = vi.fn();
const showError = vi.fn();
vi.mock('@/hooks/useToastMessages', () => ({ useToastMessages: () => ({ showSuccess, showError }) }));

setupMswV2();

const ID_TOKEN = 'test-id-token-abc';

// Fresh QueryClient per test — otherwise a cached page (same query key) leaks across tests.
let Wrapper: ReturnType<typeof createWrapper>;

beforeEach(() => {
  Wrapper = createWrapper();
  server.use(
    ...mockAppInfoReady(),
    ...mockUserLoggedIn({ username: 'admin@example.com', role: 'resource_admin', id_token: ID_TOKEN }),
    // Default: empty downloads (stub so it survives polling). Tests override as needed.
    ...mockModelPullDownloads({ data: [], total: 0 }, { stub: true })
  );
});

afterEach(() => {
  vi.clearAllMocks();
});

function ScreenWithSlots() {
  return (
    <>
      <ChromeProbe />
      <LocalDiscoveryScreen />
    </>
  );
}

async function renderScreen(initialEntries?: string[]) {
  const router = makeRouteRouter({
    path: '/models/explore/local/',
    validateSearch: localDiscoverySearchSchema as never,
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
  await waitFor(() =>
    expect(screen.getByTestId('local-discovery-content')).toHaveAttribute('data-pagestatus', 'ready')
  );
  return router;
}

describe('LocalDiscoveryScreen (Phase 1 — search-only list)', () => {
  it('renders the catalog as a table with "Showing N" below the list and the Downloads/Likes columns', async () => {
    server.use(...mockDiscoverModels());
    await renderScreen();

    const list = screen.getByTestId('ld-list');
    expect(within(list).getAllByRole('option').length).toBe(3);
    // The result bar is gone — the count lives below the list, next to Load more.
    expect(screen.queryByTestId('ld-resultbar')).not.toBeInTheDocument();
    expect(screen.getByTestId('ld-count')).toHaveTextContent('Showing 3');
    // Default sort is Downloads (descending-only); the active header carries the state.
    expect(screen.getByTestId('ld-sort-downloads')).toHaveAttribute('data-test-state', 'active');
    expect(screen.getByTestId('ld-row-Qwen-Qwen3-Coder-32B-GGUF')).toBeInTheDocument();
  });

  it('reads the catalog anonymously — no Authorization header (public read-through)', async () => {
    let sawRequest = false;
    let seenAuth: string | null = 'unset';
    server.use(
      ...mockDiscoverModels({
        onRequest: ({ authorization }) => {
          sawRequest = true;
          seenAuth = authorization;
        },
      })
    );
    await renderScreen();
    await waitFor(() => expect(sawRequest).toBe(true));
    // A present-but-wrong-env token would 401; the catalog is public, so we send none.
    expect(seenAuth).toBeNull();
  });

  it('search sends q and disables the cursor (Load more hidden)', async () => {
    const seen: URL[] = [];
    server.use(
      ...mockDiscoverModels({
        items: [
          createListModel({ namespace: 'Qwen', repo: 'Qwen3-Coder-32B-GGUF' }),
          createListModel({ namespace: 'meta-llama', repo: 'Llama-3.3-70B-Instruct-GGUF' }),
        ],
        nextCursor: 'cursor-1',
        onRequest: ({ url }) => seen.push(url),
      })
    );
    await renderScreen();
    // With a cursor available and no search, Load more shows.
    expect(screen.getByTestId('ld-load-more')).toBeInTheDocument();

    const input = within(screen.getByTestId('ld-search')).getByRole('textbox');
    await act(async () => {
      await userEvent.type(input, 'llama{Enter}');
    });

    await waitFor(() => {
      const last = seen[seen.length - 1];
      expect(last.searchParams.get('q')).toBe('llama');
    });
    // Search disables the cursor → only the matching row, no Load more.
    await waitFor(() => {
      expect(screen.queryByTestId('ld-load-more')).not.toBeInTheDocument();
      expect(within(screen.getByTestId('ld-list')).getAllByRole('option').length).toBe(1);
    });
  });

  it('a sort header picks the sort key (descending-only, never sends order=asc)', async () => {
    const seen: URL[] = [];
    server.use(...mockDiscoverModels({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-sort-likes'));
    });
    await waitFor(() => {
      const last = seen[seen.length - 1];
      expect(last.searchParams.get('sort')).toBe('likes');
      // Ascending is unsupported upstream (500s); the UI never sends an order param.
      expect(last.searchParams.get('order')).toBeNull();
    });

    // Re-clicking the active column does not flip to ascending.
    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-sort-likes'));
    });
    await waitFor(() => {
      const last = seen[seen.length - 1];
      expect(last.searchParams.get('sort')).toBe('likes');
      expect(last.searchParams.get('order')).toBeNull();
    });
    expect(screen.getByTestId('ld-sort-likes')).toHaveAttribute('data-test-state', 'active');
  });

  it('renders an Updated column and the Updated header sorts by last_modified', async () => {
    const seen: URL[] = [];
    server.use(
      ...mockDiscoverModels({
        items: [createListModel({ namespace: 'a', repo: 'dated', last_modified: '2025-09-08T00:00:00.000Z' })],
        onRequest: ({ url }) => seen.push(url),
      })
    );
    await renderScreen();

    // The list row surfaces the formatted last_modified date.
    await waitFor(() => expect(screen.getByTestId('ld-row-a-dated')).toHaveTextContent('8 Sep 2025'));

    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-sort-last_modified'));
    });
    await waitFor(() => expect(seen[seen.length - 1].searchParams.get('sort')).toBe('last_modified'));
    expect(screen.getByTestId('ld-sort-last_modified')).toHaveAttribute('data-test-state', 'active');
    expect(screen.getByTestId('cat-listhead')).toHaveTextContent('UPDATED');
  });

  it('renders "—" in the Updated column when last_modified is null', async () => {
    server.use(
      ...mockDiscoverModels({ items: [createListModel({ namespace: 'a', repo: 'undated', last_modified: null })] })
    );
    await renderScreen();
    await waitFor(() => expect(screen.getByTestId('ld-row-a-undated')).toBeInTheDocument());
    expect(screen.getByTestId('ld-row-a-undated')).toHaveTextContent('—');
  });

  it('Load more appends the cursor page', async () => {
    server.use(
      ...mockDiscoverModels({
        items: [createListModel({ namespace: 'a', repo: 'first' })],
        nextCursor: 'cursor-1',
        cursorItems: [createListModel({ namespace: 'b', repo: 'second' })],
      })
    );
    await renderScreen();
    expect(within(screen.getByTestId('ld-list')).getAllByRole('option').length).toBe(1);

    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-load-more'));
    });
    await waitFor(() => {
      expect(within(screen.getByTestId('ld-list')).getAllByRole('option').length).toBe(2);
    });
    expect(screen.getByTestId('ld-count')).toHaveTextContent('Showing 2');
  });
});

describe('LocalDiscoveryScreen (Phase 2a — Browse / Specialisation / Task facets)', () => {
  it('Browse=Trending sets sort=trending', async () => {
    const seen: URL[] = [];
    server.use(...mockDiscoverModels({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-browse-trending'));
    });
    await waitFor(() => expect(seen[seen.length - 1].searchParams.get('sort')).toBe('trending'));
  });

  it('Specialisation chips send specialisation params (repeatable AND)', async () => {
    const seen: URL[] = [];
    server.use(...mockDiscoverModels({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-spec-coding'));
    });
    await waitFor(() => expect(seen[seen.length - 1].searchParams.getAll('specialisation')).toEqual(['coding']));

    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-spec-reasoning'));
    });
    await waitFor(() =>
      expect(seen[seen.length - 1].searchParams.getAll('specialisation').sort()).toEqual(['coding', 'reasoning'])
    );
  });

  it('Task=Image-Text-to-Text sets pipeline_tag; Text Generation (default) omits it', async () => {
    const seen: URL[] = [];
    server.use(...mockDiscoverModels({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();
    // Default load omits pipeline_tag (text-generation is the API default).
    await waitFor(() => expect(seen.length).toBeGreaterThan(0));
    expect(seen[0].searchParams.get('pipeline_tag')).toBeNull();

    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-task-image-text-to-text'));
    });
    await waitFor(() => expect(seen[seen.length - 1].searchParams.get('pipeline_tag')).toBe('image-text-to-text'));

    // Switching back to Text Generation drops the param again.
    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-task-text-generation'));
    });
    await waitFor(() => expect(seen[seen.length - 1].searchParams.get('pipeline_tag')).toBeNull());
  });
});

describe('LocalDiscoveryScreen (Phase 2b/2c — Tag / Language / License / Publisher / clear-all)', () => {
  async function clickAndRead(testId: string, seen: URL[]) {
    await act(async () => {
      await userEvent.click(screen.getByTestId(testId));
    });
    await waitFor(() => expect(seen.length).toBeGreaterThan(0));
    return seen[seen.length - 1];
  }

  it('Tag / Language / License chips send their (repeatable) params', async () => {
    const seen: URL[] = [];
    server.use(...mockDiscoverModels({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    let last = await clickAndRead('ld-tag-moe', seen);
    await waitFor(() => expect((last = seen[seen.length - 1]).searchParams.getAll('tag')).toEqual(['moe']));

    await clickAndRead('ld-lang-en', seen);
    await waitFor(() => expect(seen[seen.length - 1].searchParams.getAll('language')).toEqual(['en']));

    await clickAndRead('ld-license-mit', seen);
    await waitFor(() => expect(seen[seen.length - 1].searchParams.getAll('license')).toEqual(['mit']));
  });

  it('Publisher free-text adds an author chip and sends author', async () => {
    const seen: URL[] = [];
    server.use(...mockDiscoverModels({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    const input = screen.getByTestId('ld-author-input');
    await act(async () => {
      await userEvent.type(input, 'bartowski{Enter}');
    });
    await waitFor(() => expect(seen[seen.length - 1].searchParams.getAll('author')).toEqual(['bartowski']));
    expect(screen.getByTestId('ld-author-chip-bartowski')).toBeInTheDocument();
  });

  it('Clear all filters resets every facet param', async () => {
    const seen: URL[] = [];
    server.use(...mockDiscoverModels({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-spec-coding'));
    });
    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-license-mit'));
    });
    // The toolbar reset is always present; with facets active it's in 'filters' mode.
    await waitFor(() => expect(screen.getByTestId('ld-clear-all')).toHaveAttribute('data-test-state', 'filters'));

    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-clear-all'));
    });
    await waitFor(() => {
      const last = seen[seen.length - 1];
      expect(last.searchParams.getAll('specialisation')).toEqual([]);
      expect(last.searchParams.getAll('license')).toEqual([]);
    });
    // Reset stays in the toolbar but is now inert (nothing to reset).
    expect(screen.getByTestId('ld-clear-all')).toBeDisabled();
  });
});

describe('LocalDiscoveryScreen (Phase 3 — detail rail)', () => {
  it('selecting a row opens the rail and fetches the single-model detail (Overview + quants)', async () => {
    server.use(...mockDiscoverModels(), ...mockDiscoverModelDetail());
    await renderScreen();

    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-row-Qwen-Qwen3-Coder-32B-GGUF'));
    });

    // Rail header shows namespace/repo.
    await waitFor(() =>
      expect(screen.getByTestId('harness-rail-header')).toHaveTextContent('Qwen/Qwen3-Coder-32B-GGUF')
    );

    // Overview specs come from the DETAIL fetch (context/architecture are null on list rows).
    await waitFor(() => expect(screen.getByTestId('ld-detail-specs')).toBeInTheDocument());
    expect(screen.getByTestId('ld-detail-specs')).toHaveTextContent('131,072 tokens');
    expect(screen.getByTestId('ld-detail-specs')).toHaveTextContent('qwen3-moe');

    // Download options tab renders quants from the DTO with null-size "—". The "Recommended"
    // feature is intentionally not surfaced — no badge is rendered.
    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-tab-quants'));
    });
    await waitFor(() => expect(screen.getByTestId('ld-quants')).toBeInTheDocument());
    expect(screen.getByTestId('ld-quant-Q4_K_M')).toBeInTheDocument();
    expect(screen.queryByTestId('ld-quant-rec-Q4_K_M')).not.toBeInTheDocument();
    // Q2_K has a null size in the fixture → renders "—".
    expect(screen.getByTestId('ld-quant-Q2_K')).toHaveTextContent('—');
  });

  it('has no README tab (not surfaced by the v1 API)', async () => {
    server.use(...mockDiscoverModels(), ...mockDiscoverModelDetail());
    await renderScreen();
    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-row-Qwen-Qwen3-Coder-32B-GGUF'));
    });
    await waitFor(() => expect(screen.getByTestId('ld-tab-overview')).toBeInTheDocument());
    expect(screen.queryByText(/README/i)).not.toBeInTheDocument();
  });
});

describe('LocalDiscoveryScreen (Phase 4 — Pull wiring)', () => {
  it('pulling a quant POSTs { repo, filename } to the BodhiApp pull endpoint', async () => {
    let body: { repo?: string; filename?: string } | null = null;
    server.use(
      ...mockDiscoverModels(),
      ...mockDiscoverModelDetail(),
      http.post('*/bodhi/v1/models/files/pull', async ({ request }) => {
        body = (await request.json()) as { repo: string; filename: string };
        return HttpResponse.json({ id: '1', repo: body.repo, filename: body.filename, status: 'pending' });
      })
    );
    await renderScreen();

    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-row-Qwen-Qwen3-Coder-32B-GGUF'));
    });
    // Quants (with their per-row download buttons) live on the Download options tab. The detail
    // rail opens on Overview by default — switch tabs, then download the chosen quant directly.
    await waitFor(() => expect(screen.getByTestId('ld-tab-quants')).toBeInTheDocument());
    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-tab-quants'));
    });
    await waitFor(() => expect(screen.getByTestId('ld-quant-pull-Q4_K_M')).toBeInTheDocument());
    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-quant-pull-Q4_K_M'));
    });

    await waitFor(() => expect(body).not.toBeNull());
    expect(body).toEqual({ repo: 'Qwen/Qwen3-Coder-32B-GGUF', filename: 'Qwen3-Coder-32B-Q4_K_M.gguf' });
    await waitFor(() => expect(showSuccess).toHaveBeenCalled());
  });
});

describe('LocalDiscoveryScreen (Phase 5 — error + empty states)', () => {
  it('renders an error page when the catalog request fails', async () => {
    server.use(...mockDiscoverModelsError({ status: 500, error: 'internal' }));
    const router = makeRouteRouter({
      path: '/models/explore/local/',
      validateSearch: localDiscoverySearchSchema as never,
      Screen: ScreenWithSlots,
    });
    await act(async () => {
      render(
        <ShellHarness renderProbe={false}>
          <RouteHarness router={router} />
        </ShellHarness>,
        { wrapper: Wrapper }
      );
    });
    await waitFor(() => expect(screen.getByText(/Reference API error 500/i)).toBeInTheDocument());
  });

  it('renders the empty state when the catalog returns no matches', async () => {
    server.use(...mockDiscoverModels({ items: [] }));
    await renderScreen();
    await waitFor(() => expect(screen.getByTestId('ld-empty')).toBeInTheDocument());
    // No table, no count footer when there are zero rows.
    expect(screen.queryByTestId('ld-count')).not.toBeInTheDocument();
  });
});

describe('LocalDiscoveryScreen (Phase 6 — Downloads panel)', () => {
  it('opens the Downloads panel and renders all four derived sections', async () => {
    server.use(...mockDiscoverModels(), ...mockModelPullDownloadsAllSections());
    await renderScreen();

    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-downloads-button'));
    });

    await waitFor(() => expect(screen.getByTestId('ld-downloads-panel')).toBeInTheDocument());
    expect(screen.getByTestId('ld-dl-group-downloading')).toBeInTheDocument();
    expect(screen.getByTestId('ld-dl-group-queued')).toBeInTheDocument();
    expect(screen.getByTestId('ld-dl-group-failed')).toBeInTheDocument();
    expect(screen.getByTestId('ld-dl-group-completed')).toBeInTheDocument();

    // Active badge counts downloading + queued only (2), not failed/completed.
    expect(screen.getByTestId('ld-downloads-badge')).toHaveTextContent('2');
  });

  it('toggles the Downloads panel closed on a second click', async () => {
    server.use(...mockDiscoverModels(), ...mockModelPullDownloadsAllSections());
    await renderScreen();

    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-downloads-button'));
    });
    await waitFor(() => expect(screen.getByTestId('ld-downloads-panel')).toBeInTheDocument());

    // Second click collapses the rail → panel removed.
    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-downloads-button'));
    });
    await waitFor(() => expect(screen.queryByTestId('ld-downloads-panel')).not.toBeInTheDocument());
  });

  it('cache-busts the downloads query when the button is clicked', async () => {
    let getCount = 0;
    server.use(
      ...mockDiscoverModels(),
      http.get('*/bodhi/v1/models/files/pull', () => {
        getCount += 1;
        return HttpResponse.json({ data: [], total: 0, page: 1, page_size: 100 });
      })
    );
    await renderScreen();
    const initial = getCount;

    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-downloads-button'));
    });
    await waitFor(() => expect(getCount).toBeGreaterThan(initial));
  });

  it('archives a download — calls the archive endpoint and refetches', async () => {
    let archiveCalled = false;
    server.use(
      ...mockDiscoverModels(),
      ...mockModelPullDownloadsAllSections(),
      http.post('*/bodhi/v1/models/files/pull/:id/archive', ({ params }) => {
        archiveCalled = true;
        return HttpResponse.json({
          id: String(params.id),
          repo: 'microsoft/Phi-4',
          filename: 'phi4.Q4_K_M.gguf',
          status: 'completed',
          error: null,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
          total_bytes: 5_100_000_000,
          downloaded_bytes: 5_100_000_000,
          started_at: '2024-01-01T00:00:00Z',
          archived_at: '2024-01-02T00:00:00Z',
        });
      })
    );
    await renderScreen();
    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-downloads-button'));
    });
    await waitFor(() => expect(screen.getByTestId('ld-dl-archive-dl-completed')).toBeInTheDocument());

    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-dl-archive-dl-completed'));
    });
    await waitFor(() => expect(archiveCalled).toBe(true));
  });

  it('retries a failed download — calls the retry endpoint', async () => {
    let retryCalled = false;
    server.use(
      ...mockDiscoverModels(),
      ...mockModelPullDownloadsAllSections(),
      http.post('*/bodhi/v1/models/files/pull/:id/retry', ({ params }) => {
        retryCalled = true;
        return HttpResponse.json({
          id: String(params.id),
          repo: 'deepseek-ai/DeepSeek-V3',
          filename: 'deepseek.Q2_K.gguf',
          status: 'pending',
          error: null,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-03T00:00:00Z',
          total_bytes: 35_000_000_000,
          downloaded_bytes: 0,
          started_at: null,
          archived_at: null,
        });
      })
    );
    await renderScreen();
    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-downloads-button'));
    });
    await waitFor(() => expect(screen.getByTestId('ld-dl-retry-dl-failed')).toBeInTheDocument());

    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-dl-retry-dl-failed'));
    });
    await waitFor(() => expect(retryCalled).toBe(true));
  });

  it('shows no dismiss (×) button for an actively downloading item', async () => {
    server.use(...mockDiscoverModels(), ...mockModelPullDownloadsAllSections());
    await renderScreen();
    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-downloads-button'));
    });
    await waitFor(() => expect(screen.getByTestId('ld-dl-item-dl-downloading')).toBeInTheDocument());
    // Downloading items expose neither archive nor retry.
    expect(screen.queryByTestId('ld-dl-archive-dl-downloading')).not.toBeInTheDocument();
    expect(screen.queryByTestId('ld-dl-retry-dl-downloading')).not.toBeInTheDocument();
  });
});

describe('LocalDiscoveryScreen — URL state', () => {
  it('a sort header writes ?sort and omits the descending default (no ?order)', async () => {
    server.use(...mockDiscoverModels());
    const router = await renderScreen();

    await userEvent.click(screen.getByTestId('ld-sort-likes'));
    await waitFor(() => expect(router.state.location.search).toMatchObject({ sort: 'likes' }));
    expect(router.state.location.search).not.toHaveProperty('order');
  });

  it('a facet click writes the facet to the URL; Back reverts it', async () => {
    server.use(...mockDiscoverModels());
    const router = await renderScreen();

    await userEvent.click(within(screen.getByTestId('harness-sidebar')).getByTestId('ld-spec-coding'));
    await waitFor(() => expect(router.state.location.search).toMatchObject({ specialisation: ['coding'] }));
    await act(async () => router.history.back());
    await waitFor(() => expect(router.state.location.search).not.toHaveProperty('specialisation'));
  });

  it('row selection writes ?select with replace; deep-link restores the rail', async () => {
    const items = [
      createListModel({ namespace: 'org', repo: 'alpha' }),
      createListModel({ namespace: 'org', repo: 'beta' }),
    ];
    server.use(...mockDiscoverModels({ items }), ...mockDiscoverModelDetail());
    const router = await renderScreen(['/models/explore/local/?select=org%2Falpha']);

    await waitFor(() => expect(router.state.location.search).toMatchObject({ select: 'org/alpha' }));
    // Selecting another row replaces (no new history entry).
    const lengthBefore = router.history.length;
    await userEvent.click(screen.getByTestId('ld-row-org-beta'));
    await waitFor(() => expect(router.state.location.search).toMatchObject({ select: 'org/beta' }));
    expect(router.history.length).toBe(lengthBefore);
  });

  it('Load more accumulates rows without touching the URL', async () => {
    server.use(...mockDiscoverModels({ nextCursor: 'cursor-2' }));
    const router = await renderScreen();

    expect(screen.getByTestId('ld-load-more')).toBeInTheDocument();
    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-load-more'));
    });
    // The cursor is component state, never URL state.
    expect(router.state.location.search).not.toHaveProperty('cursor');
  });

  it('toolbar reset waterfalls facets → query → disabled', async () => {
    server.use(...mockDiscoverModels());
    const router = await renderScreen(['/models/explore/local/?specialisation=%5B%22coding%22%5D&q=qwen']);

    const reset = screen.getByTestId('ld-clear-all');
    await waitFor(() => expect(reset).toHaveAttribute('data-test-state', 'filters'));
    await act(async () => {
      await userEvent.click(reset);
    });
    await waitFor(() => expect(router.state.location.search).not.toHaveProperty('specialisation'));
    await waitFor(() => expect(reset).toHaveAttribute('data-test-state', 'query'));
    await act(async () => {
      await userEvent.click(reset);
    });
    await waitFor(() => expect(router.state.location.search).not.toHaveProperty('q'));
    expect(reset).toBeDisabled();
  });

  it('arrow-down selects the first row (drives ?select)', async () => {
    server.use(...mockDiscoverModels());
    const router = await renderScreen();
    await waitFor(() => expect(screen.getAllByTestId(/^ld-row-/).length).toBeGreaterThan(0));

    await act(async () => {
      document.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }));
    });
    await waitFor(() => expect(router.state.location.search).toHaveProperty('select'));
  });
});
