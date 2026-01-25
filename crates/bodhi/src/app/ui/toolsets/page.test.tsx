/**
 * ToolsetsPage Component Tests - Instance-based Architecture
 *
 * Purpose: Verify toolsets list page displays instances with UUID-based architecture
 *
 * Focus Areas:
 * - Instance list display with status badges
 * - Navigation to edit page with UUID
 * - Admin tab navigation
 * - Error handling
 */

import ToolsetsPage from '@/app/ui/toolsets/page';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import { mockListToolsets, mockListToolsetsError } from '@/test-utils/msw-v2/handlers/toolsets';
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
  usePathname: () => '/ui/toolsets',
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

describe('ToolsetsPage - Instance List Display', () => {
  beforeEach(() => {
    server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));
  });

  it('displays toolset instance with Enabled status badge', async () => {
    server.use(
      mockListToolsets(
        [
          {
            id: 'uuid-exa-1',
            name: 'my-exa-search',
            scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
            scope: 'scope_toolset-builtin-exa-web-search',
            description: 'Search the web using Exa AI',
            enabled: true,
            has_api_key: true,
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
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
          },
        ],
        [
          {
            scope: 'scope_toolset-builtin-exa-web-search',
            scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
            enabled: true,
            updated_by: 'system',
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
          },
        ]
      )
    );

    await act(async () => {
      render(<ToolsetsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('toolsets-page')).toBeInTheDocument();
    });

    expect(screen.getByText('my-exa-search')).toBeInTheDocument();
    expect(screen.getByText('Enabled')).toBeInTheDocument();
  });

  it('displays toolset instance with Disabled status badge', async () => {
    server.use(
      mockListToolsets(
        [
          {
            id: 'uuid-exa-1',
            name: 'my-exa-search',
            scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
            scope: 'scope_toolset-builtin-exa-web-search',
            description: 'Search the web using Exa AI',
            enabled: false,
            has_api_key: true,
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
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
          },
        ],
        [
          {
            scope: 'scope_toolset-builtin-exa-web-search',
            scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
            enabled: true,
            updated_by: 'system',
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
          },
        ]
      )
    );

    await act(async () => {
      render(<ToolsetsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('toolsets-page')).toBeInTheDocument();
    });

    expect(screen.getByText('my-exa-search')).toBeInTheDocument();
    expect(screen.getByText('Disabled')).toBeInTheDocument();
  });

  it('displays toolset instance with No API Key status badge', async () => {
    server.use(
      mockListToolsets(
        [
          {
            id: 'uuid-exa-1',
            name: 'my-exa-search',
            scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
            scope: 'scope_toolset-builtin-exa-web-search',
            description: 'Search the web using Exa AI',
            enabled: true,
            has_api_key: false,
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
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
          },
        ],
        [
          {
            scope: 'scope_toolset-builtin-exa-web-search',
            scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
            enabled: true,
            updated_by: 'system',
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
          },
        ]
      )
    );

    await act(async () => {
      render(<ToolsetsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('toolsets-page')).toBeInTheDocument();
    });

    expect(screen.getByText('my-exa-search')).toBeInTheDocument();
    expect(screen.getByText('No API Key')).toBeInTheDocument();
  });

  it('displays toolset instance with App Disabled status badge', async () => {
    server.use(
      mockListToolsets(
        [
          {
            id: 'uuid-exa-1',
            name: 'my-exa-search',
            scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
            scope: 'scope_toolset-builtin-exa-web-search',
            description: 'Search the web using Exa AI',
            enabled: true,
            has_api_key: true,
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
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
          },
        ],
        [
          {
            scope: 'scope_toolset-builtin-exa-web-search',
            scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
            enabled: false,
            updated_by: 'system',
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
          },
        ]
      )
    );

    await act(async () => {
      render(<ToolsetsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('toolsets-page')).toBeInTheDocument();
    });

    expect(screen.getByText('my-exa-search')).toBeInTheDocument();
    expect(screen.getByText('App Disabled')).toBeInTheDocument();
  });

  it('displays multiple instances of the same type', async () => {
    server.use(
      mockListToolsets(
        [
          {
            id: 'uuid-exa-1',
            name: 'my-exa-search',
            scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
            scope: 'scope_toolset-builtin-exa-web-search',
            description: 'First instance',
            enabled: true,
            has_api_key: true,
            tools: [],
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
          },
          {
            id: 'uuid-exa-2',
            name: 'company-exa-search',
            scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
            scope: 'scope_toolset-builtin-exa-web-search',
            description: 'Second instance',
            enabled: true,
            has_api_key: true,
            tools: [],
            created_at: '2024-01-02T00:00:00Z',
            updated_at: '2024-01-02T00:00:00Z',
          },
        ],
        [
          {
            scope: 'scope_toolset-builtin-exa-web-search',
            scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
            enabled: true,
            updated_by: 'system',
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
          },
        ]
      )
    );

    await act(async () => {
      render(<ToolsetsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('toolsets-page')).toBeInTheDocument();
    });

    expect(screen.getByText('my-exa-search')).toBeInTheDocument();
    expect(screen.getByText('company-exa-search')).toBeInTheDocument();
  });

  it('navigates to edit page with UUID when edit button is clicked', async () => {
    const user = userEvent.setup();
    server.use(
      mockListToolsets(
        [
          {
            id: 'uuid-exa-1',
            name: 'my-exa-search',
            scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
            scope: 'scope_toolset-builtin-exa-web-search',
            description: 'Search the web using Exa AI',
            enabled: true,
            has_api_key: true,
            tools: [],
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
          },
        ],
        [
          {
            scope: 'scope_toolset-builtin-exa-web-search',
            scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
            enabled: true,
            updated_by: 'system',
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
          },
        ]
      )
    );

    await act(async () => {
      render(<ToolsetsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('toolsets-page')).toBeInTheDocument();
    });

    const editButton = screen.getByTestId('toolset-edit-button-uuid-exa-1');
    await user.click(editButton);

    expect(pushMock).toHaveBeenCalledWith('/ui/toolsets/edit?id=uuid-exa-1');
  });

  it('displays empty state when no toolsets available', async () => {
    server.use(mockListToolsets([]));

    await act(async () => {
      render(<ToolsetsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('toolsets-page')).toBeInTheDocument();
    });

    expect(screen.getByText('No toolsets configured')).toBeInTheDocument();
  });

  it('shows New button for creating toolsets', async () => {
    server.use(mockListToolsets([]));

    await act(async () => {
      render(<ToolsetsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('toolsets-page')).toBeInTheDocument();
    });

    expect(screen.getByTestId('toolset-new-button')).toBeInTheDocument();
  });
});

describe('ToolsetsPage - Admin Features', () => {
  it('shows admin tab for admin users', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({ role: 'resource_admin' }, { stub: true }),
      mockListToolsets([])
    );

    await act(async () => {
      render(<ToolsetsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('toolsets-page')).toBeInTheDocument();
    });

    expect(screen.getByText('Admin')).toBeInTheDocument();
  });

  it('does not show admin tab for regular users', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({ role: 'resource_user' }, { stub: true }),
      mockListToolsets([])
    );

    await act(async () => {
      render(<ToolsetsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('toolsets-page')).toBeInTheDocument();
    });

    expect(screen.queryByText('Admin')).not.toBeInTheDocument();
  });
});

describe('ToolsetsPage - Error Handling', () => {
  beforeEach(() => {
    server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));
  });

  it('displays error message when toolsets fetch fails', async () => {
    server.use(
      mockListToolsetsError({
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
