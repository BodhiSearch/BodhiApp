'use client';

import { Inter as FontSans } from 'next/font/google';
import '@/app/globals.css';
import '@/styles/syntax-highlighter.css';
import { cn } from '@/lib/utils';
import ClientProviders from '@/components/ClientProviders';
import { Toaster } from '@/components/ui/toaster';
import { NavigationProvider } from '@/hooks/use-navigation';
import { AppNavigation } from '@/components/navigation/AppNavigation';
import { AppBreadcrumb } from '@/components/navigation/AppBreadcrumb';
import { ThemeProvider } from '@/components/ThemeProvider';

const fontSans = FontSans({
  subsets: ['latin'],
  variable: '--font-sans',
});

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body
        className={cn(
          'min-h-screen bg-background text-foreground font-sans antialiased',
          fontSans.variable
        )}
        suppressHydrationWarning
      >
        <ThemeProvider defaultTheme="system" storageKey="bodhi-ui-theme">
          <ClientProviders>
            <NavigationProvider>
              <div
                className="min-h-screen flex flex-col"
                data-testid="root-layout"
              >
                <header
                  className="sticky top-0 z-50 flex h-16 items-center border-b border-border bg-header-elevated/90 backdrop-blur-sm"
                  data-testid="app-header"
                >
                  <AppNavigation />
                  <AppBreadcrumb />
                </header>
                <main className="flex-1" data-testid="app-main">
                  {children}
                </main>
                <Toaster />
              </div>
            </NavigationProvider>
          </ClientProviders>
        </ThemeProvider>
      </body>
    </html>
  );
}
