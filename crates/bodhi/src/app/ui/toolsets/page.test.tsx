/**
 * ToolsetsPage Component Tests
 *
 * Purpose: Verify toolsets management page functionality with comprehensive
 * scenario-based testing covering toolset listing and navigation patterns.
 *
 * Focus Areas:
 * - Toolsets list display with status badges
 * - Navigation to toolset configuration page
 * - Authentication and app initialization states
 * - Error handling
 */

import ToolsetsPage from '@/app/ui/toolsets/page';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import { mockAvailableToolsets, mockAvailableToolsetsError } from '@/test-utils/msw-v2/handlers/toolsets';
import { mockUserLoggedIn, mockUserLoggedOut } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

setupMswV2();

beforeEach(() => {
  pushMock.mockClear();
});

afterEach(() => {
  vi.resetAllMocks();
});

describe('ToolsetsPage - Authentication & Initialization', () => {
  it('redirects to /ui/setup if status is setup', async () => {
    server.use(...mockAppInfo({ status: 'setup' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));

    await act(async () => {
      render(<ToolsetsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup');
    });
  });

  it('redirects to /ui/login if user is not logged in', async () => {
    server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedOut());

    await act(async () => {
      render(<ToolsetsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/login');
    });
  });
});

describe('ToolsetsPage - Toolsets List Display', () => {
  beforeEach(() => {
    server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));
  });

  it('displays toolsets list with enabled status badge', async () => {
    server.use(
      ...mockAvailableToolsets([
        {
          toolset_id: 'builtin-exa-web-search',
          name: 'Exa Web Search',
          description: 'Search the web using Exa AI',
          app_enabled: true,
          user_config: {
            enabled: true,
            has_api_key: true,
          },
          tools: [
            {
              type: 'function',
              function: {
                name: 'search',
                description: 'Search the web',
                parameters: { type: 'object', properties: {} },
              },
            },
          ],
        },
      ])
    );

    await act(async () => {
      render(<ToolsetsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('toolsets-page')).toBeInTheDocument();
    });

    expect(screen.getByText('Exa Web Search')).toBeInTheDocument();
    expect(screen.getByText('Enabled')).toBeInTheDocument();
  });

  it('displays toolsets list with configured status badge', async () => {
    server.use(
      ...mockAvailableToolsets([
        {
          toolset_id: 'builtin-exa-web-search',
          name: 'Exa Web Search',
          description: 'Search the web using Exa AI',
          app_enabled: true,
          user_config: {
            enabled: false,
            has_api_key: true,
          },
          tools: [
            {
              type: 'function',
              function: {
                name: 'search',
                description: 'Search the web',
                parameters: { type: 'object', properties: {} },
              },
            },
          ],
        },
      ])
    );

    await act(async () => {
      render(<ToolsetsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('toolsets-page')).toBeInTheDocument();
    });

    expect(screen.getByText('Configured')).toBeInTheDocument();
  });

  it('displays toolsets list with not configured status badge', async () => {
    server.use(
      ...mockAvailableToolsets([
        {
          toolset_id: 'builtin-exa-web-search',
          name: 'Exa Web Search',
          description: 'Search the web using Exa AI',
          app_enabled: true,
          user_config: undefined,
          tools: [
            {
              type: 'function',
              function: {
                name: 'search',
                description: 'Search the web',
                parameters: { type: 'object', properties: {} },
              },
            },
          ],
        },
      ])
    );

    await act(async () => {
      render(<ToolsetsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('toolsets-page')).toBeInTheDocument();
    });

    expect(screen.getByText('Not Configured')).toBeInTheDocument();
  });

  it('displays toolsets list with app disabled status badge', async () => {
    server.use(
      ...mockAvailableToolsets([
        {
          toolset_id: 'builtin-exa-web-search',
          name: 'Exa Web Search',
          description: 'Search the web using Exa AI',
          app_enabled: false,
          user_config: undefined,
          tools: [
            {
              type: 'function',
              function: {
                name: 'search',
                description: 'Search the web',
                parameters: { type: 'object', properties: {} },
              },
            },
          ],
        },
      ])
    );

    await act(async () => {
      render(<ToolsetsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('toolsets-page')).toBeInTheDocument();
    });

    expect(screen.getByText('App Disabled')).toBeInTheDocument();
  });

  it('navigates to edit page when edit button is clicked', async () => {
    const user = userEvent.setup();
    server.use(
      ...mockAvailableToolsets([
        {
          toolset_id: 'builtin-exa-web-search',
          name: 'Exa Web Search',
          description: 'Search the web using Exa AI',
          app_enabled: true,
          user_config: undefined,
          tools: [
            {
              type: 'function',
              function: {
                name: 'search',
                description: 'Search the web',
                parameters: { type: 'object', properties: {} },
              },
            },
          ],
        },
      ])
    );

    await act(async () => {
      render(<ToolsetsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('toolsets-page')).toBeInTheDocument();
    });

    const editButton = screen.getByTestId('toolset-edit-button-builtin-exa-web-search');
    await user.click(editButton);

    expect(pushMock).toHaveBeenCalledWith('/ui/toolsets/edit?toolset_id=builtin-exa-web-search');
  });

  it('displays empty state when no toolsets available', async () => {
    server.use(...mockAvailableToolsets([]));

    await act(async () => {
      render(<ToolsetsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('toolsets-page')).toBeInTheDocument();
    });

    expect(screen.getByText('No toolsets available')).toBeInTheDocument();
  });
});

describe('ToolsetsPage - Error Handling', () => {
  beforeEach(() => {
    server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));
  });

  it('displays error message when toolsets fetch fails', async () => {
    server.use(
      ...mockAvailableToolsetsError({
        message: 'Failed to load toolsets',
        status: 500,
      })
    );

    await act(async () => {
      render(<ToolsetsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByText('Failed to load toolsets')).toBeInTheDocument();
    });
  });
});
