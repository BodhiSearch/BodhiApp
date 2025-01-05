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

  it('renders navigation sidebar trigger', () => {
    render(
      <MainLayout
        navigationSidebar={<div>Navigation</div>}
      >
        <div>Test Content</div>
      </MainLayout>
    );

    const button = screen.getByTestId('sidebar-button');
    expect(button).toBeInTheDocument();
  });

  it('positions navigation trigger correctly', () => {
    render(
      <MainLayout
        navigationSidebar={<div>Navigation</div>}
      >
        <div>Test Content</div>
      </MainLayout>
    );

    const trigger = screen.getByTestId('sidebar-button').parentElement;
    expect(trigger).toHaveClass('fixed');
    expect(trigger).toHaveClass('left-[16rem]');
  });

  it('renders sidebar provider', () => {
    render(
      <MainLayout
        navigationSidebar={<div>Navigation</div>}
      >
        <div>Test Content</div>
      </MainLayout>
    );

    const provider = screen.getByTestId('sidebar-provider');
    expect(provider).toBeInTheDocument();
  });
});