import { ChatRailTabs } from '@/routes/chat/-components/ChatRailTabs';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

describe('ChatRailTabs', () => {
  it('marks the active tab and renders the MCP count', () => {
    render(<ChatRailTabs value="parameters" onChange={vi.fn()} mcpCount={3} />);

    expect(screen.getByTestId('chat-rail-tab-parameters').className).toContain('active');
    expect(screen.getByTestId('chat-rail-tab-mcp').className).not.toContain('active');
    expect(screen.getByTestId('chat-rail-mcp-count')).toHaveTextContent('3');
  });

  it('fires onChange when a tab is clicked', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<ChatRailTabs value="parameters" onChange={onChange} mcpCount={0} />);

    await user.click(screen.getByTestId('chat-rail-tab-mcp'));
    expect(onChange).toHaveBeenCalledWith('mcp');
  });
});
