import { describe, it, beforeEach, expect, vi } from 'vitest';
import { renderHook } from '@testing-library/react';
import { Home, Settings, Users } from 'lucide-react';
import type { NavigationItem } from '@/types/navigation';
import { createWrapper } from '@/tests/wrapper';

// Setup mocks before importing the components
const mockPathname = vi.fn();
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
  };
});

// Import after mocks are setup
import { useNavigation, NavigationProvider } from '@/hooks/navigation';

describe('useNavigation', () => {
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
          title: 'Manage Users',
          href: '/users/',
          description: 'Manage users and access control',
          icon: Users,
        },
        {
          title: 'Child',
          href: '/child/',
          description: 'Test Child',
          icon: Settings,
          items: [
            {
              title: 'Grandchild',
              href: '/child/grandchild/',
              description: 'Test Grandchild',
              icon: Settings,
              skip: true,
            },
          ],
        },
      ],
    },
  ];

  const Wrapper = createWrapper();
  const renderWithProvider = ({ children }: { children: React.ReactNode }) => (
    <Wrapper>
      <NavigationProvider items={testNavigationItems}>{children}</NavigationProvider>
    </Wrapper>
  );

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should return root level item when path matches', () => {
    mockPathname.mockReturnValue('/root/');

    const { result } = renderHook(() => useNavigation(), {
      wrapper: renderWithProvider,
    });

    expect(result.current.currentItem).toEqual({
      item: expect.objectContaining({
        title: 'Root',
        href: '/root/',
      }),
      parent: null,
    });
  });

  it('should return sub-item with parent when path matches', () => {
    mockPathname.mockReturnValue('/child/');

    const { result } = renderHook(() => useNavigation(), {
      wrapper: renderWithProvider,
    });

    expect(result.current.currentItem.item).toEqual(
      expect.objectContaining({
        title: 'Child',
        href: '/child/',
        description: 'Test Child',
      })
    );

    expect(result.current.currentItem.parent).toEqual(
      expect.objectContaining({
        title: 'Settings',
        items: expect.any(Array),
      })
    );
  });

  it('should return sub-sub-item with immediate parent when path matches', () => {
    mockPathname.mockReturnValue('/child/grandchild/');

    const { result } = renderHook(() => useNavigation(), {
      wrapper: renderWithProvider,
    });

    expect(result.current.currentItem.item).toEqual(
      expect.objectContaining({
        title: 'Grandchild',
        href: '/child/grandchild/',
        skip: true,
      })
    );

    expect(result.current.currentItem.parent).toEqual(
      expect.objectContaining({
        title: 'Child',
        href: '/child/',
      })
    );
  });

  it.each([
    ['/users/', 'Manage Users', '/users/', 'Manage users and access control', 'Settings'],
    ['/users/pending', 'Manage Users', '/users/', 'Manage users and access control', 'Settings'],
    ['/users/access-requests', 'Manage Users', '/users/', 'Manage users and access control', 'Settings'],
  ])(
    'should return %s > %s for %s paths',
    (pathname, expectedItemTitle, expectedHref, expectedDescription, expectedParentTitle) => {
      mockPathname.mockReturnValue(pathname);

      const { result } = renderHook(() => useNavigation(), {
        wrapper: renderWithProvider,
      });

      expect(result.current.currentItem.item).toEqual(
        expect.objectContaining({
          title: expectedItemTitle,
          href: expectedHref,
          description: expectedDescription,
        })
      );

      expect(result.current.currentItem.parent).toEqual(
        expect.objectContaining({
          title: expectedParentTitle,
          items: expect.any(Array),
        })
      );
    }
  );

  it('should default to Home when no path matches', () => {
    mockPathname.mockReturnValue('/non/existent/path');

    const { result } = renderHook(() => useNavigation(), {
      wrapper: renderWithProvider,
    });

    expect(result.current.currentItem).toEqual({
      item: expect.objectContaining({
        title: 'Root',
        href: '/root/',
      }),
      parent: null,
    });
  });
});
