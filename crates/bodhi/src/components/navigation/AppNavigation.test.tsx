import { createWrapper } from '@/tests/wrapper';
import type { NavigationItem } from '@/types/navigation';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { FilePlus2, Home, Settings } from 'lucide-react';
import { describe, expect, it, vi } from 'vitest';

// Create mock function before vi.mock
const mockPathname = vi.fn();

// All vi.mock calls must come before any imports that use them
vi.mock('@tanstack/react-router', async () => {
  const actual = await vi.importActual('@tanstack/react-router');
  return {
    ...actual,
    Link: ({ to, children, ...rest }: any) => (
      <a href={to} {...rest}>
        {children}
      </a>
    ),
    useLocation: () => ({ pathname: mockPathname() }),
    useNavigate: () => vi.fn(),
  };
});

vi.mock('@/components/ThemeProvider', () => ({
  useTheme: () => ({
    theme: 'light',
    setTheme: vi.fn(),
  }),
}));

vi.mock('@/hooks/use-mobile', () => ({
  useIsMobile: () => false,
}));

vi.mock('@/components/LoginMenu', () => ({
  LoginMenu: () => <div data-testid="login-menu">Login Menu</div>,
}));

// Import components after all mocks are defined
import { AppNavigation } from '@/components/navigation/AppNavigation';
import { NavigationProvider } from '@/hooks/navigation';

describe('AppNavigation', () => {
  // Setup for Radix UI components
  Object.assign(window.HTMLElement.prototype, {
    scrollIntoView: vi.fn(),
    releasePointerCapture: vi.fn(),
    hasPointerCapture: vi.fn(),
  });

  beforeEach(() => {
    vi.clearAllMocks();
    mockPathname.mockReset();
    mockPathname.mockReturnValue('/test/');
  });

  const testNavigationItems: NavigationItem[] = [
    {
      title: 'Root',
      href: '/root/',
      description: 'Test Root',
      icon: Home,
    },
    {
      title: 'Settings',
      icon: Settings,
      items: [
        {
          title: 'Parent Item',
          href: '/parent/',
          description: 'Parent Item',
          icon: Settings,
          items: [
            {
              title: 'Hidden Child',
              href: '/parent/child/',
              description: 'This item should be hidden but selectable',
              icon: FilePlus2,
              skip: true,
            },
          ],
        },
        {
          title: 'Regular Item',
          href: '/regular/',
          description: 'Regular visible item',
          icon: Settings,
        },
      ],
    },
  ];

  const Wrapper = createWrapper();
  const renderNavigation = () => {
    return render(
      <Wrapper>
        <NavigationProvider items={testNavigationItems}>
          <AppNavigation />
        </NavigationProvider>
      </Wrapper>
    );
  };

  it('should render navigation menu button', () => {
    renderNavigation();
    expect(screen.getByTestId('navigation-menu-button')).toBeInTheDocument();
  });

  it('should show visible items and hide skipped items in dropdown', async () => {
    const user = userEvent.setup();
    renderNavigation();

    await user.click(screen.getByTestId('navigation-menu-button'));

    const menuContent = screen.getByTestId('navigation-menu-content');

    const parentItem = within(menuContent).getByRole('menuitem', {
      name: /Parent Item/i,
    });
    expect(parentItem).toBeInTheDocument();

    const regularItem = within(menuContent).getByRole('menuitem', {
      name: /Regular Item.*Regular visible item/i,
    });
    expect(regularItem).toBeInTheDocument();

    expect(
      within(menuContent).queryByRole('menuitem', {
        name: /Hidden Child/i,
      })
    ).not.toBeInTheDocument();
    expect(within(menuContent).queryByText('This item should be hidden but selectable')).not.toBeInTheDocument();
  });

  it('should render all parent items regardless of skip property', async () => {
    const user = userEvent.setup();
    renderNavigation();

    await user.click(screen.getByTestId('navigation-menu-button'));

    const menuContent = screen.getByTestId('navigation-menu-content');

    expect(within(menuContent).getByText('Root')).toBeInTheDocument();
    expect(within(menuContent).getByText('Settings')).toBeInTheDocument();
  });

  it('should highlight parent when skipped child is current', async () => {
    mockPathname.mockReturnValue('/parent/child/');

    const user = userEvent.setup();
    renderNavigation();

    await user.click(screen.getByTestId('navigation-menu-button'));

    const menuContent = screen.getByTestId('navigation-menu-content');

    const parentMenuItem = within(menuContent).getByRole('menuitem', {
      name: /Parent Item.*Parent Item/i,
    });

    expect(parentMenuItem).toHaveClass('bg-accent');

    const regularMenuItem = within(menuContent).getByRole('menuitem', {
      name: /Regular Item.*Regular visible item/i,
    });
    expect(regularMenuItem).not.toHaveClass('bg-accent');

    expect(
      within(menuContent).queryByRole('menuitem', {
        name: /Hidden Child/i,
      })
    ).not.toBeInTheDocument();
  });

  it('should highlight item when its href matches current path', async () => {
    mockPathname.mockReturnValue('/regular/');

    const user = userEvent.setup();
    renderNavigation();

    await user.click(screen.getByTestId('navigation-menu-button'));

    const menuContent = screen.getByTestId('navigation-menu-content');

    const regularMenuItem = within(menuContent).getByRole('menuitem', {
      name: /Regular Item.*Regular visible item/i,
    });
    expect(regularMenuItem).toHaveClass('bg-accent');

    const parentMenuItem = within(menuContent).getByRole('menuitem', {
      name: /Parent Item.*Parent Item/i,
    });
    expect(parentMenuItem).not.toHaveClass('bg-accent');
  });
});
