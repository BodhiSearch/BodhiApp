import { render, screen, fireEvent } from '@testing-library/react';
import { NewChatButton } from './NewChatButton';
import { vi } from 'vitest';
import { SidebarProvider } from '@/components/ui/sidebar';

const mockReplace = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    replace: mockReplace,
  }),
  useSearchParams: () => ({
    get: () => null,
  }),
}));

const mockSetItem = vi.fn();
let mockCurrentChat: any = null;
vi.mock('@/hooks/useLocalStorage', () => ({
  useLocalStorage: () => [mockCurrentChat, mockSetItem],
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
    mockCurrentChat = null;
  });

  it('renders the button with icon and text', () => {
    renderWithSidebar(<NewChatButton />);
    const button = screen.getByTestId('new-chat-button');
    expect(button).toBeInTheDocument();
    expect(button).toHaveTextContent('New Chat');
  });

  it('does nothing when current chat is null', () => {
    renderWithSidebar(<NewChatButton />);
    const button = screen.getByTestId('new-chat-button');
    
    fireEvent.click(button);
    
    expect(mockSetItem).not.toHaveBeenCalled();
    expect(mockReplace).not.toHaveBeenCalled();
  });

  it('does nothing when current chat has no messages', () => {
    mockCurrentChat = { id: '1', messages: [] };
    renderWithSidebar(<NewChatButton />);
    const button = screen.getByTestId('new-chat-button');
    
    fireEvent.click(button);
    
    expect(mockSetItem).not.toHaveBeenCalled();
    expect(mockReplace).not.toHaveBeenCalled();
  });

  it('creates new chat when current chat has messages', () => {
    mockCurrentChat = { id: '1', messages: [{ id: '1', content: 'test' }] };
    renderWithSidebar(<NewChatButton />);
    const button = screen.getByTestId('new-chat-button');
    
    fireEvent.click(button);
    
    expect(mockSetItem).toHaveBeenCalledWith(null);
    expect(mockReplace).toHaveBeenCalledWith('/ui/chat');
  });
}); 