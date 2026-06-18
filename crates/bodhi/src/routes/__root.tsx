import { createRootRoute, Outlet, useLocation } from '@tanstack/react-router';
import 'prismjs/themes/prism-tomorrow.css';
import '@/styles/globals.css';

import ClientProviders from '@/components/ClientProviders';
import { AppShell } from '@/components/shell';
import { isBareRoute, resolveShellRoute } from '@/components/shell/resolveShellRoute';
import { ThemeProvider } from '@/components/ThemeProvider';
import { Toaster } from '@/components/ui/toaster';
import { NavigationProvider, defaultNavigationItems } from '@/hooks/navigation';

export const Route = createRootRoute({
  component: RootLayout,
});

/**
 * Decides the layout for the current route:
 * - Bare routes (setup / login / auth / request-access / root redirectors) render chrome-less —
 *   they keep their own current layouts and are out of scope for the V2 shell migration.
 * - App routes render inside the V2 AppShell, with the active section/subPage derived from the
 *   pathname. During coexistence, unmigrated screens render their existing content inside the shell.
 */
function RootShell() {
  const { pathname } = useLocation();

  if (isBareRoute(pathname)) {
    return (
      <main data-testid="app-main">
        <Outlet />
      </main>
    );
  }

  const resolved = resolveShellRoute(pathname) ?? { section: '', subPage: null };
  return (
    <AppShell section={resolved.section} subPage={resolved.subPage} contentClass="flush">
      <Outlet />
    </AppShell>
  );
}

function RootLayout() {
  return (
    <ThemeProvider defaultTheme="system" storageKey="bodhi-ui-theme">
      <ClientProviders>
        <NavigationProvider items={defaultNavigationItems}>
          <div data-testid="root-layout">
            <RootShell />
            <Toaster />
          </div>
        </NavigationProvider>
      </ClientProviders>
    </ThemeProvider>
  );
}
