'use client';

import { Inter as FontSans } from 'next/font/google';
import '@/app/globals.css';
import { cn } from '@/lib/utils';
import ClientProviders from '@/components/ClientProviders';
import { Toaster } from '@/components/ui/toaster';
import '@/styles/syntax-highlighter.css';
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
    <html lang="en">
      <body
        className={cn(
          'min-h-screen bg-background font-sans antialiased',
          fontSans.variable
        )}
      >
        <ThemeProvider defaultTheme="system" storageKey="bodhi-ui-theme">
          <ClientProviders>
            <NavigationProvider>
              <div className="flex flex-col h-screen">
                <header className="flex h-16 items-center border-b bg-background">
                  <AppNavigation />
                  <AppBreadcrumb />
                </header>
                <main className="flex flex-1 w-full">{children}</main>
                <Toaster />
              </div>
            </NavigationProvider>
          </ClientProviders>
        </ThemeProvider>
      </body>
    </html>
  );
}
