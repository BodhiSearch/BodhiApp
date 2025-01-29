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

// Script to handle initial theme to prevent flash
const themeScript = `
  let theme = window.localStorage.getItem('bodhi-ui-theme')
  if (!theme) {
    theme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
  }
  document.documentElement.classList.add(theme)
`;

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <script dangerouslySetInnerHTML={{ __html: themeScript }} />
      </head>
      <body
        className={cn(
          'min-h-screen bg-background font-sans antialiased',
          fontSans.variable
        )}
        suppressHydrationWarning
      >
        <ThemeProvider defaultTheme="system" storageKey="bodhi-ui-theme">
          <ClientProviders>
            <NavigationProvider>
              <div
                className="flex min-h-screen flex-col"
                data-testid="root-layout"
              >
                <header
                  className="sticky top-0 z-50 h-16 border-b bg-header-elevated/90 backdrop-blur-sm"
                  data-testid="app-header"
                >
                  <div className="flex h-full items-center">
                    <AppNavigation />
                    <AppBreadcrumb />
                  </div>
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
