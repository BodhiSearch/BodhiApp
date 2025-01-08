'use client';

import * as React from 'react';
import { usePathname } from 'next/navigation';
import type { Page } from '@/types/models';
import { useContext, useMemo } from 'react';

interface NavigationContextType {
  currentPath: string;
  currentPage: Page;
  pages: Page[];
}

const NavigationContext = React.createContext<NavigationContextType>({
  currentPath: '',
  currentPage: {} as Page,
  pages: [],
});

interface NavigationProviderProps {
  children: React.ReactNode;
  pages: Page[];
}

export function NavigationProvider({
  children,
  pages,
}: NavigationProviderProps) {
  const pathname = usePathname();

  const currentPage = useMemo(() => {
    return pages.find((page) => pathname.startsWith(page.url)) || pages[0];
  }, [pathname, pages]);

  const value = useMemo(
    () => ({
      currentPath: pathname,
      currentPage,
      pages,
    }),
    [pathname, currentPage, pages]
  );

  return (
    <NavigationContext.Provider value={value}>
      {children}
    </NavigationContext.Provider>
  );
}

export const useNavigation = () => useContext(NavigationContext);
