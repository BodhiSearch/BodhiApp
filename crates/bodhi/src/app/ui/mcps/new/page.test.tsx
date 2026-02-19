import NewMcpPage from '@/app/ui/mcps/new/page';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import {
  mockAuthHeader,
  mockCreateAuthHeader,
  mockCreateMcp,
  mockFetchMcpTools,
  mockFetchMcpToolsError,
  mockGetAuthHeader,
  mockGetMcp,
  mockListMcpServers,
  mockMcp,
  mockMcpServerResponse,
  mockMcpTool,
  mockMcpWithHeaderAuth,
  mockUpdateAuthHeader,
  mockUpdateMcp,
} from '@/test-utils/msw-v2/handlers/mcps';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
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
      mockCreateAuthHeader(mockAuthHeader),
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

describe('NewMcpPage - Auth type selector', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListMcpServers([mockMcpServerResponse]),
      mockFetchMcpTools([mockMcpTool]),
      mockCreateAuthHeader(mockAuthHeader),
      mockCreateMcp(mockMcp)
    );
  });

  it('renders auth section with default public type', async () => {
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-section')).toBeInTheDocument();
    });

    expect(screen.getByTestId('mcp-auth-type-select')).toBeInTheDocument();
    expect(screen.queryByTestId('mcp-auth-header-key')).not.toBeInTheDocument();
    expect(screen.queryByTestId('mcp-auth-header-value')).not.toBeInTheDocument();
  });

  it('shows header fields when header auth type selected', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-select')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-auth-type-select'));
    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-header')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('mcp-auth-type-header'));

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-header-key')).toBeInTheDocument();
    });
    expect(screen.getByTestId('mcp-auth-header-value')).toBeInTheDocument();
  });

  it('hides header fields when switching back to public', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-select')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-auth-type-select'));
    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-header')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('mcp-auth-type-header'));

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-header-key')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-auth-type-select'));
    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-public')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('mcp-auth-type-public'));

    await waitFor(() => {
      expect(screen.queryByTestId('mcp-auth-header-key')).not.toBeInTheDocument();
    });
    expect(screen.queryByTestId('mcp-auth-header-value')).not.toBeInTheDocument();
  });
});

describe('NewMcpPage - Bearer warning', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockListMcpServers([mockMcpServerResponse]),
      mockFetchMcpTools([mockMcpTool]),
      mockCreateAuthHeader(mockAuthHeader),
      mockCreateMcp(mockMcp)
    );
  });

  it('shows warning when Authorization header value does not start with Bearer', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-select')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-auth-type-select'));
    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-header')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('mcp-auth-type-header'));

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-header-key')).toBeInTheDocument();
    });

    await user.clear(screen.getByTestId('mcp-auth-header-key'));
    await user.type(screen.getByTestId('mcp-auth-header-key'), 'Authorization');
    await user.type(screen.getByTestId('mcp-auth-header-value'), 'sk-12345');

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-bearer-warning')).toBeInTheDocument();
    });
  });

  it('does not show warning when value starts with Bearer', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-select')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-auth-type-select'));
    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-header')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('mcp-auth-type-header'));

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-header-key')).toBeInTheDocument();
    });

    await user.clear(screen.getByTestId('mcp-auth-header-key'));
    await user.type(screen.getByTestId('mcp-auth-header-key'), 'Authorization');
    await user.type(screen.getByTestId('mcp-auth-header-value'), 'Bearer sk-12345');

    expect(screen.queryByTestId('mcp-auth-bearer-warning')).not.toBeInTheDocument();
  });

  it('does not show warning for non-Authorization headers', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-select')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('mcp-auth-type-select'));
    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-type-header')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('mcp-auth-type-header'));

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-header-key')).toBeInTheDocument();
    });

    await user.clear(screen.getByTestId('mcp-auth-header-key'));
    await user.type(screen.getByTestId('mcp-auth-header-key'), 'X-API-Key');
    await user.type(screen.getByTestId('mcp-auth-header-value'), 'sk-12345');

    expect(screen.queryByTestId('mcp-auth-bearer-warning')).not.toBeInTheDocument();
  });
});

describe('NewMcpPage - Edit with header auth', () => {
  beforeEach(() => {
    searchParamsMap = { id: 'mcp-uuid-2' };
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockGetMcp(mockMcpWithHeaderAuth),
      mockGetAuthHeader(mockAuthHeader),
      mockUpdateAuthHeader(mockAuthHeader),
      mockFetchMcpTools([mockMcpTool]),
      mockUpdateMcp(mockMcpWithHeaderAuth)
    );
  });

  it('loads existing header auth and shows auth fields with header key from config', async () => {
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-name-input')).toHaveValue('Header Auth MCP');
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-header-key')).toHaveValue('Authorization');
    });
    expect(screen.getByTestId('mcp-auth-header-value')).toHaveValue('');
  });

  it('shows auth type select with header state on edit', async () => {
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-name-input')).toHaveValue('Header Auth MCP');
    });

    const trigger = screen.getByTestId('mcp-auth-type-select');
    expect(trigger).toHaveAttribute('data-test-state', 'header');
  });

  it('shows placeholder hint about existing header value in edit mode', async () => {
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-header-value')).toBeInTheDocument();
    });

    const input = screen.getByTestId('mcp-auth-header-value');
    expect(input).toHaveAttribute('placeholder', 'Leave empty to keep existing');
  });

  it('toggles header value visibility with eye button', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-auth-header-value')).toBeInTheDocument();
    });

    const input = screen.getByTestId('mcp-auth-header-value');
    expect(input).toHaveAttribute('type', 'password');

    await user.click(screen.getByTestId('mcp-auth-header-value-visibility-toggle'));
    expect(input).toHaveAttribute('type', 'text');

    await user.click(screen.getByTestId('mcp-auth-header-value-visibility-toggle'));
    expect(input).toHaveAttribute('type', 'password');
  });
});

describe('NewMcpPage - Edit with public auth', () => {
  beforeEach(() => {
    searchParamsMap = { id: 'mcp-uuid-1' };
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockGetMcp(mockMcp),
      mockFetchMcpTools([mockMcpTool]),
      mockUpdateMcp(mockMcp)
    );
  });

  it('shows auth type select with public state on edit', async () => {
    await act(async () => {
      render(<NewMcpPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('mcp-name-input')).toHaveValue('Example MCP');
    });

    const trigger = screen.getByTestId('mcp-auth-type-select');
    expect(trigger).toHaveAttribute('data-test-state', 'public');
    expect(screen.queryByTestId('mcp-auth-header-key')).not.toBeInTheDocument();
    expect(screen.queryByTestId('mcp-auth-header-value')).not.toBeInTheDocument();
  });
});
