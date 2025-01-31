'use client';

import { usePathname } from 'next/navigation';
import { AppNavigation } from '@/components/navigation/AppNavigation';
import { AppBreadcrumb } from '@/components/navigation/AppBreadcrumb';

export function AppHeader() {
  const pathname = usePathname();
  const shouldRenderHeader = !pathname?.startsWith('/ui/setup/');

  if (!shouldRenderHeader) {
    return null;
  }

  return (
    <header
      className="sticky top-0 z-50 h-16 border-b bg-header-elevated/90 backdrop-blur-sm"
      data-testid="app-header"
    >
      <div className="flex h-full items-center">
        <AppNavigation />
        <AppBreadcrumb />
      </div>
    </header>
  );
} 