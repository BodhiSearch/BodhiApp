import { useMemo } from 'react';

import { createRootRoute, Outlet, useLocation, useNavigate } from '@tanstack/react-router';
import 'prismjs/themes/prism-tomorrow.css';
import '@/styles/globals.css';
import '@/styles/view-transitions.css';

import ClientProviders from '@/components/ClientProviders';
import { AppShell, type ShellFooterUser } from '@/components/shell';
import { BareLayout } from '@/components/shell/BareLayout';
import { isBareRoute, isFullscreenRoute } from '@/components/shell/resolveShellRoute';
import { ShellChromeProvider, useShellSlots } from '@/components/shell/ShellChromeContext';
import { useShellSection } from '@/components/shell/useShellSection';
import { ThemeProvider } from '@/components/ThemeProvider';
import { Toaster } from '@/components/ui/toaster';
import { useLogoutHandler } from '@/hooks/auth';
import { NavigationProvider, defaultNavigationItems } from '@/hooks/navigation';
import { useGetUser } from '@/hooks/users';
import { ROUTE_DEFAULT, ROUTE_LOGIN } from '@/lib/constants';
import { handleSmartRedirect } from '@/lib/utils';

export const Route = createRootRoute({
  component: RootLayout,
});

/**
 * Decides the layout for the current route:
 * - Fullscreen routes (the setup wizard) render the Outlet directly with NO chrome — the wizard
 *   owns its own centered column, logo, stepper, and theme toggle.
 * - Bare routes (login / auth / request-access / oauth / root redirectors AND the standalone OAuth
 *   access-request review) render OUTSIDE the AppShell, through the slim-topbar `BareLayout`. (The
 *   central prefix switches in resolveShellRoute are interim — a route-declared layout seam is the
 *   deferred follow-up; see screen-v2/techdebt.md.)
 * - App routes render inside the V2 AppShell, with the active section/subPage derived from the
 *   pathname. A migrated screen contributes breadcrumb/headerActions/rail via `useShellChrome`
 *   (ShellChromeContext); unmigrated screens render their existing content inside the shell.
 */
function RootShell() {
  const { pathname } = useLocation();
  const slots = useShellSlots();
  const { section, subPage } = useShellSection();
  const navigate = useNavigate();

  // App-shell-only data: skip the fetch on bare/fullscreen routes (login/setup) where the
  // shell doesn't render and the user may not be logged in yet.
  const inAppShell = !isFullscreenRoute(pathname) && !isBareRoute(pathname);
  const { data: userInfo } = useGetUser({ enabled: inAppShell });
  const { logout, isLoading: logoutPending } = useLogoutHandler({
    onSuccess: (response) => {
      const redirectUrl = response.data?.location || ROUTE_DEFAULT;
      handleSmartRedirect(redirectUrl, navigate);
    },
    onError: () => {
      // Best-effort cleanup so a server-side error doesn't leave the client logged-in-looking.
      localStorage.clear();
      sessionStorage.clear();
      document.cookie.split(';').forEach((c) => {
        const eq = c.indexOf('=');
        const name = eq > -1 ? c.substr(0, eq) : c;
        document.cookie = name + '=;expires=Thu, 01 Jan 1970 00:00:00 GMT;path=/';
      });
      handleSmartRedirect(ROUTE_LOGIN, navigate);
    },
  });

  const shellUser: ShellFooterUser = useMemo(() => {
    if (userInfo?.auth_status !== 'logged_in') return {};
    const first = userInfo.first_name?.trim() ?? '';
    const last = userInfo.last_name?.trim() ?? '';
    const fullName = [first, last].filter(Boolean).join(' ');
    const display = fullName || userInfo.username;
    const initials = ((first[0] ?? '') + (last[0] ?? '')).toUpperCase() || display.slice(0, 2).toUpperCase();
    return {
      name: display,
      email: userInfo.username,
      role: roleLabel(userInfo.role),
      initials,
    };
  }, [userInfo]);

  if (isFullscreenRoute(pathname)) {
    return <Outlet />;
  }

  if (isBareRoute(pathname)) {
    return (
      <BareLayout>
        <Outlet />
      </BareLayout>
    );
  }

  return (
    <AppShell
      section={section}
      subPage={subPage}
      user={shellUser}
      onLogout={() => logout()}
      logoutPending={logoutPending}
      {...slots}
      // section/subPage come from route staticData (useShellSection). Default the content to "flush"
      // and persist column widths per-section; a screen can still override either via useShellChrome.
      contentClass={slots.contentClass ?? 'flush'}
      resizeKey={slots.resizeKey ?? section}
    >
      <Outlet />
    </AppShell>
  );
}

// Map backend ResourceRole values (e.g. "resource_admin") to short labels for the footer chip.
function roleLabel(role: string | null | undefined): string {
  if (!role) return '';
  const trimmed = role.replace(/^resource_/, '').replace(/_/g, ' ');
  return trimmed
    .split(' ')
    .filter(Boolean)
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(' ');
}

function RootLayout() {
  return (
    <ThemeProvider defaultTheme="system" storageKey="bodhi-ui-theme">
      <ClientProviders>
        <NavigationProvider items={defaultNavigationItems}>
          <ShellChromeProvider>
            <div data-testid="root-layout">
              <RootShell />
              <Toaster />
            </div>
          </ShellChromeProvider>
        </NavigationProvider>
      </ClientProviders>
    </ThemeProvider>
  );
}
