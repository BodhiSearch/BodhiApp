import { useEffect } from 'react';

import { createFileRoute, useNavigate } from '@tanstack/react-router';

import AppInitializer from '@/components/AppInitializer';
import { useGetAppInfo } from '@/hooks/info';
import { ROUTE_MODELS } from '@/lib/constants';

import { ExploreApiScreen } from './-components/ExploreApiScreen';

export const Route = createFileRoute('/models/explore/api/')({
  component: ExploreApiPage,
});

/**
 * The API-model catalog is a browse-and-configure surface for hosted models. Mirror Explore · Local
 * Models: hide it in MultiTenant — redirect to My Models if a multi-tenant user reaches the route.
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

export default function ExploreApiPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <MultiTenantGuard>
        <ExploreApiScreen />
      </MultiTenantGuard>
    </AppInitializer>
  );
}
