import { NewChatButton } from '@/routes/chat/-components/NewChatButton';
import { SidebarProvider } from '@/components/ui/sidebar';
import { useChatStore } from '@/stores/chatStore';
import { fireEvent, render, screen } from '@testing-library/react';
import { vi } from 'vitest';

const mockCreateNewChat = vi.fn();

vi.mock('@/stores/chatStore', () => {
  const { create } = require('zustand');
  const store = create(() => ({
    createNewChat: vi.fn(),
  }));
  return { useChatStore: store };
});

const renderWithSidebar = (component: React.ReactNode) => {
  return render(<SidebarProvider>{component}</SidebarProvider>);
};

describe('NewChatButton', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useChatStore.setState({ createNewChat: mockCreateNewChat });
  });

  it('renders the button with icon and text', () => {
    renderWithSidebar(<NewChatButton />);
    const button = screen.getByTestId('new-chat-button');
    expect(button).toBeInTheDocument();
    expect(button).toHaveTextContent('New Chat');
  });

  it('calls createNewChat when clicked', () => {
    renderWithSidebar(<NewChatButton />);
    const button = screen.getByTestId('new-chat-button');
    fireEvent.click(button);
    expect(mockCreateNewChat).toHaveBeenCalled();
  });
});
