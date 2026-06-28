import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { PlaygroundScreen } from '@/routes/mcps/playground/-components/PlaygroundScreen';
import { playgroundSearchSchema } from '@/routes/mcps/playground/index';
import { ChromeProbe, ShellHarness } from '@/test-utils/shell-harness';
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import { mockGetMcp, mockListMcps } from '@/test-utils/msw-v2/handlers/mcps';
import { createMcpProtocolHandlers } from '@/test-utils/msw-v2/handlers/mcp-protocol';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { makeRouteRouter, RouteHarness } from '@/test-utils/router-harness';
import { createWrapper } from '@/tests/wrapper';
import { createMockMcp, createMockMcpServerInfo } from '@/test-fixtures/mcps';

vi.mock('@/hooks/useViewTransition', () => ({ useViewTransition: () => (cb: () => void) => cb() }));

setupMswV2();

let Wrapper: ReturnType<typeof createWrapper>;

const MCP_ID = 'mcp-uuid-1';
const MCP_ENDPOINT = '/bodhi/v1/apps/mcps/mcp-uuid-1/mcp';

const mockInstance = createMockMcp({
  id: MCP_ID,
  name: 'Test MCP',
  mcp_server: createMockMcpServerInfo({
    id: 'server-1',
    name: 'Test Server',
    url: 'https://example.com/mcp',
  }),
});
const mcpForPlayground = { ...mockInstance, path: MCP_ENDPOINT };

const mockTools = [
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

const mockPrompts = [
  {
    name: 'summarize',
    description: 'Summarize a topic',
    arguments: [{ name: 'topic', description: 'Topic to summarize', required: true }],
  },
];

const mockResources = [
  {
    uri: 'file:///docs/readme.md',
    name: 'README',
    mimeType: 'text/markdown',
  },
];

beforeEach(() => {
  localStorage.clear();
  Wrapper = createWrapper();
  server.use(
    ...mockAppInfoReady(),
    ...mockUserLoggedIn({ username: 'admin@example.com', role: 'resource_admin', id_token: 'test-id-token' }),
    mockGetMcp(mcpForPlayground),
    mockListMcps([mockInstance])
  );
});

function buildRouter(initialEntries?: string[]) {
  return makeRouteRouter({
    path: '/mcps/playground/',
    validateSearch: playgroundSearchSchema as never,
    Screen: () => (
      <ShellHarness renderProbe={false}>
        <ChromeProbe />
        <PlaygroundScreen />
      </ShellHarness>
    ),
    initialEntries: initialEntries ?? [`/mcps/playground/?id=${MCP_ID}`],
  });
}

async function renderScreen(initialEntries?: string[]) {
  const router = buildRouter(initialEntries);
  await act(async () => {
    render(<RouteHarness router={router} />, { wrapper: Wrapper });
  });
  await waitFor(() => expect(screen.getByTestId('mcp-playground-page')).toBeInTheDocument(), { timeout: 5000 });
  return router;
}

describe('McpPlaygroundPage — Overview (3-pane)', () => {
  beforeEach(() => {
    server.use(
      ...createMcpProtocolHandlers({
        endpoint: MCP_ENDPOINT,
        tools: mockTools,
        prompts: mockPrompts,
        resources: mockResources,
      })
    );
  });

  it('renders sidebar with instance picker and capability nav', async () => {
    await renderScreen();
    const sidebar = await waitFor(() => screen.getByTestId('mcp-playground-sidebar'));
    expect(within(sidebar).getByTestId('mcp-playground-instance-picker')).toBeInTheDocument();
    expect(within(sidebar).getByTestId('mcp-playground-capability-overview')).toBeInTheDocument();
    expect(within(sidebar).getByTestId('mcp-playground-capability-tools')).toBeInTheDocument();
    expect(within(sidebar).getByTestId('mcp-playground-capability-prompts')).toBeInTheDocument();
    expect(within(sidebar).getByTestId('mcp-playground-capability-resources')).toBeInTheDocument();
  });

  it('shows overview with capability counts after connect', async () => {
    await renderScreen();
    await waitFor(
      () => {
        expect(screen.getByTestId('mcp-playground-capability-count-tools')).toHaveTextContent('2');
      },
      { timeout: 5000 }
    );
    expect(screen.getByTestId('mcp-playground-capability-count-prompts')).toHaveTextContent('1');
    expect(screen.getByTestId('mcp-playground-capability-count-resources')).toHaveTextContent('1');

    expect(screen.getByTestId('mcp-playground-overview')).toBeInTheDocument();
    expect(screen.getByTestId('mcp-playground-overview-card-tools')).toBeInTheDocument();
    expect(screen.getByTestId('mcp-playground-overview-card-prompts')).toBeInTheDocument();
    expect(screen.getByTestId('mcp-playground-overview-card-resources')).toBeInTheDocument();
  });

  it('navigates to tools feature when capability card clicked', async () => {
    const user = userEvent.setup();
    const router = await renderScreen();
    await waitFor(() => expect(screen.getByTestId('mcp-playground-overview-card-tools')).toBeInTheDocument());

    await user.click(screen.getByTestId('mcp-playground-overview-card-tools'));
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ feature: 'tools' });
    });
  });
});

describe('McpPlaygroundPage — Tools', () => {
  beforeEach(() => {
    server.use(
      ...createMcpProtocolHandlers({
        endpoint: MCP_ENDPOINT,
        tools: mockTools,
      })
    );
  });

  it('shows tool list in rail when feature=tools', async () => {
    await renderScreen([`/mcps/playground/?id=${MCP_ID}&feature=tools`]);
    await waitFor(
      () => {
        expect(screen.getByTestId('mcp-playground-rail-item-read_wiki_structure')).toBeInTheDocument();
      },
      { timeout: 5000 }
    );
    expect(screen.getByTestId('mcp-playground-rail-item-ask_question')).toBeInTheDocument();
  });

  it('selecting a rail item shows the ToolDetail', async () => {
    const user = userEvent.setup();
    await renderScreen([`/mcps/playground/?id=${MCP_ID}&feature=tools`]);
    await waitFor(() => screen.getByTestId('mcp-playground-rail-item-read_wiki_structure'));

    await user.click(screen.getByTestId('mcp-playground-rail-item-read_wiki_structure'));
    await waitFor(() => {
      expect(screen.getByTestId('mcp-playground-tool-detail')).toBeInTheDocument();
    });
    expect(screen.getByTestId('mcp-playground-tool-name')).toHaveTextContent('read_wiki_structure');
    expect(screen.getByTestId('mcp-playground-param-repo_name')).toBeInTheDocument();
  });

  it('shows empty rail when no tools advertised', async () => {
    server.use(...createMcpProtocolHandlers({ endpoint: MCP_ENDPOINT, tools: [] }));
    await renderScreen([`/mcps/playground/?id=${MCP_ID}&feature=tools`]);
    await waitFor(() => expect(screen.getByTestId('mcp-playground-rail-empty')).toBeInTheDocument());
  });
});

describe('McpPlaygroundPage — Tool execution', () => {
  beforeEach(() => {
    server.use(
      ...createMcpProtocolHandlers({
        endpoint: MCP_ENDPOINT,
        tools: mockTools,
        toolCallHandler: (toolName) => {
          if (toolName === 'read_wiki_structure') {
            return { text: 'Wiki structure: chapter 1, chapter 2', isError: false };
          }
          if (toolName === 'ask_question') {
            return { text: 'Tool not allowed', isError: true };
          }
          return { text: `Mock result from ${toolName}` };
        },
      })
    );
  });

  it('runs a tool and shows success result', async () => {
    const user = userEvent.setup();
    await renderScreen([`/mcps/playground/?id=${MCP_ID}&feature=tools&item=read_wiki_structure`]);
    await waitFor(() => screen.getByTestId('mcp-playground-tool-detail'));

    // Fill required param
    const paramContainer = screen.getByTestId('mcp-playground-param-repo_name');
    const input = within(paramContainer).getByRole('textbox');
    await user.type(input, 'my-repo');

    await user.click(screen.getByTestId('mcp-playground-run-button'));

    await waitFor(() => {
      const status = screen.getByTestId('mcp-playground-result-status');
      expect(status).toHaveAttribute('data-test-state', 'success');
    });
  });

  it('shows error result when tool returns isError', async () => {
    const user = userEvent.setup();
    await renderScreen([`/mcps/playground/?id=${MCP_ID}&feature=tools&item=ask_question`]);
    await waitFor(() => screen.getByTestId('mcp-playground-tool-detail'));

    // Fill required params
    const repoUrlContainer = screen.getByTestId('mcp-playground-param-repo_url');
    await user.type(within(repoUrlContainer).getByRole('textbox'), 'https://example.com');
    const qContainer = screen.getByTestId('mcp-playground-param-question');
    await user.type(within(qContainer).getByRole('textbox'), 'What is X?');

    await user.click(screen.getByTestId('mcp-playground-run-button'));

    await waitFor(() => {
      const status = screen.getByTestId('mcp-playground-result-status');
      expect(status).toHaveAttribute('data-test-state', 'error');
    });
  });

  it('shows result tabs and allows switching', async () => {
    const user = userEvent.setup();
    await renderScreen([`/mcps/playground/?id=${MCP_ID}&feature=tools&item=read_wiki_structure`]);
    await waitFor(() => screen.getByTestId('mcp-playground-tool-detail'));

    const paramContainer = screen.getByTestId('mcp-playground-param-repo_name');
    await user.type(within(paramContainer).getByRole('textbox'), 'my-repo');
    await user.click(screen.getByTestId('mcp-playground-run-button'));

    await waitFor(() => screen.getByTestId('mcp-playground-result-status'));
    expect(screen.getByTestId('mcp-playground-result-tab-readable')).toBeInTheDocument();
    expect(screen.getByTestId('mcp-playground-result-tab-raw')).toBeInTheDocument();
    expect(screen.getByTestId('mcp-playground-result-tab-request')).toBeInTheDocument();

    await user.click(screen.getByTestId('mcp-playground-result-tab-request'));
    expect(screen.getByTestId('mcp-playground-result-request')).toBeInTheDocument();
  });
});

describe('McpPlaygroundPage — connection status', () => {
  beforeEach(() => {
    server.use(...createMcpProtocolHandlers({ endpoint: MCP_ENDPOINT, tools: mockTools }));
  });

  it('reaches connected state', async () => {
    await renderScreen();
    await waitFor(
      () => {
        expect(screen.getByTestId('mcp-playground-connection-status')).toHaveAttribute('data-test-state', 'connected');
      },
      { timeout: 5000 }
    );
  });
});
