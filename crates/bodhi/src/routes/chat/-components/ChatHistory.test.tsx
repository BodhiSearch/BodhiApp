import { ChatHistory } from '@/routes/chat/-components/ChatHistory';
import { useChatStore } from '@/stores/chatStore';
import { Chat } from '@/types/chat';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@/stores/chatStore', () => {
  const { create } = require('zustand');
  const store = create(() => ({
    chats: [],
    deleteChat: vi.fn(),
    currentChatId: null,
    setCurrentChatId: vi.fn(),
  }));
  return { useChatStore: store };
});

function Wrapper({ children }: { children: React.ReactNode }) {
  return <>{children}</>;
}

describe('ChatHistory', () => {
  const now = Date.now();
  const yesterday = now - 24 * 60 * 60 * 1000;
  const lastWeek = now - 7 * 24 * 60 * 60 * 1000;

  const mockChats: Chat[] = [
    {
      id: '1',
      title: 'Today Chat',
      messages: [{ role: 'user', content: 'test' }],
      messageCount: 1,
      createdAt: now,
      updatedAt: now,
    },
    {
      id: '2',
      title: 'Yesterday Chat',
      messages: [{ role: 'user', content: 'test' }],
      messageCount: 1,
      createdAt: yesterday,
      updatedAt: yesterday,
    },
    {
      id: '3',
      title: 'Previous Chat',
      messages: [{ role: 'user', content: 'test' }],
      messageCount: 1,
      createdAt: lastWeek,
      updatedAt: lastWeek,
    },
  ];

  const mockDeleteChat = vi.fn();
  const mockSetCurrentChatId = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();

    useChatStore.setState({
      chats: mockChats,
      deleteChat: mockDeleteChat,
      currentChatId: '1',
      setCurrentChatId: mockSetCurrentChatId,
    });
  });

  it('renders chats in correct groups', () => {
    render(<ChatHistory />, { wrapper: Wrapper });

    expect(screen.getByText('TODAY')).toBeInTheDocument();
    expect(screen.getByText('YESTERDAY')).toBeInTheDocument();
    expect(screen.getByText('PREVIOUS 7 DAYS')).toBeInTheDocument();

    expect(screen.getByText('Today Chat')).toBeInTheDocument();
    expect(screen.getByText('Yesterday Chat')).toBeInTheDocument();
    expect(screen.getByText('Previous Chat')).toBeInTheDocument();
  });

  it('does not render empty chats', () => {
    const chatsWithEmpty = [
      ...mockChats,
      {
        id: '4',
        title: 'Empty Chat',
        messages: [],
        messageCount: 0,
        createdAt: now,
        updatedAt: now,
      },
    ];

    useChatStore.setState({ chats: chatsWithEmpty });

    render(<ChatHistory />, { wrapper: Wrapper });
    expect(screen.queryByText('Empty Chat')).not.toBeInTheDocument();
  });

  it('selects chat when clicked', async () => {
    const user = userEvent.setup();
    render(<ChatHistory />, { wrapper: Wrapper });

    await user.click(screen.getByText('Yesterday Chat'));
    expect(mockSetCurrentChatId).toHaveBeenCalledWith('2');
  });

  it('marks current chat as active', () => {
    render(<ChatHistory />, { wrapper: Wrapper });

    // The active row carries the `on` class on its `chat-item` container (CSS-driven highlight).
    const currentRow = screen.getByTestId('chat-history-item-1');
    const otherRow = screen.getByTestId('chat-history-item-2');
    expect(currentRow.className).toContain('on');
    expect(otherRow.className).not.toContain('on');
  });

  it('filters chats by the search prop', () => {
    render(<ChatHistory search="yesterday" />, { wrapper: Wrapper });

    expect(screen.getByText('Yesterday Chat')).toBeInTheDocument();
    expect(screen.queryByText('Today Chat')).not.toBeInTheDocument();
    expect(screen.queryByText('Previous Chat')).not.toBeInTheDocument();
  });

  it('deletes a chat via the hover trash icon without selecting it', async () => {
    const user = userEvent.setup();
    render(<ChatHistory />, { wrapper: Wrapper });

    await user.click(screen.getByTestId('delete-chat-1'));

    expect(mockDeleteChat).toHaveBeenCalledWith('1');
    expect(mockSetCurrentChatId).not.toHaveBeenCalled();
  });

  it('renders the compact popover variant', () => {
    render(<ChatHistory compact />, { wrapper: Wrapper });

    expect(screen.getByText('Today')).toBeInTheDocument();
    expect(screen.getByTestId('chat-history-button-1')).toBeInTheDocument();
  });
});
