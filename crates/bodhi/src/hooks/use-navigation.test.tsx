import { describe, it, beforeEach, expect, vi } from 'vitest';
import { renderHook } from '@testing-library/react';
import { Home, Settings, Users, BookText } from 'lucide-react';
import type { NavigationItem } from '@/types/navigation';
import { createWrapper } from '@/tests/wrapper';

// Setup mocks before importing the components
const mockUsePathname = vi.fn();
vi.mock('next/navigation', () => ({
  usePathname: () => mockUsePathname(),
}));

// Import after mocks are setup
import { useNavigation, NavigationProvider } from '@/hooks/use-navigation';

describe('useNavigation', () => {
  const testNavigationItems: NavigationItem[] = [
    {
      title: 'Root',
      href: '/ui/root/',
      description: 'Test Root',
      icon: Home,
    },
    {
      title: 'Settings',
      icon: Settings,
      items: [
        {
          title: 'Manage Users',
          href: '/ui/users/',
          description: 'Manage users and access control',
          icon: Users,
        },
        {
          title: 'Child',
          href: '/ui/child/',
          description: 'Test Child',
          icon: Settings,
          items: [
            {
              title: 'Grandchild',
              href: '/ui/child/grandchild/',
              description: 'Test Grandchild',
              icon: Settings,
              skip: true,
            },
          ],
        },
      ],
    },
    {
      title: 'Documentation',
      icon: BookText,
      items: [
        {
          title: 'App Guide',
          href: '/docs/',
          description: 'User guides and documentation',
          icon: BookText,
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
    mockUsePathname.mockReturnValue('/ui/root/');

    const { result } = renderHook(() => useNavigation(), {
      wrapper: renderWithProvider,
    });

    expect(result.current.currentItem).toEqual({
      item: expect.objectContaining({
        title: 'Root',
        href: '/ui/root/',
      }),
      parent: null,
    });
  });

  it('should return sub-item with parent when path matches', () => {
    mockUsePathname.mockReturnValue('/ui/child/');

    const { result } = renderHook(() => useNavigation(), {
      wrapper: renderWithProvider,
    });

    expect(result.current.currentItem.item).toEqual(
      expect.objectContaining({
        title: 'Child',
        href: '/ui/child/',
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
    mockUsePathname.mockReturnValue('/ui/child/grandchild/');

    const { result } = renderHook(() => useNavigation(), {
      wrapper: renderWithProvider,
    });

    expect(result.current.currentItem.item).toEqual(
      expect.objectContaining({
        title: 'Grandchild',
        href: '/ui/child/grandchild/',
        skip: true,
      })
    );

    expect(result.current.currentItem.parent).toEqual(
      expect.objectContaining({
        title: 'Child',
        href: '/ui/child/',
      })
    );
  });

  it.each([
    ['/ui/users/', 'Manage Users', '/ui/users/', 'Manage users and access control', 'Settings'],
    ['/ui/users/pending', 'Manage Users', '/ui/users/', 'Manage users and access control', 'Settings'],
    ['/ui/users/access-requests', 'Manage Users', '/ui/users/', 'Manage users and access control', 'Settings'],
    ['/docs/', 'App Guide', '/docs/', 'User guides and documentation', 'Documentation'],
  ])('should return %s > %s for %s paths', (pathname, expectedItemTitle, expectedHref, expectedDescription, expectedParentTitle) => {
    mockUsePathname.mockReturnValue(pathname);

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
  });

  it('should default to Home when no path matches', () => {
    mockUsePathname.mockReturnValue('/non/existent/path');

    const { result } = renderHook(() => useNavigation(), {
      wrapper: renderWithProvider,
    });

    expect(result.current.currentItem).toEqual({
      item: expect.objectContaining({
        title: 'Root',
        href: '/ui/root/',
      }),
      parent: null,
    });
  });
});
