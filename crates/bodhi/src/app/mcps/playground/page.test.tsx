/**
 * McpPlaygroundPage Component Tests
 *
 * Purpose: Verify MCP playground page with tool sidebar, form/JSON input, execution, and result display
 *
 * Focus Areas:
 * - Loading and error states
 * - Tool list rendering from MCP client (via real useMcpClient hook + MSW MCP protocol handlers)
 * - Tool selection and form generation from input_schema
 * - Form/JSON toggle with bidirectional sync
 * - Execute success and error flows via MCP SDK client
 * - Connection status display
 *
 * Strategy: Uses MSW MCP protocol handlers (createMcpProtocolHandlers) to simulate
 * the MCP Streamable HTTP server at the protocol level, allowing the real useMcpClient
 * hook and MCP SDK to run end-to-end against mocked HTTP responses.
 */

import McpPlaygroundPage from '@/app/mcps/playground/page';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import { mockGetMcp, mockMcp } from '@/test-utils/msw-v2/handlers/mcps';
import { createMcpProtocolHandlers } from '@/test-utils/msw-v2/handlers/mcp-protocol';
import { mockUserLoggedIn, mockUserLoggedOut } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Mcp } from '@/hooks/mcps';

const navigateMock = vi.fn();
let mockSearch: Record<string, string | undefined> = {};

vi.mock('@tanstack/react-router', async () => {
  const actual = await vi.importActual('@tanstack/react-router');
  return {
    ...actual,
    Link: ({ to, children, ...rest }: any) => (
      <a href={to} {...rest}>
        {children}
      </a>
    ),
    useNavigate: () => navigateMock,
    useLocation: () => ({ pathname: '/mcps/playground' }),
    useSearch: () => mockSearch,
  };
});

setupMswV2();

const MCP_ENDPOINT = '/bodhi/v1/apps/mcps/mcp-uuid-1/mcp';

const mcpTools = [
  {
    name: 'read_wiki_structure',
    description: 'Read the structure of a wiki',
    inputSchema: {
      type: 'object',
      properties: {
        repo_name: { type: 'string', description: 'Repository name' },
      },
      required: ['repo_name'],
    },
  },
  {
    name: 'ask_question',
    description: 'Ask a question about a repository',
    inputSchema: {
      type: 'object',
      properties: {
        repo_url: { type: 'string', description: 'Repository URL' },
        question: { type: 'string', description: 'Your question' },
      },
      required: ['repo_url', 'question'],
    },
  },
];

const mcpForPlayground: Mcp = {
  ...mockMcp,
  path: MCP_ENDPOINT,
};

beforeEach(() => {
  navigateMock.mockClear();
  mockSearch = { id: 'mcp-uuid-1' };
});

afterEach(() => {
  vi.resetAllMocks();
});

describe('McpPlaygroundPage - Authentication', () => {
  it('redirects to /ui/setup if status is setup', async () => {
    server.use(...mockAppInfo({ status: 'setup' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));

    await act(async () => {
      render(<McpPlaygroundPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(navigateMock).toHaveBeenCalledWith({ to: '/setup' });
    });
  });

  it('redirects to /ui/login if user is not logged in', async () => {
    server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedOut());

    await act(async () => {
      render(<McpPlaygroundPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(navigateMock).toHaveBeenCalledWith({ to: '/login' });
    });
  });
});

describe('McpPlaygroundPage - Tool List', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockGetMcp(mcpForPlayground),
      ...createMcpProtocolHandlers({
        endpoint: MCP_ENDPOINT,
        tools: mcpTools,
      })
    );
  });

  it('renders tool list from MCP client', async () => {
    await act(async () => {
      render(<McpPlaygroundPage />, { wrapper: createWrapper() });
    });

    await waitFor(
      () => {
        expect(screen.getByTestId('mcp-playground-tool-read_wiki_structure')).toBeInTheDocument();
      },
      { timeout: 5000 }
    );

    expect(screen.getByTestId('mcp-playground-tool-ask_question')).toBeInTheDocument();
  });

  it('shows empty state with no tools message', async () => {
    // Override with no tools
    server.use(
      ...createMcpProtocolHandlers({
        endpoint: MCP_ENDPOINT,
        tools: [],
      })
    );

    await act(async () => {
      render(<McpPlaygroundPage />, { wrapper: createWrapper() });
    });

    await waitFor(
      () => {
        expect(screen.getByTestId('mcp-playground-page')).toBeInTheDocument();
      },
      { timeout: 5000 }
    );

    // Wait for connection to complete (no tools)
    await waitFor(() => {
      expect(screen.getByText('No tools available')).toBeInTheDocument();
    });
  });
});

describe('McpPlaygroundPage - Tool Selection', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockGetMcp(mcpForPlayground),
      ...createMcpProtocolHandlers({
        endpoint: MCP_ENDPOINT,
        tools: mcpTools,
      })
    );
  });

  it('shows tool name and form fields when tool selected', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<McpPlaygroundPage />, { wrapper: createWrapper() });
    });

    await waitFor(
      () => {
        expect(screen.getByTestId('mcp-playground-tool-read_wiki_structure')).toBeInTheDocument();
      },
      { timeout: 5000 }
    );

    await user.click(screen.getByTestId('mcp-playground-tool-read_wiki_structure'));

    expect(screen.getByTestId('mcp-playground-tool-name')).toHaveTextContent('read_wiki_structure');
    expect(screen.getByTestId('mcp-playground-param-repo_name')).toBeInTheDocument();
  });
});

describe('McpPlaygroundPage - Form/JSON Toggle', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockGetMcp(mcpForPlayground),
      ...createMcpProtocolHandlers({
        endpoint: MCP_ENDPOINT,
        tools: mcpTools,
      })
    );
  });

  it('switches between form and JSON modes', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<McpPlaygroundPage />, { wrapper: createWrapper() });
    });

    await waitFor(
      () => {
        expect(screen.getByTestId('mcp-playground-tool-ask_question')).toBeInTheDocument();
      },
      { timeout: 5000 }
    );

    await user.click(screen.getByTestId('mcp-playground-tool-ask_question'));

    expect(screen.getByTestId('mcp-playground-param-repo_url')).toBeInTheDocument();
    expect(screen.queryByTestId('mcp-playground-json-editor')).not.toBeInTheDocument();

    await user.click(screen.getByTestId('mcp-playground-input-mode-json'));

    expect(screen.getByTestId('mcp-playground-json-editor')).toBeInTheDocument();
    expect(screen.queryByTestId('mcp-playground-param-repo_url')).not.toBeInTheDocument();

    await user.click(screen.getByTestId('mcp-playground-input-mode-form'));

    expect(screen.getByTestId('mcp-playground-param-repo_url')).toBeInTheDocument();
  });

  it('syncs form changes to JSON editor', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<McpPlaygroundPage />, { wrapper: createWrapper() });
    });

    await waitFor(
      () => {
        expect(screen.getByTestId('mcp-playground-tool-ask_question')).toBeInTheDocument();
      },
      { timeout: 5000 }
    );

    await user.click(screen.getByTestId('mcp-playground-tool-ask_question'));

    const paramContainer = screen.getByTestId('mcp-playground-param-repo_url');
    const input = within(paramContainer).getByRole('textbox');
    await user.type(input, 'https://github.com/test');

    await user.click(screen.getByTestId('mcp-playground-input-mode-json'));

    const jsonEditor = screen.getByTestId('mcp-playground-json-editor') as HTMLTextAreaElement;
    expect(jsonEditor.value).toContain('https://github.com/test');
  });
});

describe('McpPlaygroundPage - Execute', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockGetMcp(mcpForPlayground),
      ...createMcpProtocolHandlers({
        endpoint: MCP_ENDPOINT,
        tools: mcpTools,
        toolCallHandler: (toolName, _args) => {
          if (toolName === 'read_wiki_structure') {
            return { text: JSON.stringify({ wiki: 'content' }), isError: false };
          }
          if (toolName === 'ask_question') {
            return { text: 'Tool not allowed', isError: true };
          }
          return { text: `Mock result from ${toolName}` };
        },
      })
    );
  });

  it('executes tool and shows success result', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<McpPlaygroundPage />, { wrapper: createWrapper() });
    });

    await waitFor(
      () => {
        expect(screen.getByTestId('mcp-playground-tool-read_wiki_structure')).toBeInTheDocument();
      },
      { timeout: 5000 }
    );

    await user.click(screen.getByTestId('mcp-playground-tool-read_wiki_structure'));
    await user.click(screen.getByTestId('mcp-playground-execute-button'));

    await waitFor(() => {
      expect(screen.getByTestId('mcp-playground-result-section')).toBeInTheDocument();
    });

    const status = screen.getByTestId('mcp-playground-result-status');
    expect(status).toHaveAttribute('data-test-state', 'success');
  });

  it('shows error result on execution failure', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<McpPlaygroundPage />, { wrapper: createWrapper() });
    });

    await waitFor(
      () => {
        expect(screen.getByTestId('mcp-playground-tool-ask_question')).toBeInTheDocument();
      },
      { timeout: 5000 }
    );

    await user.click(screen.getByTestId('mcp-playground-tool-ask_question'));
    await user.click(screen.getByTestId('mcp-playground-execute-button'));

    await waitFor(() => {
      expect(screen.getByTestId('mcp-playground-result-section')).toBeInTheDocument();
    });

    const status = screen.getByTestId('mcp-playground-result-status');
    expect(status).toHaveAttribute('data-test-state', 'error');
  });

  it('shows result tabs and allows switching', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<McpPlaygroundPage />, { wrapper: createWrapper() });
    });

    await waitFor(
      () => {
        expect(screen.getByTestId('mcp-playground-tool-read_wiki_structure')).toBeInTheDocument();
      },
      { timeout: 5000 }
    );

    await user.click(screen.getByTestId('mcp-playground-tool-read_wiki_structure'));
    await user.click(screen.getByTestId('mcp-playground-execute-button'));

    await waitFor(() => {
      expect(screen.getByTestId('mcp-playground-result-section')).toBeInTheDocument();
    });

    expect(screen.getByTestId('mcp-playground-result-tab-response')).toBeInTheDocument();
    expect(screen.getByTestId('mcp-playground-result-tab-raw')).toBeInTheDocument();
    expect(screen.getByTestId('mcp-playground-result-tab-request')).toBeInTheDocument();

    await user.click(screen.getByTestId('mcp-playground-result-tab-request'));
    const content = screen.getByTestId('mcp-playground-result-content');
    expect(content.textContent).toContain('read_wiki_structure');
  });
});

describe('McpPlaygroundPage - Refresh', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockGetMcp(mcpForPlayground),
      ...createMcpProtocolHandlers({
        endpoint: MCP_ENDPOINT,
        tools: mcpTools,
      })
    );
  });

  it('calls refreshTools when refresh button clicked', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<McpPlaygroundPage />, { wrapper: createWrapper() });
    });

    await waitFor(
      () => {
        expect(screen.getByTestId('mcp-playground-tool-read_wiki_structure')).toBeInTheDocument();
      },
      { timeout: 5000 }
    );

    // The refresh button triggers a tools/list call via the MCP SDK.
    // After click, the tools should still be present (re-fetched from MSW handler).
    await user.click(screen.getByTestId('mcp-playground-refresh-button'));

    // Verify tools are still listed after refresh (the MSW handler returns the same tools)
    await waitFor(() => {
      expect(screen.getByTestId('mcp-playground-tool-read_wiki_structure')).toBeInTheDocument();
      expect(screen.getByTestId('mcp-playground-tool-ask_question')).toBeInTheDocument();
    });
  });
});

describe('McpPlaygroundPage - Loading and Error', () => {
  it('shows loading skeleton', async () => {
    server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));

    await act(async () => {
      render(<McpPlaygroundPage />, { wrapper: createWrapper() });
    });

    expect(screen.getByTestId('mcp-playground-loading')).toBeInTheDocument();
  });

  it('shows prompt to select tool when none selected', async () => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockGetMcp(mcpForPlayground),
      ...createMcpProtocolHandlers({
        endpoint: MCP_ENDPOINT,
        tools: mcpTools,
      })
    );

    await act(async () => {
      render(<McpPlaygroundPage />, { wrapper: createWrapper() });
    });

    await waitFor(
      () => {
        expect(screen.getByTestId('mcp-playground-page')).toBeInTheDocument();
      },
      { timeout: 5000 }
    );

    expect(screen.getByText('Select a tool from the sidebar to get started')).toBeInTheDocument();
  });
});
