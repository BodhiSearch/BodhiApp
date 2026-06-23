import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { ShellSlotsProvider } from '@/components/shell';
import { ExploreApiScreen } from '@/routes/models/explore/api/-components/ExploreApiScreen';
import { createModelLite, createModelsListResponse } from '@/test-fixtures/catalog-models';
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import { mockCatalogError, mockCatalogModels } from '@/test-utils/msw-v2/handlers/reference-catalog';
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

  it('shows "Load more" and appends the next page without duplicates', async () => {
    const items = Array.from({ length: 31 }, (_, i) =>
      createModelLite({ slug: 'p', model_id: `m-${i}`, name: `Model ${i}` })
    );
    server.use(...mockCatalogModels({ response: createModelsListResponse(items) }));
    await renderScreen();

    expect(screen.getByTestId('cat-model-resultbar')).toHaveTextContent('Showing 30 of 31');
    const user = userEvent.setup();
    await user.click(screen.getByTestId('cat-model-load-more'));

    await waitFor(() => expect(screen.getByTestId('cat-model-resultbar')).toHaveTextContent('Showing 31 of 31'));
    expect(within(screen.getByTestId('cat-model-list')).getAllByRole('option').length).toBe(31);
    expect(screen.queryByTestId('cat-model-load-more')).not.toBeInTheDocument();
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
