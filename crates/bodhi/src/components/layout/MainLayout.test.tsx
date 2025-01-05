import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { MainLayout } from '@/components/layout/MainLayout';

// Mock useLocalStorage hook
vi.mock('@/hooks/useLocalStorage', () => ({
  useLocalStorage: () => [true, vi.fn()],
}));

// Mock the sidebar components
vi.mock('@/components/ui/sidebar', () => ({
  SidebarProvider: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="sidebar-provider">{children}</div>
  ),
}));

// Mock the Button component
vi.mock('@/components/ui/button', () => ({
  Button: ({ children, ...props }: { children: React.ReactNode }) => (
    <button {...props} data-testid="sidebar-button">
      {children}
    </button>
  ),
}));

describe('MainLayout', () => {
  it('renders children content', () => {
    render(
      <MainLayout>
        <div>Test Content</div>
      </MainLayout>
    );

    expect(screen.getByText('Test Content')).toBeInTheDocument();
  });

  it('renders navigation sidebar when provided', () => {
    render(
      <MainLayout
        navigationSidebar={<div>Navigation Sidebar</div>}
      >
        <div>Test Content</div>
      </MainLayout>
    );

    expect(screen.getByText('Navigation Sidebar')).toBeInTheDocument();
  });

  it('renders settings sidebar when provided', () => {
    render(
      <MainLayout
        settingsSidebar={<div>Settings Sidebar</div>}
      >
        <div>Test Content</div>
      </MainLayout>
    );

    expect(screen.getByText('Settings Sidebar')).toBeInTheDocument();
  });

  it('renders both navigation and settings sidebars when provided', () => {
    render(
      <MainLayout
        navigationSidebar={<div>Navigation Sidebar</div>}
        settingsSidebar={<div>Settings Sidebar</div>}
      >
        <div>Test Content</div>
      </MainLayout>
    );

    expect(screen.getByText('Navigation Sidebar')).toBeInTheDocument();
    expect(screen.getByText('Settings Sidebar')).toBeInTheDocument();
  });

  it('renders both sidebar triggers', () => {
    render(
      <MainLayout
        navigationSidebar={<div>Navigation</div>}
        settingsSidebar={<div>Settings</div>}
      >
        <div>Test Content</div>
      </MainLayout>
    );

    const buttons = screen.getAllByTestId('sidebar-button');
    expect(buttons).toHaveLength(2);
  });

  it('positions triggers correctly based on side prop', () => {
    render(
      <MainLayout
        navigationSidebar={<div>Navigation</div>}
        settingsSidebar={<div>Settings</div>}
      >
        <div>Test Content</div>
      </MainLayout>
    );

    const [leftTrigger, rightTrigger] = screen.getAllByTestId('sidebar-button')
      .map(button => button.parentElement);

    expect(leftTrigger).toHaveClass('fixed');
    expect(rightTrigger).toHaveClass('fixed');

    // Check left trigger positioning
    expect(leftTrigger).toHaveClass('left-[16rem]');

    // Check right trigger positioning
    expect(rightTrigger).toHaveClass('right-[16rem]');
  });

  it('renders sidebar providers', () => {
    render(
      <MainLayout
        navigationSidebar={<div>Navigation</div>}
        settingsSidebar={<div>Settings</div>}
      >
        <div>Test Content</div>
      </MainLayout>
    );

    const providers = screen.getAllByTestId('sidebar-provider');
    expect(providers).toHaveLength(2);
  });
});