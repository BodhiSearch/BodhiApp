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
      onAdd={vi.fn()}
      onRemove={vi.fn()}
      mcpTools={tools}
      mcpConnectionStatus={status}
      {...props}
    />
  );
}

describe('McpServersPane (add-server model)', () => {
  it('renders the empty state with a settings link when there are no MCPs', () => {
    renderPane({ mcps: [] });
    expect(screen.getByTestId('mcps-empty-state')).toBeInTheDocument();
    expect(screen.getByTestId('mcps-settings-link')).toHaveAttribute('href', '/mcps/');
  });

  it('shows the add combobox + the none-added state when nothing is added yet', () => {
    renderPane();
    expect(screen.getByTestId('mcp-add-trigger')).toBeInTheDocument();
    expect(screen.getByTestId('mcps-none-added')).toBeInTheDocument();
    // A not-yet-added server is NOT shown as a row.
    expect(screen.queryByTestId('mcp-item-m1')).not.toBeInTheDocument();
  });

  it('adds a server from the combobox (establishes its connection)', async () => {
    const user = userEvent.setup();
    const onAdd = vi.fn();
    renderPane({ onAdd });

    await user.click(screen.getByTestId('mcp-add-trigger'));
    await user.click(screen.getByTestId('mcp-add-option-m1'));

    expect(onAdd).toHaveBeenCalledWith(expect.objectContaining({ id: 'm1' }));
  });

  it('lists an added server as an expandable row and reveals its tools', async () => {
    const user = userEvent.setup();
    renderPane({ enabledMcpTools: { m1: ['read_file', 'write_file'] } });

    // Added → shown as a row; combobox reports all-added.
    expect(screen.getByTestId('mcp-item-m1')).toBeInTheDocument();
    expect(screen.queryByTestId('mcp-add-trigger')).not.toBeInTheDocument();

    await user.click(screen.getByTestId('mcp-expand-m1'));
    expect(screen.getByTestId('mcp-tool-checkbox-m1-read_file')).toBeInTheDocument();
    expect(screen.getByTestId('mcp-tool-checkbox-m1-write_file')).toBeInTheDocument();
  });

  it('toggles a single tool off within an added server', async () => {
    const user = userEvent.setup();
    const onToggleTool = vi.fn();
    renderPane({ onToggleTool, enabledMcpTools: { m1: ['read_file', 'write_file'] } });

    await user.click(screen.getByTestId('mcp-expand-m1'));
    await user.click(screen.getByTestId('mcp-tool-checkbox-m1-write_file'));
    expect(onToggleTool).toHaveBeenCalledWith('m1', 'write_file');
  });

  it('removes an added server (tears down its connection)', async () => {
    const user = userEvent.setup();
    const onRemove = vi.fn();
    renderPane({ onRemove, enabledMcpTools: { m1: ['read_file'] } });

    await user.click(screen.getByTestId('mcp-remove-m1'));
    expect(onRemove).toHaveBeenCalledWith('m1');
  });

  it('omits unavailable servers from the add combobox', async () => {
    const user = userEvent.setup();
    // One available + one admin-disabled server; only the available one is offered.
    renderPane({ mcps: [mcp(), mcp({ id: 'm2', slug: 'blocked', mcp_server: { enabled: false } as never })] });

    await user.click(screen.getByTestId('mcp-add-trigger'));
    expect(screen.getByTestId('mcp-add-option-m1')).toBeInTheDocument();
    expect(screen.queryByTestId('mcp-add-option-m2')).not.toBeInTheDocument();
  });
});
