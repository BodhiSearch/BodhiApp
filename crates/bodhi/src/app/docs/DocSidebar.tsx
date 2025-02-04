'use client';

import { Navigation } from '@/app/docs/Navigation';
import { Button } from '@/components/ui/button';
import { Sheet, SheetContent, SheetTrigger } from '@/components/ui/sheet';
import { Menu } from 'lucide-react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import type { NavItem } from './types';

function getCurrentFolderNavigation(
  navigation: NavItem[],
  currentPath: string | null
): NavItem[] {
  if (!currentPath || currentPath === '/docs') {
    return navigation;
  }

  const pathParts = currentPath.replace('/docs/', '').split('/');
  let currentLevel = navigation;
  let result = navigation;

  // Traverse the navigation tree to find the current folder's items
  for (const part of pathParts) {
    const found = currentLevel.find(
      (item) => item.slug === part || item.slug.endsWith('/' + part)
    );
    if (found && found.children) {
      currentLevel = found.children;
      result = found.children;
    }
  }

  return result;
}

interface DocSidebarProps {
  navigation: NavItem[];
}

export function DocSidebar({ navigation }: DocSidebarProps) {
  const pathname = usePathname();
  const currentFolderNav = getCurrentFolderNavigation(navigation, pathname);

  const sidebarContent = (
    <div className="py-4" data-testid="sidebar-content">
      <Navigation items={navigation} />
      {pathname !== '/docs' && currentFolderNav.length > 0 && (
        <div className="mt-6 px-4" data-testid="section-navigation">
          <h2 className="text-sm font-semibold mb-2 text-muted-foreground">
            In This Section
          </h2>
          <ul className="space-y-2">
            {currentFolderNav.map((item) => (
              <li key={item.slug} data-testid={`section-item-${item.slug}`}>
                <Link
                  href={`/docs/${item.slug}`}
                  className="text-sm text-muted-foreground hover:text-foreground block py-1"
                >
                  {item.title}
                </Link>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );

  return (
    <>
      {/* Desktop Sidebar */}
      <aside
        className="hidden lg:block w-80 shrink-0 border-r bg-background"
        aria-label="Documentation navigation"
        data-testid="desktop-sidebar"
      >
        <div className="h-16 border-b px-6 flex items-center">
          <h1 className="text-lg font-semibold">Documentation</h1>
        </div>
        {sidebarContent}
      </aside>

      {/* Mobile Sidebar */}
      <Sheet>
        <SheetTrigger asChild>
          <Button
            variant="ghost"
            size="icon"
            className="lg:hidden fixed left-4 top-4 z-[50]"
            aria-label="Open documentation navigation"
            data-testid="mobile-sidebar-trigger"
          >
            <Menu className="h-4 w-4" />
          </Button>
        </SheetTrigger>
        <SheetContent
          side="left"
          className="w-80 p-0"
          data-testid="mobile-sidebar"
        >
          <div className="h-16 border-b px-6 flex items-center">
            <h2 className="text-lg font-semibold">Documentation</h2>
          </div>
          {sidebarContent}
        </SheetContent>
      </Sheet>
    </>
  );
}
