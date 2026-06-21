import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { ShellSlotsProvider } from '@/components/shell';
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

async function renderScreen() {
  await act(async () => {
    render(
      <ShellSlotsProvider>
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
