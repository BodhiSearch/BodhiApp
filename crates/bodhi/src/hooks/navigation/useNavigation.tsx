import { ReactNode, createContext, useContext, useEffect, useMemo } from 'react';

import {
  BookOpen,
  BookText,
  Cog,
  Database,
  Download,
  FileJson,
  FilePlus2,
  Files,
  Key,
  MessageSquare,
  Plug,
  Settings,
  Settings2,
  Users,
  Play,
  Wrench,
} from 'lucide-react';
import { useLocation } from '@tanstack/react-router';

import { NavigationItem } from '@/types/navigation';

// Rename and export the default navigation items
export const defaultNavigationItems: NavigationItem[] = [
  {
    title: 'Chat',
    href: '/chat/',
    description: 'AI Chat Interface',
    icon: MessageSquare,
  },
  {
    title: 'Models',
    icon: Database,
    items: [
      {
        title: 'Model Aliases',
        href: '/models/',
        description: 'Configure and manage model aliases',
        icon: Settings2,
        items: [
          {
            title: 'New Model Alias',
            href: '/models/alias/new/',
            description: 'Create a new model alias',
            icon: FilePlus2,
            skip: true,
          },
          {
            title: 'Edit Model Alias',
            href: '/models/alias/edit/',
            description: 'Edit a model alias',
            icon: FilePlus2,
            skip: true,
          },
          {
            title: 'New API Model',
            href: '/models/api/new/',
            description: 'Create a new API model',
            icon: FilePlus2,
            skip: true,
          },
          {
            title: 'Edit API Model',
            href: '/models/api/edit/',
            description: 'Edit an API model',
            icon: FilePlus2,
            skip: true,
          },
        ],
      },
      {
        title: 'Model Files',
        href: '/models/files/',
        description: 'Browse and manage model files',
        icon: Files,
      },
      {
        title: 'Model Downloads',
        href: '/models/files/pull/',
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
        href: '/settings/',
        description: 'Manage application settings',
        icon: Cog,
      },
      {
        title: 'API Tokens',
        href: '/tokens/',
        description: 'Manage API access tokens',
        icon: Key,
      },
      {
        title: 'Toolsets',
        href: '/toolsets/',
        description: 'Configure AI toolsets',
        icon: Wrench,
        items: [
          {
            title: 'New Toolset',
            href: '/toolsets/new/',
            description: 'Create a new toolset',
            icon: Wrench,
            skip: true,
          },
          {
            title: 'Edit Toolset',
            href: '/toolsets/edit/',
            description: 'Edit a toolset',
            icon: Wrench,
            skip: true,
          },
          {
            title: 'Admin Toolsets',
            href: '/toolsets/admin/',
            description: 'Admin toolsets',
            icon: Wrench,
            skip: true,
          },
        ],
      },
      {
        title: 'MCP',
        href: '/mcps/',
        description: 'Manage MCP servers and instances',
        icon: Plug,
        items: [
          {
            title: 'MCP Servers',
            href: '/mcps/servers/',
            description: 'Browse MCP server registry',
            icon: Plug,
            skip: true,
          },
          {
            title: 'New MCP',
            href: '/mcps/new/',
            description: 'Add an MCP instance',
            icon: Plug,
            skip: true,
          },
          {
            title: 'New MCP Server',
            href: '/mcps/servers/new/',
            description: 'Register a new MCP server',
            icon: Plug,
            skip: true,
          },
          {
            title: 'Edit MCP Server',
            href: '/mcps/servers/edit/',
            description: 'Edit an MCP server',
            icon: Plug,
            skip: true,
          },
          {
            title: 'Playground',
            href: '/mcps/playground/',
            description: 'Test MCP tools',
            icon: Play,
            skip: true,
          },
        ],
      },
      {
        title: 'Manage Users',
        href: '/users/',
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
        href: 'https://getbodhi.app/docs/',
        description: 'User guides and documentation',
        icon: BookOpen,
        target: '_blank',
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
  const { pathname } = useLocation();

  const currentItem = useMemo(() => {
    // First check top-level items
    if (pathname?.startsWith('/users/')) {
      const settingsItem = items.find((item) => item.title === 'Settings');
      const usersSubItem = settingsItem?.items?.find((item) => item.href === '/users/');
      return { item: usersSubItem!, parent: settingsItem! };
    }
    if (pathname?.startsWith('/mcps/')) {
      const settingsItem = items.find((item) => item.title === 'Settings');
      const mcpSubItem = settingsItem?.items?.find((item) => item.href === '/mcps/');
      return { item: mcpSubItem!, parent: settingsItem! };
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
