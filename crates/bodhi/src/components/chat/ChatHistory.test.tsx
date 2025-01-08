import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ChatHistory } from './ChatHistory';
import { useChatDB } from '@/hooks/use-chat-db';
import { useRouter, useSearchParams } from 'next/navigation';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { Chat } from '@/types/chat';
import { SidebarProvider } from '@/components/ui/sidebar';

// Mock dependencies
vi.mock('next/navigation', () => ({
  useRouter: vi.fn(() => ({
    push: vi.fn()
  })),
  useSearchParams: vi.fn(() => ({
    get: vi.fn()
  }))
}));

vi.mock('@/hooks/use-chat-db', () => ({
  useChatDB: vi.fn()
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
      messages: [],
      createdAt: Date.now(),
      updatedAt: Date.now()
    },
    {
      id: '2',
      title: 'Chat 2',
      messages: [],
      createdAt: Date.now(),
      updatedAt: Date.now()
    }
  ];

  const mockDeleteChat = vi.fn();
  const mockPush = vi.fn();
  const mockGetParam = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();

    // Mock matchMedia for SidebarProvider
    Object.defineProperty(window, 'matchMedia', {
      writable: true,
      value: vi.fn().mockImplementation(query => ({
        matches: false,
        media: query,
        onchange: null,
        addListener: vi.fn(),
        removeListener: vi.fn(),
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        dispatchEvent: vi.fn(),
      })),
    });

    vi.mocked(useRouter).mockReturnValue({
      push: mockPush
    } as any);

    vi.mocked(useSearchParams).mockReturnValue({
      get: mockGetParam
    } as any);

    vi.mocked(useChatDB).mockReturnValue({
      chats: mockChats,
      deleteChat: mockDeleteChat,
      getChat: vi.fn(),
      createOrUpdateChat: vi.fn(),
      clearChats: vi.fn()
    });
  });

  it('renders chat list', () => {
    render(<ChatHistory />, { wrapper: Wrapper });

    expect(screen.getByText('Chat 1')).toBeInTheDocument();
    expect(screen.getByText('Chat 2')).toBeInTheDocument();
  });

  it('navigates to chat when clicked', async () => {
    const user = userEvent.setup();
    render(<ChatHistory />, { wrapper: Wrapper });

    await user.click(screen.getByText('Chat 1'));
    expect(mockPush).toHaveBeenCalledWith('/ui/chat/?id=1');
  });

  it('marks current chat as active', () => {
    mockGetParam.mockReturnValue('1'); // Mock current chat id
    render(<ChatHistory />, { wrapper: Wrapper });
    
    const chatButton = screen.getByText('Chat 1').closest('button');
    expect(chatButton).toHaveAttribute('data-active', 'true');
    
    const otherChatButton = screen.getByText('Chat 2').closest('button');
    expect(otherChatButton).toHaveAttribute('data-active', 'false');
  });

  it('deletes chat when trash icon is clicked', async () => {
    const user = userEvent.setup();
    render(<ChatHistory />, { wrapper: Wrapper });

    const deleteButton = screen.getByTestId('delete-chat-1');
    await user.click(deleteButton);
    expect(mockDeleteChat).toHaveBeenCalledWith('1');
  });

  it('renders chat with title "New Chat"', () => {
    const chatsWithoutTitle: Chat[] = [{
      id: '1',
      title: 'New Chat',
      messages: [],
      createdAt: Date.now(),
      updatedAt: Date.now()
    }];

    vi.mocked(useChatDB).mockReturnValue({
      chats: chatsWithoutTitle,
      deleteChat: mockDeleteChat,
      getChat: vi.fn(),
      createOrUpdateChat: vi.fn(),
      clearChats: vi.fn()
    });

    render(<ChatHistory />, { wrapper: Wrapper });
    expect(screen.getByText('New Chat')).toBeInTheDocument();
  });

  it('stops event propagation when deleting chat', async () => {
    const user = userEvent.setup();
    render(<ChatHistory />, { wrapper: Wrapper });

    const deleteButton = screen.getByTestId('delete-chat-1');

    const mockStopPropagation = vi.fn();
    const event = new MouseEvent('click', {
      bubbles: true,
      cancelable: true
    });
    Object.defineProperty(event, 'stopPropagation', {
      value: mockStopPropagation
    });

    deleteButton.dispatchEvent(event);
    expect(mockStopPropagation).toHaveBeenCalled();
    expect(mockPush).not.toHaveBeenCalled(); // Ensures navigation didn't occur
  });
});