import { createFileRoute } from '@tanstack/react-router';

import UsersPage from '@/app/users/page';

export const Route = createFileRoute('/users/')({
  component: UsersPage,
});
