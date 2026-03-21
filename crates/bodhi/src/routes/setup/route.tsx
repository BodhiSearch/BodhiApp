import { createFileRoute, Outlet } from '@tanstack/react-router';

import SetupLayoutComponent from '@/app/setup/layout';

export const Route = createFileRoute('/setup')({
  component: () => (
    <SetupLayoutComponent>
      <Outlet />
    </SetupLayoutComponent>
  ),
});
