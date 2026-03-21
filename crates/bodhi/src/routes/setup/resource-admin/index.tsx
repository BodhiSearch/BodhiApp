import { createFileRoute } from '@tanstack/react-router';

import ResourceAdminPage from '@/app/setup/resource-admin/page';

export const Route = createFileRoute('/setup/resource-admin/')({
  component: ResourceAdminPage,
});
