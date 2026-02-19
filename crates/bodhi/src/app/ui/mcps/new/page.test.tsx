import NewMcpPage from '@/app/ui/mcps/new/page';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import {
  mockCreateMcp,
  mockCreateMcpError,
  mockFetchMcpTools,
  mockFetchMcpToolsError,
  mockGetMcp,
  mockListMcpServers,
  mockMcp,
  mockMcpServerResponse,
  mockMcpTool,
} from '@/test-utils/msw-v2/handlers/mcps';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const pushMock = vi.fn();
let searchParamsMap: Record<string, string | null> = {};

vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  useSearchParams: () => ({
    get: (key: string) => searchParamsMap[key] ?? null,
  }),
  usePathname: () => '/ui/mcps/new',
}));

setupMswV2();

beforeEach(() => {
  pushMock.mockClear();
  searchParamsMap = {};
});

afterEach(() => {
  vi.resetAllMocks();
});

describe('NewMcpPage - Create flow', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListMcpServers([mockMcpServerResponse]),
      mockFetchMcpTools([mockMcpTool]),
      mockCreateMcp(mockMcp)
    );
  });

  it('renders the page with tools section always visible', async () => {
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('new-mcp-page')).toBeInTheDocument();
    });

    expect(screen.getByTestId('mcp-tools-section')).toBeInTheDocument();
    expect(screen.getByTestId('mcp-tools-empty-state')).toHaveTextContent(
      'Select a server and fetch tools to see available tools.'
    );
  });

  it('disables Create MCP button until tools are fetched', async () => {
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-create-button')).toBeInTheDocument();
    });

    expect(screen.getByTestId('mcp-create-button')).toBeDisabled();
  });

  it('enables Create MCP button after fetching tools', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-server-combobox')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-server-combobox'));
    await waitFor(() => {
      expect(screen.getByTestId(`mcp-server-option-${mockMcpServerResponse.id}`)).toBeInTheDocument();
    });
    await user.click(screen.getByTestId(`mcp-server-option-${mockMcpServerResponse.id}`));

    await user.click(screen.getByTestId('mcp-fetch-tools-button'));

    await waitFor(() => {
      expect(screen.getByTestId('mcp-tools-list')).toBeInTheDocument();
    });

    expect(screen.getByTestId('mcp-create-button')).not.toBeDisabled();
  });

  it('shows fetched tools with checkboxes', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-server-combobox')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-server-combobox'));
    await waitFor(() => {
      expect(screen.getByTestId(`mcp-server-option-${mockMcpServerResponse.id}`)).toBeInTheDocument();
    });
    await user.click(screen.getByTestId(`mcp-server-option-${mockMcpServerResponse.id}`));
    await user.click(screen.getByTestId('mcp-fetch-tools-button'));

    await waitFor(() => {
      expect(screen.getByTestId('mcp-tool-read_wiki_structure')).toBeInTheDocument();
    });

    const toolCheckbox = screen.getByTestId('mcp-tool-checkbox-read_wiki_structure');
    expect(toolCheckbox).toBeInTheDocument();
  });

  it('creates MCP with tools data in single POST', async () => {
    const user = userEvent.setup();

    let capturedBody: Record<string, unknown> | null = null;
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListMcpServers([mockMcpServerResponse]),
      mockFetchMcpTools([mockMcpTool]),
      mockCreateMcp(mockMcp)
    );

    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-server-combobox')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-server-combobox'));
    await waitFor(() => {
      expect(screen.getByTestId(`mcp-server-option-${mockMcpServerResponse.id}`)).toBeInTheDocument();
    });
    await user.click(screen.getByTestId(`mcp-server-option-${mockMcpServerResponse.id}`));
    await user.click(screen.getByTestId('mcp-fetch-tools-button'));

    await waitFor(() => {
      expect(screen.getByTestId('mcp-tools-list')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-create-button'));

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/mcps');
    });
  });

  it('shows toast error when fetch tools fails', async () => {
    const user = userEvent.setup();
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListMcpServers([mockMcpServerResponse]),
      mockFetchMcpToolsError({ message: 'Connection refused' })
    );

    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-server-combobox')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-server-combobox'));
    await waitFor(() => {
      expect(screen.getByTestId(`mcp-server-option-${mockMcpServerResponse.id}`)).toBeInTheDocument();
    });
    await user.click(screen.getByTestId(`mcp-server-option-${mockMcpServerResponse.id}`));
    await user.click(screen.getByTestId('mcp-fetch-tools-button'));

    await waitFor(() => {
      expect(screen.getByTestId('mcp-create-button')).toBeDisabled();
    });
  });
});

describe('NewMcpPage - Edit flow', () => {
  beforeEach(() => {
    searchParamsMap = { id: 'mcp-uuid-1' };
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockGetMcp(mockMcp),
      mockFetchMcpTools([mockMcpTool])
    );
  });

  it('loads existing MCP and shows cached tools', async () => {
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('new-mcp-page')).toBeInTheDocument();
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-name-input')).toHaveValue('Example MCP');
    });

    expect(screen.getByTestId('mcp-tools-list')).toBeInTheDocument();
    expect(screen.getByTestId('mcp-tool-read_wiki_structure')).toBeInTheDocument();
  });

  it('shows Update MCP button (not Create) in edit mode', async () => {
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-update-button')).toBeInTheDocument();
    });

    expect(screen.queryByTestId('mcp-create-button')).not.toBeInTheDocument();
  });
});
