import { createFileRoute } from '@tanstack/react-router';

import SettingsPage from '@/app/settings/page';

export const Route = createFileRoute('/settings/')({
  component: SettingsPage,
});
