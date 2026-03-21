import { createFileRoute } from '@tanstack/react-router';

import BrowserExtensionPage from '@/app/setup/browser-extension/page';

export const Route = createFileRoute('/setup/browser-extension/')({
  component: BrowserExtensionPage,
});
