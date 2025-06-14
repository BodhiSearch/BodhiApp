'use client';

import { NavigationItem } from '@/types/navigation';
import {
  Cog,
  Database,
  Download,
  FileJson,
  FilePlus2,
  Files,
  Key,
  MessageSquare,
  Settings,
  Settings2,
  Users,
  BookOpen,
  BookText,
} from 'lucide-react';
import { usePathname } from 'next/navigation';
import { ReactNode, createContext, useContext, useEffect, useMemo } from 'react';

// Rename and export the default navigation items
export const defaultNavigationItems: NavigationItem[] = [
  {
    title: 'Chat',
    href: '/ui/chat/',
    description: 'AI Chat Interface',
    icon: MessageSquare,
  },
  {
    title: 'Models',
    icon: Database,
    items: [
      {
        title: 'Model Aliases',
        href: '/ui/models/',
        description: 'Configure and manage model aliases',
        icon: Settings2,
        items: [
          {
            title: 'New Model Alias',
            href: '/ui/models/new/',
            description: 'Create a new model alias',
            icon: FilePlus2,
            skip: true,
          },
          {
            title: 'Edit Model Alias',
            href: '/ui/models/edit/',
            description: 'Edit a model alias',
            icon: FilePlus2,
            skip: true,
          },
        ],
      },
      {
        title: 'Model Files',
        href: '/ui/modelfiles/',
        description: 'Browse and manage model files',
        icon: Files,
      },
      {
        title: 'Model Downloads',
        href: '/ui/pull/',
        description: 'Download new models',
        icon: Download,
      },
    ],
  },
  {
    title: 'Settings',
    icon: Settings,
    items: [
      {
        title: 'App Settings',
        href: '/ui/settings/',
        description: 'Manage application settings',
        icon: Cog,
      },
      {
        title: 'API Tokens',
        href: '/ui/tokens/',
        description: 'Manage API access tokens',
        icon: Key,
      },
      {
        title: 'Manage Users',
        href: '/ui/users/',
        description: 'Manage users and access control',
        icon: Users,
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
        icon: BookOpen,
      },
      {
        title: 'OpenAPI Docs',
        href: '/swagger-ui',
        description: 'API Documentation',
        icon: FileJson,
        target: '_blank',
      },
    ],
  },
];

interface NavigationContextType {
  currentPath: string;
  currentItem: {
    item: NavigationItem;
    parent: NavigationItem | null;
  };
  navigationItems: NavigationItem[];
}

const NavigationContext = createContext<NavigationContextType>({
  currentPath: '',
  currentItem: {
    item: {} as NavigationItem,
    parent: null,
  },
  navigationItems: [],
});

interface NavigationProviderProps {
  children: ReactNode;
  items?: NavigationItem[];
}

export function NavigationProvider({ children, items = defaultNavigationItems }: NavigationProviderProps) {
  const pathname = usePathname();

  const currentItem = useMemo(() => {
    // First check top-level items
    if (pathname?.startsWith('/docs/')) {
      const docsItem = items.find((item) => item.title === 'Documentation');
      const docsSubItem = docsItem?.items?.find((item) => item.href === '/docs/');
      return { item: docsSubItem!, parent: docsItem! };
    }
    const topLevelItem = items.find((item) => item.href === pathname);
    if (topLevelItem) {
      return { item: topLevelItem, parent: null };
    }

    // Then check sub-items
    for (const item of items) {
      if (item.items) {
        for (const subItem of item.items) {
          if (subItem.href === pathname) {
            return { item: subItem, parent: item };
          }
          // Check for sub-sub-items
          if (subItem.items) {
            const subSubItem = subItem.items.find((subSubItem) => subSubItem.href === pathname);
            if (subSubItem) {
              return { item: subSubItem, parent: subItem };
            }
          }
        }
      }
    }

    // Default to Chat if no match found
    return { item: items[0], parent: null };
  }, [pathname, items]);

  // Update document title based on currentItem
  useEffect(() => {
    const parts = [];

    // Add current item title
    parts.push(currentItem.item.title);

    // Add parent title if exists
    if (currentItem.parent) {
      parts.push(currentItem.parent.title);
    }

    // Add base title
    parts.push('Bodhi App | Run LLMs Locally');

    // Set document title
    document.title = parts.join(' | ');
  }, [currentItem]);

  const value = useMemo(
    () => ({
      currentPath: pathname || '',
      currentItem,
      navigationItems: items,
    }),
    [pathname, currentItem, items]
  );

  return <NavigationContext.Provider value={value}>{children}</NavigationContext.Provider>;
}

export const useNavigation = () => useContext(NavigationContext);
