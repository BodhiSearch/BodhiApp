import { createFileRoute } from '@tanstack/react-router';

import RequestAccessPage from '@/app/request-access/page';

export const Route = createFileRoute('/request-access/')({
  component: RequestAccessPage,
});
