import { useEffect } from 'react';

import { useNavigate } from '@tanstack/react-router';

import { useGetAppInfo } from '@/hooks/info';
import { ROUTE_MODELS } from '@/lib/constants';

/**
 * Explore catalogs (API Models, API Providers, Local Models) are browse-and-configure surfaces tied
 * to local-model/download features that HubService rejects in MultiTenant. Hide them there — redirect
 * to My Models if a multi-tenant user reaches the route.
 */
export function MultiTenantGuard({ children }: { children: React.ReactNode }) {
  const { data: appInfo } = useGetAppInfo();
  const navigate = useNavigate();
  const isMultiTenant = appInfo?.deployment === 'multi_tenant';

  useEffect(() => {
    if (isMultiTenant) navigate({ to: ROUTE_MODELS });
  }, [isMultiTenant, navigate]);

  if (isMultiTenant) return null;
  return <>{children}</>;
}
