import { createRootRoute, Outlet } from '@tanstack/react-router';
import 'prismjs/themes/prism-tomorrow.css';
import '@/styles/globals.css';

import ClientProviders from '@/components/ClientProviders';
import { AppHeader } from '@/components/navigation/AppHeader';
import { ThemeProvider } from '@/components/ThemeProvider';
import { Toaster } from '@/components/ui/toaster';
import { NavigationProvider, defaultNavigationItems } from '@/hooks/navigation';

export const Route = createRootRoute({
  component: RootLayout,
});

function RootLayout() {
  return (
    <ThemeProvider defaultTheme="system" storageKey="bodhi-ui-theme">
      <ClientProviders>
        <NavigationProvider items={defaultNavigationItems}>
          <div className="flex min-h-screen flex-col" data-testid="root-layout">
            <AppHeader />
            <main className="flex-1" data-testid="app-main">
              <Outlet />
            </main>
            <Toaster />
          </div>
        </NavigationProvider>
      </ClientProviders>
    </ThemeProvider>
  );
}
