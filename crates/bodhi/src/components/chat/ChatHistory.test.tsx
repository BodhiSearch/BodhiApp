import { ChatHistory } from '@/components/chat/ChatHistory';
import { SidebarProvider } from '@/components/ui/sidebar';
import { useChatDB } from '@/hooks/use-chat-db';
import { Chat } from '@/types/chat';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

// Mock dependencies
vi.mock('@/hooks/use-chat-db', () => ({
  useChatDB: vi.fn(),
}));

// Create a wrapper component with SidebarProvider
function Wrapper({ children }: { children: React.ReactNode }) {
  return (
    <SidebarProvider>
      {children}
    </SidebarProvider>
  );
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
      createdAt: now,
      updatedAt: now,
    },
    {
      id: '2',
      title: 'Yesterday Chat',
      messages: [{ role: 'user', content: 'test' }],
      createdAt: yesterday,
      updatedAt: yesterday,
    },
    {
      id: '3',
      title: 'Previous Chat',
      messages: [{ role: 'user', content: 'test' }],
      createdAt: lastWeek,
      updatedAt: lastWeek,
    },
  ];

  const mockDeleteChat = vi.fn();
  const mockSetCurrentChatId = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();

    vi.mocked(useChatDB).mockReturnValue({
      chats: mockChats,
      deleteChat: mockDeleteChat,
      currentChatId: '1',
      setCurrentChatId: mockSetCurrentChatId,
    } as any);
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
        createdAt: now,
        updatedAt: now,
      },
    ];

    vi.mocked(useChatDB).mockReturnValue({
      chats: chatsWithEmpty,
      deleteChat: mockDeleteChat,
      currentChatId: '1',
      setCurrentChatId: mockSetCurrentChatId,
    } as any);

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

    const currentChat = screen.getByText('Today Chat').closest('button');
    const otherChat = screen.getByText('Yesterday Chat').closest('button');

    expect(currentChat).toHaveAttribute('data-active', 'true');
    expect(otherChat).toHaveAttribute('data-active', 'false');
  });

  it('deletes chat when trash icon is clicked', async () => {
    const user = userEvent.setup();
    render(<ChatHistory />, { wrapper: Wrapper });

    const deleteButton = screen.getByTestId('delete-chat-1');
    await user.click(deleteButton);

    expect(mockDeleteChat).toHaveBeenCalledWith('1');
  });

  it('prevents chat selection when deleting', async () => {
    const user = userEvent.setup();
    render(<ChatHistory />, { wrapper: Wrapper });

    const deleteButton = screen.getByTestId('delete-chat-1');
    await user.click(deleteButton);

    expect(mockSetCurrentChatId).not.toHaveBeenCalled();
  });
});