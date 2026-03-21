import { createFileRoute } from '@tanstack/react-router';

import TenantsPage from '@/app/setup/tenants/page';

export const Route = createFileRoute('/setup/tenants/')({
  component: TenantsPage,
});
