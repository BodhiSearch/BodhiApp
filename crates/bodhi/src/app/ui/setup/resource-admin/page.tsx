'use client';

import AppInitializer from '@/components/AppInitializer';
import { AuthCard } from '@/components/auth/AuthCard';
import { ENDPOINT_APP_LOGIN } from '@/hooks/useQuery';

function ResourceAdminContent() {
  return (
    <AuthCard
      title="Resource Admin Setup"
      description="You will be made the app admin using the account you log in with."
      actions={[
        {
          label: 'Log In',
          href: ENDPOINT_APP_LOGIN,
        },
      ]}
    />
  );
}

export default function ResourceAdminPage() {
  return (
    <AppInitializer allowedStatus="resource-admin" authenticated={false}>
      <div className="flex-1 pt-12 sm:pt-16" data-testid="resource-admin-page">
        <ResourceAdminContent />
      </div>
    </AppInitializer>
  );
}
