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
  const { rail, railHeader } = useShellSlots();
  return (
    <>
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
    expect(within(rail).getByTestId('cat-mcp-detail-metadata')).toHaveTextContent('mcpservers.org');
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
