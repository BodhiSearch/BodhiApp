'use client';

import * as React from 'react';
import { usePathname } from 'next/navigation';
import { useContext, useMemo } from 'react';
import { NavigationItem } from '@/types/navigation';
import {
  Home,
  MessageSquare,
  Database,
  Settings2,
  Files,
  Download,
  Key,
  FlaskRound,
  Settings,
} from 'lucide-react';

const navigationItems: NavigationItem[] = [
  {
    title: 'Home',
    href: '/ui/home/',
    description: 'Dashboard and overview',
    icon: Home,
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
      },
      {
        title: 'Model Files',
        href: '/ui/modelfiles/',
        description: 'Browse and manage model files',
        icon: Files,
      },
      {
        title: 'Download Models',
        href: '/ui/pull/',
        description: 'Download new models',
        icon: Download,
      },
    ],
  },
  {
    title: 'Playground',
    icon: FlaskRound,
    items: [
      {
        title: 'Chat',
        href: '/ui/chat/',
        description: 'AI Chat Interface',
        icon: MessageSquare,
      },
    ],
  },
  {
    title: 'Settings',
    icon: Settings,
    items: [
      {
        title: 'API Tokens',
        href: '/ui/tokens/',
        description: 'Manage API access tokens',
        icon: Key,
        authRequired: true,
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

const NavigationContext = React.createContext<NavigationContextType>({
  currentPath: '',
  currentItem: {
    item: {} as NavigationItem,
    parent: null,
  },
  navigationItems: [],
});

interface NavigationProviderProps {
  children: React.ReactNode;
}

export function NavigationProvider({ children }: NavigationProviderProps) {
  const pathname = usePathname();

  const currentItem = useMemo(() => {
    // First check top-level items
    const topLevelItem = navigationItems.find((item) => item.href === pathname);
    if (topLevelItem) {
      return { item: topLevelItem, parent: null };
    }

    // Then check sub-items
    for (const item of navigationItems) {
      if (item.items) {
        const subItem = item.items.find((subItem) => subItem.href === pathname);
        if (subItem) {
          return { item: subItem, parent: item };
        }
      }
    }

    // Default to Home if no match found
    return { item: navigationItems[0], parent: null };
  }, [pathname]);

  const value = useMemo(
    () => ({
      currentPath: pathname,
      currentItem,
      navigationItems,
    }),
    [pathname, currentItem]
  );

  return (
    <NavigationContext.Provider value={value}>
      {children}
    </NavigationContext.Provider>
  );
}

export const useNavigation = () => useContext(NavigationContext);
