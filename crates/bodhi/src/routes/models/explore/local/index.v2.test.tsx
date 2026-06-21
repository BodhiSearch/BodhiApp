import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { ShellSlotsProvider, useShellSlots } from '@/components/shell';
import { LocalDiscoveryScreen } from '@/routes/models/explore/local/-components/LocalDiscoveryScreen';
import { createListModel } from '@/test-fixtures/discover-models';
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import { mockDiscoverModels } from '@/test-utils/msw-v2/handlers/reference-models';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

vi.mock('@/hooks/useViewTransition', () => ({ useViewTransition: () => (cb: () => void) => cb() }));

setupMswV2();

const ID_TOKEN = 'test-id-token-abc';

// Fresh QueryClient per test — otherwise a cached page (same query key) leaks across tests.
let Wrapper: ReturnType<typeof createWrapper>;

beforeEach(() => {
  Wrapper = createWrapper();
  server.use(
    ...mockAppInfoReady(),
    ...mockUserLoggedIn({ username: 'admin@example.com', role: 'resource_admin', id_token: ID_TOKEN })
  );
});

afterEach(() => {
  vi.clearAllMocks();
});

// Surfaces the published sidebar slot so facet chips are in the DOM under test.
function SlotsConsumer() {
  const { sidebar } = useShellSlots();
  return <div data-testid="harness-sidebar">{sidebar}</div>;
}

async function renderScreen() {
  await act(async () => {
    render(
      <ShellSlotsProvider>
        <SlotsConsumer />
        <LocalDiscoveryScreen />
      </ShellSlotsProvider>,
      { wrapper: Wrapper }
    );
  });
  await waitFor(() =>
    expect(screen.getByTestId('local-discovery-content')).toHaveAttribute('data-pagestatus', 'ready')
  );
}

describe('LocalDiscoveryScreen (Phase 1 — search-only list)', () => {
  it('renders the catalog with "Showing N" (no total) and Downloads/Likes', async () => {
    server.use(...mockDiscoverModels());
    await renderScreen();

    const list = screen.getByTestId('ld-list');
    expect(within(list).getAllByRole('option').length).toBe(3);
    expect(screen.getByTestId('ld-resultbar')).toHaveTextContent('Showing 3');
    // Default sort is Downloads desc.
    expect(screen.getByTestId('ld-resultbar')).toHaveTextContent(/sorted by\s*Downloads\s*·\s*descending/);
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

  it('toggling a sort header sends sort + order', async () => {
    const seen: URL[] = [];
    server.use(...mockDiscoverModels({ onRequest: ({ url }) => seen.push(url) }));
    await renderScreen();

    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-sort-likes'));
    });
    await waitFor(() => {
      const last = seen[seen.length - 1];
      expect(last.searchParams.get('sort')).toBe('likes');
      expect(last.searchParams.get('order')).toBe('desc');
    });

    // Clicking the active column flips the order.
    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-sort-likes'));
    });
    await waitFor(() => {
      const last = seen[seen.length - 1];
      expect(last.searchParams.get('sort')).toBe('likes');
      expect(last.searchParams.get('order')).toBe('asc');
    });
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
    expect(screen.getByTestId('ld-resultbar')).toHaveTextContent('Showing 2');
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
    await waitFor(() => expect(screen.getByTestId('ld-clear-all')).toBeInTheDocument());

    await act(async () => {
      await userEvent.click(screen.getByTestId('ld-clear-all'));
    });
    await waitFor(() => {
      const last = seen[seen.length - 1];
      expect(last.searchParams.getAll('specialisation')).toEqual([]);
      expect(last.searchParams.getAll('license')).toEqual([]);
    });
    expect(screen.queryByTestId('ld-clear-all')).not.toBeInTheDocument();
  });
});
