import { McpServersPane } from '@/routes/chat/-components/settings/McpServersPane';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

vi.mock('@tanstack/react-router', () => ({
  Link: ({ to, children, ...rest }: { to: string; children: React.ReactNode }) => (
    <a href={to} {...rest}>
      {children}
    </a>
  ),
}));

type Mcp = Parameters<typeof McpServersPane>[0]['mcps'][number];

const mcp = (over: Partial<Mcp> = {}): Mcp =>
  ({
    id: 'm1',
    slug: 'filesystem',
    enabled: true,
    path: '/mcp',
    mcp_server: { enabled: true },
    ...over,
  }) as Mcp;

const tools = new Map([['m1', [{ name: 'read_file' }, { name: 'write_file' }] as never]]);
const status = new Map();

function renderPane(props: Partial<Parameters<typeof McpServersPane>[0]> = {}) {
  return render(
    <McpServersPane
      mcps={[mcp()]}
      enabledMcpTools={{}}
      onToggleTool={vi.fn()}
      onToggleMcp={vi.fn()}
      mcpTools={tools}
      mcpConnectionStatus={status}
      {...props}
    />
  );
}

describe('McpServersPane', () => {
  it('renders the empty state with a settings link when there are no MCPs', () => {
    renderPane({ mcps: [] });
    expect(screen.getByTestId('mcps-empty-state')).toBeInTheDocument();
    expect(screen.getByTestId('mcps-settings-link')).toHaveAttribute('href', '/mcps/');
  });

  it('lists a configured server and expands its tools', async () => {
    const user = userEvent.setup();
    renderPane();

    expect(screen.getByTestId('mcp-row-m1')).toBeInTheDocument();
    expect(screen.queryByTestId('mcp-tool-row-m1-read_file')).not.toBeInTheDocument();

    await user.click(screen.getByTestId('mcp-expand-m1'));
    expect(screen.getByTestId('mcp-tool-checkbox-m1-read_file')).toBeInTheDocument();
    expect(screen.getByTestId('mcp-tool-checkbox-m1-write_file')).toBeInTheDocument();
  });

  it('toggles all tools via the server checkbox', async () => {
    const user = userEvent.setup();
    const onToggleMcp = vi.fn();
    renderPane({ onToggleMcp });

    await user.click(screen.getByTestId('mcp-checkbox-m1'));
    expect(onToggleMcp).toHaveBeenCalledWith('m1', ['read_file', 'write_file']);
  });

  it('toggles a single tool', async () => {
    const user = userEvent.setup();
    const onToggleTool = vi.fn();
    renderPane({ onToggleTool, enabledMcpTools: { m1: ['read_file'] } });

    await user.click(screen.getByTestId('mcp-expand-m1'));
    await user.click(screen.getByTestId('mcp-tool-checkbox-m1-write_file'));
    expect(onToggleTool).toHaveBeenCalledWith('m1', 'write_file');
  });

  it('shows an indeterminate server checkbox for a partial selection', () => {
    renderPane({ enabledMcpTools: { m1: ['read_file'] } });
    expect(screen.getByTestId('mcp-checkbox-m1')).toHaveAttribute('data-state', 'indeterminate');
  });

  it('disables a server that is turned off by the administrator', () => {
    renderPane({ mcps: [mcp({ mcp_server: { enabled: false } as never })] });
    expect(screen.getByTestId('mcp-expand-m1')).toBeDisabled();
  });
});
