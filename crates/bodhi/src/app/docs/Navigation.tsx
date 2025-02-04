'use client';

import { SidebarNav } from '@/app/docs/sidebar-nav';
import type { NavigationProps } from '@/app/docs/types';
import { usePathname } from 'next/navigation';
import { cn } from '@/lib/utils';

const DOCS_BASE_PATH = '/docs';

export function Navigation({ items }: NavigationProps) {
  const pathname = usePathname();

  // Convert our doc items to nav items
  const navItems = items.map((item) => ({
    title: item.title,
    href: `${DOCS_BASE_PATH}/${item.slug}`,
    isFolder: item.children && item.children.length > 0,
    className: cn(
      'hover:text-accent-foreground',
      pathname === `${DOCS_BASE_PATH}/${item.slug}` &&
        'text-accent-foreground font-medium'
    ),
    children: item.children?.map((child) => ({
      title: child.title,
      href: `${DOCS_BASE_PATH}/${child.slug}`,
      className: cn(
        'hover:text-accent-foreground',
        pathname === `${DOCS_BASE_PATH}/${child.slug}` &&
          'text-accent-foreground font-medium'
      ),
    })),
  }));

  return (
    <SidebarNav
      items={navItems}
      className="p-4"
      data-testid="main-navigation"
    />
  );
}
