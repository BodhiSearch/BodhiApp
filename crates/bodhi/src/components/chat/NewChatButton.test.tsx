import { render, screen, fireEvent } from '@testing-library/react';
import { NewChatButton } from './NewChatButton';
import { vi } from 'vitest';
import { SidebarProvider } from '@/components/ui/sidebar';

// Mock hooks
const mockCreateNewChat = vi.fn();
vi.mock('@/hooks/use-chat-db', () => ({
  useChatDB: () => ({
    createNewChat: mockCreateNewChat,
  }),
}));

// Test wrapper component
const renderWithSidebar = (component: React.ReactNode) => {
  return render(
    <SidebarProvider>
      {component}
    </SidebarProvider>
  );
};

describe('NewChatButton', () => {
  beforeEach(() => {
    vi.clearAllMocks();
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