import { useEffect } from 'react';

import { createFileRoute, useNavigate } from '@tanstack/react-router';
import { z } from 'zod';

import AppInitializer from '@/components/AppInitializer';
import { useGetAppInfo } from '@/hooks/info';
import { ROUTE_MODELS } from '@/lib/constants';

import { ExploreProvidersScreen } from './-components/ExploreProvidersScreen';

// `select` is the cross-link target from the API Models page ("Served by" → a provider).
const providersSearchSchema = z.object({ select: z.string().optional() });

export const Route = createFileRoute('/models/explore/api-providers/')({
  validateSearch: providersSearchSchema,
  component: ExploreProvidersPage,
});

/**
 * The API-provider catalog is a browse-and-configure surface for hosted models. Mirror Explore ·
 * Local Models: hide it in MultiTenant (where local-model features are unavailable) — redirect to
 * My Models if a multi-tenant user reaches the route.
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

export default function ExploreProvidersPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <MultiTenantGuard>
        <ExploreProvidersScreen />
      </MultiTenantGuard>
    </AppInitializer>
  );
}
