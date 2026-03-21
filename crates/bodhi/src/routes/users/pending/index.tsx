import { createFileRoute } from '@tanstack/react-router';

import PendingUsersPage from '@/app/users/pending/page';

export const Route = createFileRoute('/users/pending/')({
  component: PendingUsersPage,
});
