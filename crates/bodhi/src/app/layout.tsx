import type { Metadata, Viewport } from 'next';
import { Inter as FontSans } from 'next/font/google';
import './globals.css';
import { cn } from '@/lib/utils';
import ClientProviders from '@/components/ClientProviders';
import { Toaster } from '@/components/ui/toaster';
import '@/styles/syntax-highlighter.css';
import { NavigationProvider } from '@/hooks/use-navigation';
import type { Page } from '@/types/models';
import { SidebarProvider } from '@/components/ui/sidebar';

const fontSans = FontSans({
  subsets: ['latin'],
  variable: '--font-sans',
});

const APP_NAME = 'Bodhi App';
const APP_DEFAULT_TITLE = 'Bodhi App - Run LLMs Locally';
const APP_TITLE_TEMPLATE = '%s - Bodhi App';
const APP_DESCRIPTION = 'Bodhi App - Run LLMs Locally';

export const metadata: Metadata = {
  applicationName: APP_NAME,
  title: {
    default: APP_DEFAULT_TITLE,
    template: APP_TITLE_TEMPLATE,
  },
  description: APP_DESCRIPTION,
  manifest: '/manifest.json',
  appleWebApp: {
    capable: true,
    statusBarStyle: 'default',
    title: APP_DEFAULT_TITLE,
  },
  formatDetection: {
    telephone: false,
  },
  openGraph: {
    type: 'website',
    siteName: APP_NAME,
    title: {
      default: APP_DEFAULT_TITLE,
      template: APP_TITLE_TEMPLATE,
    },
    description: APP_DESCRIPTION,
  },
  twitter: {
    card: 'summary',
    title: {
      default: APP_DEFAULT_TITLE,
      template: APP_TITLE_TEMPLATE,
    },
    description: APP_DESCRIPTION,
  },
};

export const viewport: Viewport = {
  themeColor: '#f69435',
};

const pages: Page[] = [
  {
    title: 'Home',
    url: '/ui/home',
    iconName: 'home',
  },
  {
    title: 'Chat',
    url: '/ui/chat',
    iconName: 'messageSquareText',
  },
  {
    title: 'Model Aliases',
    url: '/ui/models',
    iconName: 'file',
  },
  {
    title: 'New Model Alias',
    url: '/ui/models/new',
    iconName: 'file',
  },
  {
    title: 'Model Files',
    url: '/ui/modelfiles',
    iconName: 'file',
  },
];

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
        <ClientProviders>
          <NavigationProvider pages={pages}>
            <SidebarProvider>
              {children}
              <Toaster />
            </SidebarProvider>
          </NavigationProvider>
        </ClientProviders>
      </body>
    </html>
  );
}
