'use client';

import { SidebarNav } from '@/app/docs/sidebar-nav';
import type { NavigationProps } from '@/app/docs/types';

const DOCS_BASE_PATH = '/docs';

export function Navigation({ items }: NavigationProps) {
  // Convert our doc items to nav items
  const navItems = items.map((item) => ({
    title: item.title,
    href: `${DOCS_BASE_PATH}/${item.slug}`,
    children: item.children?.map((child) => ({
      title: child.title,
      href: `${DOCS_BASE_PATH}/${child.slug}`,
    })),
  }));

  return <SidebarNav items={navItems} className="p-4" />;
}
