import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { ShellSlotsProvider, useShellSlots } from '@/components/shell';
import { ExploreMcpScreen } from '@/routes/mcps/explore/-components/ExploreMcpScreen';
import { exploreMcpSearchSchema } from '@/routes/mcps/explore/index';
import { createMcpServerSummary, createMcpServersListResponse } from '@/test-fixtures/mcp-catalog';
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import { mockMcpServerDetail, mockMcpServers } from '@/test-utils/msw-v2/handlers/mcp-catalog';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { makeRouteRouter, RouteHarness } from '@/test-utils/router-harness';
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

function buildRouter(initialEntries?: string[]) {
  return makeRouteRouter({
    path: '/mcps/explore/',
    validateSearch: exploreMcpSearchSchema as never,
    Screen: () => (
      <ShellSlotsProvider>
        <SlotsConsumer />
        <ExploreMcpScreen />
      </ShellSlotsProvider>
    ),
    initialEntries,
  });
}

async function renderScreen(initialEntries?: string[]) {
  const router = buildRouter(initialEntries);
  await act(async () => {
    render(<RouteHarness router={router} />, { wrapper: Wrapper });
  });
  await waitFor(() => expect(screen.getByTestId('explore-mcp-content')).toHaveAttribute('data-pagestatus', 'ready'));
  return router;
}

describe('ExploreMcpScreen (Phase 1 — list)', () => {
  it('renders MCP server rows from the catalog', async () => {
    server.use(...mockMcpServers());
    await renderScreen();

    const list = screen.getByTestId('cat-mcp-list');
    expect(within(list).getAllByRole('option').length).toBe(3);
    expect(screen.getByTestId('cat-mcp-row-notion')).toHaveTextContent('Notion');
    expect(screen.getByTestId('cat-mcp-row-notion')).toHaveTextContent('Pages, databases');
    // The Auth column carries the auth_type placeholder.
    expect(screen.getByTestId('cat-mcp-row-notion')).toHaveTextContent('http');
    expect(screen.getByTestId('cat-listhead')).toHaveTextContent('AUTH');
  });

  it('reads the catalog anonymously — no Authorization header', async () => {
    let seenAuth: string | null = 'unset';
    let sawRequest = false;
    server.use(
      ...mockMcpServers({
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

  it('commits search to the URL and forwards q server-side', async () => {
    const seen: URL[] = [];
    server.use(...mockMcpServers({ onRequest: ({ url }) => seen.push(url) }));
    const router = await renderScreen();

    const input = within(screen.getByTestId('cat-mcp-search')).getByRole('textbox');
    await act(async () => {
      await userEvent.type(input, 'notion{enter}');
    });

    await waitFor(() => expect(router.state.location.search).toMatchObject({ q: 'notion' }));
    await waitFor(() => expect(seen.some((u) => u.searchParams.get('q') === 'notion')).toBe(true));
    // Only the matching row remains (server-side filter in the stub).
    await waitFor(() => expect(within(screen.getByTestId('cat-mcp-list')).getAllByRole('option').length).toBe(1));
  });

  it('renders a numbered pager and navigates to page 2', async () => {
    const items = Array.from({ length: 60 }, (_, i) =>
      createMcpServerSummary({ id: `srv-${i}`, slug: `srv-${i}`, name: `Server ${i}` })
    );
    const seen: URL[] = [];
    server.use(
      ...mockMcpServers({ response: createMcpServersListResponse({ items }), onRequest: ({ url }) => seen.push(url) })
    );
    const router = await renderScreen();

    expect(within(screen.getByTestId('cat-mcp-list')).getAllByRole('option').length).toBe(50);
    await act(async () => {
      await userEvent.click(screen.getByRole('button', { name: '2' }));
    });
    await waitFor(() => expect(router.state.location.search).toMatchObject({ page: 2 }));
    await waitFor(() => expect(seen.some((u) => u.searchParams.get('page') === '2')).toBe(true));
  });

  it('shows the empty state when no servers match', async () => {
    server.use(...mockMcpServers({ response: createMcpServersListResponse({ items: [] }) }));
    await renderScreen();
    expect(screen.getByTestId('cat-mcp-empty')).toBeInTheDocument();
  });
});

describe('ExploreMcpScreen (Phase 2 — selection + rail)', () => {
  it('clicking a row writes ?select and opens the rail with connection + metadata', async () => {
    server.use(...mockMcpServers(), ...mockMcpServerDetail());
    const router = await renderScreen();

    await act(async () => {
      await userEvent.click(screen.getByTestId('cat-mcp-row-notion'));
    });
    await waitFor(() => expect(router.state.location.search).toMatchObject({ select: 'notion' }));

    const rail = screen.getByTestId('harness-rail');
    await waitFor(() => expect(within(rail).getByTestId('cat-mcp-detail-connection')).toBeInTheDocument());
    expect(within(rail).getByTestId('cat-mcp-detail-connection')).toHaveTextContent('mcp.notion.com');
    expect(within(rail).getByTestId('cat-mcp-detail-connection')).toHaveTextContent('streamable-http');
    // details (long description) replaces the summary description once detail loads.
    await waitFor(() =>
      expect(within(rail).getByTestId('cat-mcp-detail-description')).toHaveTextContent('Search, read and write')
    );
  });

  it('restores the rail from ?select on load and closing strips the param', async () => {
    server.use(...mockMcpServers(), ...mockMcpServerDetail());
    const router = await renderScreen(['/mcps/explore/?select=notion']);

    const rail = screen.getByTestId('harness-rail');
    expect(within(rail).getByTestId('cat-mcp-detail-notion')).toBeInTheDocument();

    await act(async () => {
      await userEvent.click(screen.getByTestId('cat-mcp-detail-close'));
    });
    await waitFor(() => expect(router.state.location.search).not.toMatchObject({ select: 'notion' }));
    expect(screen.queryByTestId('cat-mcp-detail-notion')).not.toBeInTheDocument();
  });
});

describe('ExploreMcpScreen (Phase 3 — facets + reset + columns)', () => {
  it('renders Auth facet data-driven; hides Category when the facet is empty (v1)', async () => {
    server.use(...mockMcpServers()); // default facets: { category: [], auth: ['http'] }
    await renderScreen();

    const sidebar = screen.getByTestId('harness-sidebar');
    expect(within(sidebar).getByTestId('cat-mcp-auth-http')).toBeInTheDocument();
    // Category group is hidden while facets.category is empty.
    expect(within(sidebar).queryByTestId('cat-mcp-category-Productivity')).not.toBeInTheDocument();
    // Verified pill is always present.
    expect(within(sidebar).getByTestId('cat-mcp-verified')).toBeInTheDocument();
  });

  it('shows Category chips data-driven when the facet is populated', async () => {
    server.use(
      ...mockMcpServers({
        response: createMcpServersListResponse({ facets: { category: ['Productivity', 'Dev Tools'], auth: ['http'] } }),
      })
    );
    await renderScreen();
    const sidebar = screen.getByTestId('harness-sidebar');
    expect(within(sidebar).getByTestId('cat-mcp-category-Productivity')).toBeInTheDocument();
    expect(within(sidebar).getByTestId('cat-mcp-category-Dev Tools')).toBeInTheDocument();
  });

  it('clicking the Auth facet sends ?auth server-side; reset clears it', async () => {
    const seen: URL[] = [];
    server.use(...mockMcpServers({ onRequest: ({ url }) => seen.push(url) }));
    const router = await renderScreen();

    await act(async () => {
      await userEvent.click(within(screen.getByTestId('harness-sidebar')).getByTestId('cat-mcp-auth-http'));
    });
    await waitFor(() => expect(router.state.location.search).toMatchObject({ auth: ['http'] }));
    await waitFor(() => expect(seen.some((u) => u.searchParams.getAll('auth').includes('http'))).toBe(true));

    await act(async () => {
      await userEvent.click(screen.getByTestId('cat-mcp-clear-all'));
    });
    await waitFor(() => expect(router.state.location.search).not.toMatchObject({ auth: ['http'] }));
    expect(screen.getByTestId('cat-mcp-clear-all')).toHaveAttribute('data-test-state', 'none');
  });

  it('Verified filters client-side (no verified API param)', async () => {
    const items = [
      createMcpServerSummary({ id: 'a', slug: 'a', name: 'Alpha', verified: false }),
      createMcpServerSummary({ id: 'b', slug: 'b', name: 'Bravo', verified: true }),
    ];
    const seen: URL[] = [];
    server.use(
      ...mockMcpServers({ response: createMcpServersListResponse({ items }), onRequest: ({ url }) => seen.push(url) })
    );
    await renderScreen();

    expect(within(screen.getByTestId('cat-mcp-list')).getAllByRole('option').length).toBe(2);
    await act(async () => {
      await userEvent.click(within(screen.getByTestId('harness-sidebar')).getByTestId('cat-mcp-verified'));
    });
    // Only the verified row remains; the request carried NO `verified` param (client-side cut).
    await waitFor(() => expect(within(screen.getByTestId('cat-mcp-list')).getAllByRole('option').length).toBe(1));
    expect(screen.getByTestId('cat-mcp-row-b')).toBeInTheDocument();
    expect(seen.every((u) => !u.searchParams.has('verified'))).toBe(true);
  });

  it('column picker hides the Auth column', async () => {
    server.use(...mockMcpServers());
    await renderScreen();
    expect(screen.getByTestId('cat-listhead')).toHaveTextContent('AUTH');

    await act(async () => {
      await userEvent.click(screen.getByTestId('cat-mcp-columns'));
    });
    await act(async () => {
      await userEvent.click(await screen.findByTestId('cat-mcp-col-auth'));
    });
    await waitFor(() => expect(screen.getByTestId('cat-listhead')).not.toHaveTextContent('AUTH'));
  });
});
