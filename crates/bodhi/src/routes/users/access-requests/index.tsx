import { createFileRoute } from '@tanstack/react-router';

import AccessRequestsPage from '@/app/users/access-requests/page';

export const Route = createFileRoute('/users/access-requests/')({
  component: AccessRequestsPage,
});
