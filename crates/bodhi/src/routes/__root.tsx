import { createRootRoute, Outlet, useLocation } from '@tanstack/react-router';
import 'prismjs/themes/prism-tomorrow.css';
import '@/styles/globals.css';
import '@/styles/view-transitions.css';

import ClientProviders from '@/components/ClientProviders';
import { AppShell } from '@/components/shell';
import { BareLayout } from '@/components/shell/BareLayout';
import { isBareRoute, resolveShellRoute } from '@/components/shell/resolveShellRoute';
import { ShellSlotsProvider, useShellSlots } from '@/components/shell/ShellSlotsContext';
import { ThemeProvider } from '@/components/ThemeProvider';
import { Toaster } from '@/components/ui/toaster';
import { NavigationProvider, defaultNavigationItems } from '@/hooks/navigation';

export const Route = createRootRoute({
  component: RootLayout,
});

/**
 * Decides the layout for the current route:
 * - Bare routes (setup / login / auth / request-access / oauth / root redirectors AND the
 *   standalone OAuth access-request review) render OUTSIDE the AppShell, through the slim-topbar
 *   `BareLayout`. (The central `BARE_PREFIXES` switch in resolveShellRoute is interim — a
 *   route-declared layout seam is the deferred follow-up; see screen-v2/techdebt.md.)
 * - App routes render inside the V2 AppShell, with the active section/subPage derived from the
 *   pathname. A migrated screen contributes breadcrumb/headerActions/rail via `useShellChrome`
 *   (ShellSlotsContext); unmigrated screens render their existing content inside the shell.
 */
function RootShell() {
  const { pathname } = useLocation();
  const slots = useShellSlots();

  if (isBareRoute(pathname)) {
    return (
      <BareLayout>
        <Outlet />
      </BareLayout>
    );
  }

  const resolved = resolveShellRoute(pathname) ?? { section: '', subPage: null };
  return (
    <AppShell section={resolved.section} subPage={resolved.subPage} contentClass="flush" {...slots}>
      <Outlet />
    </AppShell>
  );
}

function RootLayout() {
  return (
    <ThemeProvider defaultTheme="system" storageKey="bodhi-ui-theme">
      <ClientProviders>
        <NavigationProvider items={defaultNavigationItems}>
          <ShellSlotsProvider>
            <div data-testid="root-layout">
              <RootShell />
              <Toaster />
            </div>
          </ShellSlotsProvider>
        </NavigationProvider>
      </ClientProviders>
    </ThemeProvider>
  );
}
