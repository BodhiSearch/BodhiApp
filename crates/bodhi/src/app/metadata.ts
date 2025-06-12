export const APP_NAME = 'Bodhi App';
export const APP_DEFAULT_TITLE = 'Bodhi App - Run LLMs Locally';
export const APP_TITLE_TEMPLATE = '%s - Bodhi App';
export const APP_DESCRIPTION = 'Bodhi App - Run LLMs Locally';

export const metadata = {
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
} as const;

export const viewport = {
  themeColor: '#f69435',
} as const;
