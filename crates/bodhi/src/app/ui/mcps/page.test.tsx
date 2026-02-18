/**
 * McpsPage Component Tests
 *
 * Purpose: Verify MCP list page displays instances with status badges
 *
 * Focus Areas:
 * - Instance list display with status badges
 * - Empty state
 * - Error handling
 * - Delete flow
 */

import McpsPage from '@/app/ui/mcps/page';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import { mockListMcps, mockListMcpsError, mockMcp } from '@/test-utils/msw-v2/handlers/mcps';
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
  usePathname: () => '/ui/mcps',
}));

setupMswV2();

beforeEach(() => {
  pushMock.mockClear();
});

afterEach(() => {
  vi.resetAllMocks();
});

describe('McpsPage - Authentication & Initialization', () => {
  it('redirects to /ui/setup if status is setup', async () => {
    server.use(...mockAppInfo({ status: 'setup' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));

    await act(async () => {
      render(<McpsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup');
    });
  });

  it('redirects to /ui/login if user is not logged in', async () => {
    server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedOut());

    await act(async () => {
      render(<McpsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/login');
    });
  });
});

describe('McpsPage - Instance List Display', () => {
  beforeEach(() => {
    server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));
  });

  it('displays MCP instance with Active status badge', async () => {
    server.use(
      mockListMcps([
        {
          ...mockMcp,
          enabled: true,
          tools_filter: ['read_wiki_structure'],
        },
      ])
    );

    await act(async () => {
      render(<McpsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcps-page')).toBeInTheDocument();
    });

    expect(screen.getByText('Example MCP')).toBeInTheDocument();
    expect(screen.getByText('Active')).toBeInTheDocument();
    expect(screen.getByText('1 tool')).toBeInTheDocument();
  });

  it('displays MCP instance with Disabled status badge', async () => {
    server.use(
      mockListMcps([
        {
          ...mockMcp,
          enabled: false,
        },
      ])
    );

    await act(async () => {
      render(<McpsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcps-page')).toBeInTheDocument();
    });

    expect(screen.getByText('Example MCP')).toBeInTheDocument();
    expect(screen.getByText('Disabled')).toBeInTheDocument();
  });

  it('displays MCP instance with No Tools status badge', async () => {
    server.use(
      mockListMcps([
        {
          ...mockMcp,
          enabled: true,
          tools_filter: [],
        },
      ])
    );

    await act(async () => {
      render(<McpsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcps-page')).toBeInTheDocument();
    });

    expect(screen.getByText('Example MCP')).toBeInTheDocument();
    expect(screen.getByText('No Tools')).toBeInTheDocument();
  });

  it('displays empty state when no MCPs available', async () => {
    server.use(mockListMcps([]));

    await act(async () => {
      render(<McpsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcps-page')).toBeInTheDocument();
    });

    expect(screen.getByText('No MCP servers configured')).toBeInTheDocument();
  });

  it('shows Add MCP Server button', async () => {
    server.use(mockListMcps([]));

    await act(async () => {
      render(<McpsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcps-page')).toBeInTheDocument();
    });

    expect(screen.getByTestId('mcp-new-button')).toBeInTheDocument();
  });

  it('navigates to edit page when edit button is clicked', async () => {
    const user = userEvent.setup();
    server.use(mockListMcps([mockMcp]));

    await act(async () => {
      render(<McpsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcps-page')).toBeInTheDocument();
    });

    const editButton = screen.getByTestId('mcp-edit-button-mcp-uuid-1');
    await user.click(editButton);

    expect(pushMock).toHaveBeenCalledWith('/ui/mcps/new?id=mcp-uuid-1');
  });

  it('shows delete confirmation dialog', async () => {
    const user = userEvent.setup();
    server.use(mockListMcps([mockMcp]));

    await act(async () => {
      render(<McpsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcps-page')).toBeInTheDocument();
    });

    const deleteButton = screen.getByTestId('mcp-delete-button-mcp-uuid-1');
    await user.click(deleteButton);

    expect(screen.getByText('Delete MCP Server')).toBeInTheDocument();
    expect(screen.getByText(/Are you sure you want to delete/)).toBeInTheDocument();
  });
});

describe('McpsPage - Error Handling', () => {
  beforeEach(() => {
    server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));
  });

  it('displays error message when MCPs fetch fails', async () => {
    server.use(
      mockListMcpsError({
        message: 'Failed to load MCPs',
        status: 500,
      })
    );

    await act(async () => {
      render(<McpsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByText('Failed to load MCPs')).toBeInTheDocument();
    });
  });
});
