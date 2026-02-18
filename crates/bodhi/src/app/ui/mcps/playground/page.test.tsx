/**
 * McpPlaygroundPage Component Tests
 *
 * Purpose: Verify MCP playground page with tool sidebar, form/JSON input, execution, and result display
 *
 * Focus Areas:
 * - Loading and error states
 * - Tool list rendering with whitelisted/non-whitelisted styling
 * - Tool selection and form generation from input_schema
 * - Form/JSON toggle with bidirectional sync
 * - Execute success and error flows
 * - Warning banner for non-whitelisted tools
 * - Refresh tools
 * - Auth redirect
 */

import McpPlaygroundPage from '@/app/ui/mcps/playground/page';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import {
  mockExecuteMcpTool,
  mockExecuteMcpToolError,
  mockGetMcp,
  mockMcp,
  mockMcpTool,
  mockRefreshMcpTools,
} from '@/test-utils/msw-v2/handlers/mcps';
import { mockUserLoggedIn, mockUserLoggedOut } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { McpResponse, McpTool } from '@/hooks/useMcps';

const pushMock = vi.fn();
let mockSearchParams: URLSearchParams;

vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  usePathname: () => '/ui/mcps/playground',
  useSearchParams: () => mockSearchParams,
}));

setupMswV2();

const secondTool: McpTool = {
  name: 'ask_question',
  description: 'Ask a question about a repository',
  input_schema: {
    type: 'object',
    properties: {
      repo_url: { type: 'string', description: 'Repository URL' },
      question: { type: 'string', description: 'Your question' },
    },
    required: ['repo_url', 'question'],
  },
};

const mcpWithTools: McpResponse = {
  ...mockMcp,
  tools_cache: [mockMcpTool, secondTool],
  tools_filter: ['read_wiki_structure'],
};

beforeEach(() => {
  pushMock.mockClear();
  mockSearchParams = new URLSearchParams('id=mcp-uuid-1');
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
      expect(pushMock).toHaveBeenCalledWith('/ui/setup');
    });
  });

  it('redirects to /ui/login if user is not logged in', async () => {
    server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedOut());

    await act(async () => {
      render(<McpPlaygroundPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/login');
    });
  });
});

describe('McpPlaygroundPage - Tool List', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockGetMcp(mcpWithTools)
    );
  });

  it('renders tool list from tools_cache', async () => {
    await act(async () => {
      render(<McpPlaygroundPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-playground-page')).toBeInTheDocument();
    });

    expect(screen.getByTestId('mcp-playground-tool-read_wiki_structure')).toBeInTheDocument();
    expect(screen.getByTestId('mcp-playground-tool-ask_question')).toBeInTheDocument();
  });

  it('shows non-whitelisted tool with reduced opacity', async () => {
    await act(async () => {
      render(<McpPlaygroundPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-playground-page')).toBeInTheDocument();
    });

    const askQuestionTool = screen.getByTestId('mcp-playground-tool-ask_question');
    expect(askQuestionTool).toHaveClass('opacity-50');

    const wikiTool = screen.getByTestId('mcp-playground-tool-read_wiki_structure');
    expect(wikiTool).not.toHaveClass('opacity-50');
  });

  it('shows empty state with no tools message', async () => {
    server.use(mockGetMcp({ ...mockMcp, tools_cache: [], tools_filter: [] }));

    await act(async () => {
      render(<McpPlaygroundPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-playground-page')).toBeInTheDocument();
    });

    expect(screen.getByText('No tools available')).toBeInTheDocument();
  });
});

describe('McpPlaygroundPage - Tool Selection', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockGetMcp(mcpWithTools)
    );
  });

  it('shows tool name and form fields when tool selected', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<McpPlaygroundPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-playground-page')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-playground-tool-read_wiki_structure'));

    expect(screen.getByTestId('mcp-playground-tool-name')).toHaveTextContent('read_wiki_structure');
    expect(screen.getByTestId('mcp-playground-param-repo_name')).toBeInTheDocument();
  });

  it('shows warning banner for non-whitelisted tool', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<McpPlaygroundPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-playground-page')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-playground-tool-ask_question'));

    expect(screen.getByTestId('mcp-playground-not-whitelisted-warning')).toBeInTheDocument();
  });

  it('does not show warning for whitelisted tool', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<McpPlaygroundPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-playground-page')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-playground-tool-read_wiki_structure'));

    expect(screen.queryByTestId('mcp-playground-not-whitelisted-warning')).not.toBeInTheDocument();
  });
});

describe('McpPlaygroundPage - Form/JSON Toggle', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockGetMcp(mcpWithTools)
    );
  });

  it('switches between form and JSON modes', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<McpPlaygroundPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-playground-page')).toBeInTheDocument();
    });

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

    await waitFor(() => {
      expect(screen.getByTestId('mcp-playground-page')).toBeInTheDocument();
    });

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
      mockGetMcp(mcpWithTools)
    );
  });

  it('executes tool and shows success result', async () => {
    const user = userEvent.setup();
    server.use(mockExecuteMcpTool({ result: { wiki: 'content' } }));

    await act(async () => {
      render(<McpPlaygroundPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-playground-page')).toBeInTheDocument();
    });

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
    server.use(mockExecuteMcpToolError({ message: 'Tool not allowed', status: 400 }));

    await act(async () => {
      render(<McpPlaygroundPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-playground-page')).toBeInTheDocument();
    });

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
    server.use(mockExecuteMcpTool({ result: { data: 'test' } }));

    await act(async () => {
      render(<McpPlaygroundPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-playground-page')).toBeInTheDocument();
    });

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
      mockGetMcp(mcpWithTools)
    );
  });

  it('refreshes tools when refresh button clicked', async () => {
    const user = userEvent.setup();
    const newTool: McpTool = { name: 'new_tool', description: 'New tool' };
    server.use(mockRefreshMcpTools([mockMcpTool, secondTool, newTool]));

    await act(async () => {
      render(<McpPlaygroundPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-playground-page')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-playground-refresh-button'));

    await waitFor(() => {
      expect(screen.getByTestId('mcp-playground-refresh-button')).not.toBeDisabled();
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
      mockGetMcp(mcpWithTools)
    );

    await act(async () => {
      render(<McpPlaygroundPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-playground-page')).toBeInTheDocument();
    });

    expect(screen.getByText('Select a tool from the sidebar to get started')).toBeInTheDocument();
  });
});
