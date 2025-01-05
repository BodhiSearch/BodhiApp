import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { NavigationSidebar } from '@/components/navigation/NavigationSidebar';

vi.mock('next/navigation', () => ({
  usePathname: () => '/ui/home'
}));

// Mock the sidebar components
vi.mock('@/components/ui/sidebar', () => ({
  Sidebar: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="sidebar">{children}</div>
  ),
  SidebarHeader: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="sidebar-header">{children}</div>
  ),
  SidebarContent: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="sidebar-content">{children}</div>
  ),
  SidebarGroup: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="sidebar-group">{children}</div>
  ),
  SidebarGroupContent: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="sidebar-group-content">{children}</div>
  ),
  SidebarGroupLabel: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="sidebar-group-label">{children}</div>
  ),
  SidebarMenu: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="sidebar-menu">{children}</div>
  ),
  SidebarMenuItem: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="sidebar-menu-item">{children}</div>
  ),
  SidebarMenuButton: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="sidebar-menu-button">{children}</div>
  ),
}));

// Mock RecentChats component
vi.mock('@/components/navigation/RecentChats', () => ({
  RecentChats: () => <div data-testid="recent-chats">Recent Chats Mock</div>
}));

describe('NavigationSidebar', () => {
  it('renders navigation items', () => {
    render(<NavigationSidebar />);

    expect(screen.getByText('Bodhi')).toBeInTheDocument();
    expect(screen.getByText('New Chat')).toBeInTheDocument();
    expect(screen.getByText('Home')).toBeInTheDocument();
    expect(screen.getByText('Assistants')).toBeInTheDocument();
  });

  it('uses shadcn sidebar components', () => {
    render(<NavigationSidebar />);

    expect(screen.getByTestId('sidebar')).toBeInTheDocument();
    expect(screen.getByTestId('sidebar-header')).toBeInTheDocument();
    expect(screen.getByTestId('sidebar-content')).toBeInTheDocument();
    expect(screen.getAllByTestId('sidebar-group').length).toBeGreaterThan(0);
  });

  it('renders recent chats component', () => {
    render(<NavigationSidebar />);

    expect(screen.getByTestId('recent-chats')).toBeInTheDocument();
  });
});