import { useEffect } from 'react';

import { createFileRoute, useNavigate } from '@tanstack/react-router';

import AppInitializer from '@/components/AppInitializer';
import { useGetAppInfo } from '@/hooks/info';
import { ROUTE_MODELS } from '@/lib/constants';

import { LocalDiscoveryScreen } from './-components/LocalDiscoveryScreen';

export const Route = createFileRoute('/models/explore/local/')({
  component: LocalDiscoverPage,
});

/**
 * The local-model catalog relies on downloading GGUFs, which HubService rejects in MultiTenant.
 * Hide the entire feature there — redirect to My Models if a multi-tenant user reaches the route.
 */
function MultiTenantGuard({ children }: { children: React.ReactNode }) {
  const { data: appInfo } = useGetAppInfo();
  const navigate = useNavigate();
  const isMultiTenant = appInfo?.deployment === 'multi_tenant';

  useEffect(() => {
    if (isMultiTenant) navigate({ to: ROUTE_MODELS });
  }, [isMultiTenant, navigate]);

  if (isMultiTenant) return null;
  return <>{children}</>;
}

export default function LocalDiscoverPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <MultiTenantGuard>
        <LocalDiscoveryScreen />
      </MultiTenantGuard>
    </AppInitializer>
  );
}
