import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ChatHistory } from './ChatHistory';
import { useChatDB } from '@/hooks/use-chat-db';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { Chat } from '@/types/chat';
import { SidebarProvider } from '@/components/ui/sidebar';

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
  const mockChats: Chat[] = [
    {
      id: '1',
      title: 'Chat 1',
      messages: [{ role: 'user', content: 'test' }],
      createdAt: Date.now(),
      updatedAt: Date.now(),
    },
    {
      id: '2',
      title: 'Chat 2',
      messages: [{ role: 'user', content: 'test' }],
      createdAt: Date.now(),
      updatedAt: Date.now(),
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

  it('renders non-empty chats', () => {
    render(<ChatHistory />, { wrapper: Wrapper });

    expect(screen.getByText('Chat 1')).toBeInTheDocument();
    expect(screen.getByText('Chat 2')).toBeInTheDocument();
  });

  it('does not render empty chats', () => {
    const chatsWithEmpty = [
      ...mockChats,
      {
        id: '3',
        title: 'Empty Chat',
        messages: [],
        createdAt: Date.now(),
        updatedAt: Date.now(),
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

    await user.click(screen.getByText('Chat 2'));
    expect(mockSetCurrentChatId).toHaveBeenCalledWith('2');
  });

  it('marks current chat as active', () => {
    render(<ChatHistory />, { wrapper: Wrapper });
    
    const currentChat = screen.getByText('Chat 1').closest('button');
    const otherChat = screen.getByText('Chat 2').closest('button');
    
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