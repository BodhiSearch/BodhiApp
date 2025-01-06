import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { MainLayout } from '@/components/layout/MainLayout';

// Mock useLocalStorage hook - no longer needed but keeping for reference
vi.mock('@/hooks/useLocalStorage', () => ({
  useLocalStorage: () => [true, vi.fn()],
}));

// Mock the sidebar components
vi.mock('@/components/ui/sidebar', () => ({
  SidebarProvider: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="sidebar-provider">{children}</div>
  ),
  SidebarInset: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="sidebar-inset">{children}</div>
  ),
  SidebarTrigger: () => <button data-testid="sidebar-trigger">Toggle</button>,
}));

// Mock the separator component
vi.mock('@/components/ui/separator', () => ({
  Separator: () => <div data-testid="separator" />,
}));

// Mock the breadcrumb components
vi.mock('@/components/ui/breadcrumb', () => ({
  Breadcrumb: ({ children }: { children: React.ReactNode }) => (
    <nav data-testid="breadcrumb">{children}</nav>
  ),
  BreadcrumbItem: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="breadcrumb-item">{children}</div>
  ),
  BreadcrumbLink: ({ children }: { children: React.ReactNode }) => (
    <a data-testid="breadcrumb-link">{children}</a>
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

  it('renders header with sidebar trigger and breadcrumbs', () => {
    render(
      <MainLayout>
        <div>Test Content</div>
      </MainLayout>
    );

    expect(screen.getByTestId('sidebar-trigger')).toBeInTheDocument();
    expect(screen.getByTestId('separator')).toBeInTheDocument();
    expect(screen.getByTestId('breadcrumb')).toBeInTheDocument();
    expect(screen.getByText('Chat')).toBeInTheDocument();
  });

  it('renders sidebar inset with proper structure', () => {
    render(
      <MainLayout>
        <div>Test Content</div>
      </MainLayout>
    );

    const sidebarInset = screen.getByTestId('sidebar-inset');
    expect(sidebarInset).toBeInTheDocument();
    
    // Check for header
    const header = sidebarInset.querySelector('header');
    expect(header).toHaveClass('flex', 'h-16', 'shrink-0');

    // Check for main content wrapper
    const contentWrapper = sidebarInset.querySelector('div');
    expect(contentWrapper).toHaveClass('flex', 'items-center');
  });

  it('renders sidebar provider', () => {
    render(
      <MainLayout>
        <div>Test Content</div>
      </MainLayout>
    );

    expect(screen.getByTestId('sidebar-provider')).toBeInTheDocument();
  });
});