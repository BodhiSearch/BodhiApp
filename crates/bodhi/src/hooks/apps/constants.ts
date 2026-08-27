import { BODHI_API_BASE } from '@/hooks/constants';

export const ENDPOINT_ACCESS_REQUESTS = `${BODHI_API_BASE}/access-requests`;
export const ENDPOINT_ACCESS_REQUESTS_APPS = `${BODHI_API_BASE}/access-requests/apps`;
export const ENDPOINT_ACCESS_REQUESTS_REVOKE = `${BODHI_API_BASE}/access-requests/{id}/revoke`;
export const ENDPOINT_APPS_ACCESS_REQUESTS = `${BODHI_API_BASE}/apps/access-requests`;
export const ENDPOINT_APPS_ACCESS_REQUESTS_CONSENT = `${BODHI_API_BASE}/apps/access-requests/consent`;

export const appAccessRequestKeys = {
  all: ['app-access-request'] as const,
  list: () => [...appAccessRequestKeys.all, 'list'] as const,
  consent: (search: string) => [...appAccessRequestKeys.all, 'consent', search] as const,
};
