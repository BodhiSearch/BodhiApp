import { createFileRoute } from '@tanstack/react-router';

import AppInitializer from '@/components/AppInitializer';

import { ManageUsersV2 } from './-components/ManageUsersV2';

export const Route = createFileRoute('/users/')({
  component: UsersPage,
});

export default function UsersPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true} minRole="manager">
      <ManageUsersV2 />
    </AppInitializer>
  );
}
