'use client';

import { Inter as FontSans } from 'next/font/google';
import '@/app/globals.css';
import { cn } from '@/lib/utils';
import ClientProviders from '@/components/ClientProviders';
import { Toaster } from '@/components/ui/toaster';
import { NavigationProvider, defaultNavigationItems } from '@/hooks/use-navigation';
import { ThemeProvider } from '@/components/ThemeProvider';
import { AppHeader } from '@/components/navigation/AppHeader';

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
        <link
          rel="stylesheet"
          href="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/themes/prism-tomorrow.min.css"
        />
      </head>
      <body
        className={cn('min-h-screen bg-background font-sans antialiased', fontSans.variable)}
        suppressHydrationWarning
      >
        <ThemeProvider defaultTheme="system" storageKey="bodhi-ui-theme">
          <ClientProviders>
            <NavigationProvider items={defaultNavigationItems}>
              <div className="flex min-h-screen flex-col" data-testid="root-layout">
                <AppHeader />
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
