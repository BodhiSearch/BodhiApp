import { useEffect } from 'react';

import { useNavigate } from '@tanstack/react-router';

import { useGetAppInfo } from '@/hooks/info';
import { ROUTE_MODELS } from '@/lib/constants';

/**
 * Guards local-model surfaces that depend on downloads HubService rejects in MultiTenant — Explore ·
 * Local Models (browse-and-pull) and New Local Model (create-from-GGUF). Hide there — redirect to My
 * Models if a multi-tenant user reaches the route.
 * (The API Models / API Providers catalogs are served by the external reference API and stay available
 * in multi-tenant, so they no longer use this guard.)
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
