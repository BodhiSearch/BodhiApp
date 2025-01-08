'use client';

import { createWrapper } from '@/tests/wrapper';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';
import { NavigationSidebar } from './NavigationSidebar';
import { SidebarProvider } from '@/components/ui/sidebar';
import { Check } from 'lucide-react';

const pushMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

// Mock the useNavigation hook
vi.mock('@/hooks/use-navigation', () => ({
  useNavigation: () => ({
    pages: [
      { url: '/ui/chat', title: 'Chat', iconName: 'messageSquare' },
      { url: '/ui/models', title: 'Models', iconName: 'boxes' },
      { url: '/ui/settings', title: 'Settings', iconName: 'settings' },
    ],
    currentPage: {
      url: '/ui/chat',
      title: 'Chat',
      iconName: 'messageSquare'
    }
  })
}));

// Mock the lucide-react Check component
vi.mock('lucide-react', () => ({
  Check: () => <svg data-testid="check-icon" />,
  ChevronsUpDown: () => <svg data-testid="chevrons-icon" />,
  MessageSquare: () => <svg data-testid="chevrons-icon" />
}));

const renderWithProviders = (ui: React.ReactElement) => {
  const Wrapper = ({ children }: { children: React.ReactNode }) => {
    const BaseWrapper = createWrapper();
    return (
      <BaseWrapper>
        <SidebarProvider>
          {children}
        </SidebarProvider>
      </BaseWrapper>
    );
  };
  return render(ui, { wrapper: Wrapper });
};

describe('NavigationSidebar', () => {
  it('renders current page with correct icon and title', () => {
    renderWithProviders(<NavigationSidebar />);

    // Check if current page title is displayed in the button
    const button = screen.getByRole('button');
    expect(button).toHaveTextContent('Chat');

    // Check if icon container exists
    const iconContainer = button.querySelector('.size-8');
    expect(iconContainer).toBeInTheDocument();
  });

  it('shows all pages in dropdown and allows navigation', async () => {
    const user = userEvent.setup();
    renderWithProviders(<NavigationSidebar />);

    // Open dropdown
    const trigger = screen.getByRole('button');
    await user.click(trigger);

    // Get all menu items
    const menuItems = screen.getAllByRole('menuitem');

    // Verify all pages are listed in the dropdown
    expect(menuItems).toHaveLength(3);
    expect(menuItems[0]).toHaveTextContent('Chat');
    expect(menuItems[1]).toHaveTextContent('Models');
    expect(menuItems[2]).toHaveTextContent('Settings');

    // Current page (Chat) should have a check mark
    const checkIcon = screen.getByTestId('check-icon');
    expect(checkIcon).toBeInTheDocument();

    // Click on Models option
    await user.click(menuItems[1]);

    // Verify navigation was triggered
    expect(pushMock).toHaveBeenCalledWith('/ui/models');
  });
});