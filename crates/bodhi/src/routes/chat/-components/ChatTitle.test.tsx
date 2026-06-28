import { ChatTitle } from '@/routes/chat/-components/ChatTitle';
import { useChatStore } from '@/stores/chatStore';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const getChat = vi.fn();
const createOrUpdateChat = vi.fn();

vi.mock('@/stores/chatStore', () => {
  const { create } = require('zustand');
  return {
    useChatStore: create(() => ({
      currentChatId: 'c1',
      chats: [{ id: 'c1', title: 'My Chat', messageCount: 2, createdAt: 0, updatedAt: 0, messages: [] }],
      getChat: (...args: unknown[]) => getChat(...args),
      createOrUpdateChat: (...args: unknown[]) => createOrUpdateChat(...args),
    })),
  };
});

describe('ChatTitle', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useChatStore.setState({
      currentChatId: 'c1',
      chats: [{ id: 'c1', title: 'My Chat', messageCount: 2, createdAt: 0, updatedAt: 0, messages: [] } as never],
    });
  });

  it('renders the crumb and current chat title', () => {
    render(<ChatTitle />);
    expect(screen.getByTestId('chat-title')).toHaveTextContent('Chat');
    expect(screen.getByTestId('chat-title-edit')).toHaveTextContent('My Chat');
  });

  it('renames the chat through the store on Enter', async () => {
    const user = userEvent.setup();
    getChat.mockResolvedValue({ data: { id: 'c1', title: 'My Chat', messages: [] }, status: 200 });
    createOrUpdateChat.mockResolvedValue('c1');

    render(<ChatTitle />);
    await user.click(screen.getByTestId('chat-title-edit'));

    const input = screen.getByTestId('chat-title-input');
    await user.clear(input);
    await user.type(input, 'Renamed{Enter}');

    await waitFor(() => expect(createOrUpdateChat).toHaveBeenCalledWith(expect.objectContaining({ title: 'Renamed' })));
  });

  it('does not save when the title is unchanged', async () => {
    const user = userEvent.setup();
    render(<ChatTitle />);

    await user.click(screen.getByTestId('chat-title-edit'));
    await user.keyboard('{Enter}');

    expect(createOrUpdateChat).not.toHaveBeenCalled();
  });
});
