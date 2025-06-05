'use client';

import { SidebarNav } from '@/app/docs/nav';
import type { NavigationProps } from '@/app/docs/types';
import { usePathname } from '@/lib/navigation';

export const DOCS_BASE_PATH = '/docs';

export function Navigation({ items }: NavigationProps) {
  const pathname = usePathname();

  // Convert our doc items to nav items with selection state
  const navItems = items.map((item) => {
    const itemPath = `${DOCS_BASE_PATH}/${item.slug}/`;
    const isSelected = pathname === itemPath;

    // Check if any children are selected
    const children = item.children?.map((child) => {
      const childPath = `${DOCS_BASE_PATH}/${child.slug}/`;
      return {
        title: child.title,
        href: childPath,
        selected: pathname === childPath,
      };
    });

    return {
      title: item.title,
      href: itemPath,
      selected: isSelected || children?.some((child) => child.selected),
      children,
    };
  });

  return (
    <SidebarNav
      items={navItems}
      className="p-4"
      data-testid="main-navigation"
    />
  );
}
