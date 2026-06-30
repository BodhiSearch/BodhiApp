import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { AppTokensPage } from '@/routes/tokens/apps/index';
import { mockAppAccessList, mockAppAccessListEmpty, mockAppAccessRevoked } from '@/test-fixtures/apps';
import { mockListAppAccess, mockRevokeAppAccess } from '@/test-utils/msw-v2/handlers/apps';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { ShellHarness } from '@/test-utils/shell-harness';
import { createWrapper } from '@/tests/wrapper';

const navigateMock = vi.fn();
vi.mock('@tanstack/react-router', async () => {
  const actual = await vi.importActual('@tanstack/react-router');
  return { ...actual, useNavigate: () => navigateMock };
});

setupMswV2();

beforeEach(() => {
  navigateMock.mockClear();
  server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));
});

afterEach(() => {
  window.history.replaceState({}, '', '/');
  vi.clearAllMocks();
});

async function renderReady() {
  await act(async () => {
    render(
      <ShellHarness>
        <AppTokensPage />
      </ShellHarness>,
      { wrapper: createWrapper() }
    );
  });
  await waitFor(() => {
    expect(screen.getByTestId('app-tokens-page')).toHaveAttribute('data-pagestatus', 'ready');
  });
}

describe('AppTokensPage - list', () => {
  it('lists granted apps with grant summaries', async () => {
    server.use(...mockListAppAccess(mockAppAccessList));
    await renderReady();

    expect(screen.getByTestId('app-row-app-grant-1')).toBeInTheDocument();
    expect(screen.getByTestId('app-name-app-grant-1')).toHaveTextContent('Research Copilot');
    expect(screen.getByTestId('app-row-app-grant-2')).toBeInTheDocument();
    expect(screen.getByText('1 model')).toBeInTheDocument();
    expect(screen.getByText('All models')).toBeInTheDocument();
  });

  it('shows an empty state when no apps are connected', async () => {
    server.use(...mockListAppAccess(mockAppAccessListEmpty));
    await renderReady();

    expect(screen.getByTestId('app-tokens-empty')).toBeInTheDocument();
  });
});

describe('AppTokensPage - detail rail + revoke', () => {
  it('opens the rail and revokes access on confirm', async () => {
    const user = userEvent.setup();
    server.use(...mockListAppAccess(mockAppAccessList), ...mockRevokeAppAccess(mockAppAccessRevoked));
    await renderReady();

    await user.click(screen.getByTestId('app-row-app-grant-1'));

    const rail = await screen.findByTestId('app-detail-rail');
    expect(rail).toBeInTheDocument();
    expect(screen.getByTestId('app-model-grant-gpt-4o')).toBeInTheDocument();
    expect(screen.getByTestId('app-mcp-grant-mcp-instance-1')).toBeInTheDocument();

    await user.click(screen.getByTestId('app-revoke'));
    await user.click(screen.getByTestId('app-revoke-confirm'));

    await waitFor(() => expect(screen.queryByTestId('app-detail-rail')).not.toBeInTheDocument());
  });
});
