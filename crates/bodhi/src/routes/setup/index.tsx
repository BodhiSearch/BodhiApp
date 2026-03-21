import { createFileRoute } from '@tanstack/react-router';

import SetupPage from '@/app/setup/page';

export const Route = createFileRoute('/setup/')({
  component: SetupPage,
});
