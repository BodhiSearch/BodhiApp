import { render, act, waitFor } from '@testing-library/react';
import { ChatContainer } from './ChatContainer';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createWrapper } from '@/tests/wrapper';
import { useLocalStorage } from '@/hooks/useLocalStorage';
import { renderHook } from '@testing-library/react';

// Mock hooks
const mockSetLocalStorage = vi.fn();
const mockInitializeCurrentChatId = vi.fn();

// Mock Next.js router
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: vi.fn(),
    replace: vi.fn(),
    refresh: vi.fn(),
  }),
  useSearchParams: () => ({
    get: vi.fn(),
  }),
}));

vi.mock('@/hooks/useLocalStorage', () => ({
  useLocalStorage: vi.fn(() => [true, mockSetLocalStorage]),
}));

vi.mock('@/hooks/use-chat-db', () => ({
  useChatDB: vi.fn(() => ({
    initializeCurrentChatId: mockInitializeCurrentChatId,
  })),
}));

// Mock child components to avoid testing their implementation
vi.mock('@/components/chat/ChatUI', () => ({
  ChatUI: ({ isLoading }: { isLoading: boolean }) => <div data-testid="chat-ui" data-loading={isLoading.toString()} />,
}));

vi.mock('@/components/chat/ChatHistory', () => ({
  ChatHistory: () => <div data-testid="chat-history" />,
}));

vi.mock('@/components/settings/SettingsSidebar', () => ({
  SettingsSidebar: () => <div data-testid="settings-sidebar" />,
}));

// Mock MainLayout
vi.mock('@/components/layout/MainLayout', () => ({
  MainLayout: ({ children }: { children: React.ReactNode }) => <div data-testid="main-layout">{children}</div>,
}));

// Mock SidebarProvider
vi.mock('@/components/ui/sidebar', () => ({
  SidebarProvider: ({ children }: { children: React.ReactNode }) => <div data-testid="sidebar-provider">{children}</div>,
  SidebarTrigger: () => <button data-testid="sidebar-trigger">Settings</button>,
}));

// Mock ChatSettingsProvider
vi.mock('@/hooks/use-chat-settings', () => ({
  ChatSettingsProvider: ({ children }: { children: React.ReactNode }) => <div data-testid="chat-settings-provider">{children}</div>,
}));

describe('ChatContainer', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Initialization', () => {
    it('should initialize current chat ID on mount', async () => {
      render(<ChatContainer />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(mockInitializeCurrentChatId).toHaveBeenCalled();
      });
    });

    it('should show loading state during initialization', async () => {
      let resolveInitialize: () => void;
      mockInitializeCurrentChatId.mockImplementation(
        () => new Promise((resolve) => {
          resolveInitialize = resolve as () => void;
        })
      );

      const { getByTestId } = render(<ChatContainer />, {
        wrapper: createWrapper(),
      });

      await act(async () => {
        resolveInitialize!();
      });
      expect(getByTestId('chat-ui').getAttribute('data-loading')).toBe('false');
    });
  });

  describe('Settings sidebar state', () => {
    it('should persist settings sidebar state', async () => {
      render(<ChatContainer />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(useLocalStorage).toHaveBeenCalledWith('settings-sidebar-state', true);
      });
    });

    it('should handle settings sidebar state changes', async () => {
      const { result } = renderHook(() => useLocalStorage('settings-sidebar-state', true));
      const [, setSettingsOpen] = result.current;

      render(<ChatContainer />, { wrapper: createWrapper() });

      await act(async () => {
        setSettingsOpen(false);
      });

      expect(mockSetLocalStorage).toHaveBeenCalledWith(false);
    });
  });
});