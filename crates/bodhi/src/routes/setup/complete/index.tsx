import { createFileRoute } from '@tanstack/react-router';

import SetupCompletePage from '@/app/setup/complete/page';

export const Route = createFileRoute('/setup/complete/')({
  component: SetupCompletePage,
});
