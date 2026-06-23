import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { ShellSlotsProvider } from '@/components/shell';
import { ExploreProvidersScreen } from '@/routes/models/explore/api-providers/-components/ExploreProvidersScreen';
import { createProviderListResponse, createProviderSummary } from '@/test-fixtures/catalog-providers';
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import { mockCatalogError, mockCatalogProviders } from '@/test-utils/msw-v2/handlers/reference-catalog';
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

async function renderScreen() {
  await act(async () => {
    render(
      <ShellSlotsProvider>
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

  it('shows "Load more" when more pages remain and appends without duplicates', async () => {
    // 31 providers, page_size 30 → page 1 returns 30, total 31, Load-more visible.
    const items = Array.from({ length: 31 }, (_, i) =>
      createProviderSummary({ slug: `prov-${i}`, name: `Provider ${i}`, rank: i + 1 })
    );
    server.use(...mockCatalogProviders({ response: createProviderListResponse(items) }));
    await renderScreen();

    expect(screen.getByTestId('cat-prov-resultbar')).toHaveTextContent('Showing 30 of 31');
    const loadMore = screen.getByTestId('cat-prov-load-more');

    const user = userEvent.setup();
    await user.click(loadMore);

    await waitFor(() => expect(screen.getByTestId('cat-prov-resultbar')).toHaveTextContent('Showing 31 of 31'));
    // No duplicate rows after appending page 2.
    const list = screen.getByTestId('cat-prov-list');
    expect(within(list).getAllByRole('option').length).toBe(31);
    expect(screen.queryByTestId('cat-prov-load-more')).not.toBeInTheDocument();
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
