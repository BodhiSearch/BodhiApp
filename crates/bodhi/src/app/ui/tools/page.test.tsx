/**
 * ToolsPage Component Tests
 *
 * Purpose: Verify tools management page functionality with comprehensive
 * scenario-based testing covering tool listing and navigation patterns.
 *
 * Focus Areas:
 * - Tools list display with status badges
 * - Navigation to tool configuration page
 * - Authentication and app initialization states
 * - Error handling
 */

import ToolsPage from '@/app/ui/tools/page';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import { mockAvailableTools, mockAvailableToolsError } from '@/test-utils/msw-v2/handlers/tools';
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

describe('ToolsPage - Authentication & Initialization', () => {
  it('redirects to /ui/setup if status is setup', async () => {
    server.use(...mockAppInfo({ status: 'setup' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));

    await act(async () => {
      render(<ToolsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup');
    });
  });

  it('redirects to /ui/login if user is not logged in', async () => {
    server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedOut());

    await act(async () => {
      render(<ToolsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/login');
    });
  });
});

describe('ToolsPage - Tools List Display', () => {
  beforeEach(() => {
    server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));
  });

  it('displays tools list with enabled status badge', async () => {
    server.use(
      ...mockAvailableTools([
        {
          type: 'function',
          function: {
            name: 'builtin-exa-web-search',
            description: 'Search the web using Exa AI',
            parameters: { type: 'object', properties: {} },
          },
          app_enabled: true,
          user_config: {
            enabled: true,
            has_api_key: true,
          },
        },
      ])
    );

    await act(async () => {
      render(<ToolsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('tools-page')).toBeInTheDocument();
    });

    expect(screen.getByText('Exa Web Search')).toBeInTheDocument();
    expect(screen.getByText('Enabled')).toBeInTheDocument();
  });

  it('displays tools list with configured status badge', async () => {
    server.use(
      ...mockAvailableTools([
        {
          type: 'function',
          function: {
            name: 'builtin-exa-web-search',
            description: 'Search the web using Exa AI',
            parameters: { type: 'object', properties: {} },
          },
          app_enabled: true,
          user_config: {
            enabled: false,
            has_api_key: true,
          },
        },
      ])
    );

    await act(async () => {
      render(<ToolsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('tools-page')).toBeInTheDocument();
    });

    expect(screen.getByText('Configured')).toBeInTheDocument();
  });

  it('displays tools list with not configured status badge', async () => {
    server.use(
      ...mockAvailableTools([
        {
          type: 'function',
          function: {
            name: 'builtin-exa-web-search',
            description: 'Search the web using Exa AI',
            parameters: { type: 'object', properties: {} },
          },
          app_enabled: true,
          user_config: undefined,
        },
      ])
    );

    await act(async () => {
      render(<ToolsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('tools-page')).toBeInTheDocument();
    });

    expect(screen.getByText('Not Configured')).toBeInTheDocument();
  });

  it('displays tools list with app disabled status badge', async () => {
    server.use(
      ...mockAvailableTools([
        {
          type: 'function',
          function: {
            name: 'builtin-exa-web-search',
            description: 'Search the web using Exa AI',
            parameters: { type: 'object', properties: {} },
          },
          app_enabled: false,
          user_config: undefined,
        },
      ])
    );

    await act(async () => {
      render(<ToolsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('tools-page')).toBeInTheDocument();
    });

    expect(screen.getByText('App Disabled')).toBeInTheDocument();
  });

  it('navigates to edit page when edit button is clicked', async () => {
    const user = userEvent.setup();
    server.use(
      ...mockAvailableTools([
        {
          type: 'function',
          function: {
            name: 'builtin-exa-web-search',
            description: 'Search the web using Exa AI',
            parameters: { type: 'object', properties: {} },
          },
          app_enabled: true,
          user_config: undefined,
        },
      ])
    );

    await act(async () => {
      render(<ToolsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('tools-page')).toBeInTheDocument();
    });

    const editButton = screen.getByTestId('tool-edit-button-builtin-exa-web-search');
    await user.click(editButton);

    expect(pushMock).toHaveBeenCalledWith('/ui/tools/edit?toolid=builtin-exa-web-search');
  });

  it('displays empty state when no tools available', async () => {
    server.use(...mockAvailableTools([]));

    await act(async () => {
      render(<ToolsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('tools-page')).toBeInTheDocument();
    });

    expect(screen.getByText('No tools available')).toBeInTheDocument();
  });
});

describe('ToolsPage - Error Handling', () => {
  beforeEach(() => {
    server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));
  });

  it('displays error message when tools fetch fails', async () => {
    server.use(
      ...mockAvailableToolsError({
        message: 'Failed to load tools',
        status: 500,
      })
    );

    await act(async () => {
      render(<ToolsPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByText('Failed to load tools')).toBeInTheDocument();
    });
  });
});
