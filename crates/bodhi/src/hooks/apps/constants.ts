import { BODHI_API_BASE } from '@/hooks/constants';

export const ENDPOINT_ACCESS_REQUESTS = `${BODHI_API_BASE}/access-requests`;
export const ENDPOINT_ACCESS_REQUESTS_REVIEW = `${BODHI_API_BASE}/access-requests/{id}/review`;
export const ENDPOINT_ACCESS_REQUESTS_APPROVE = `${BODHI_API_BASE}/access-requests/{id}/approve`;
export const ENDPOINT_ACCESS_REQUESTS_DENY = `${BODHI_API_BASE}/access-requests/{id}/deny`;
export const ENDPOINT_ACCESS_REQUESTS_APPS = `${BODHI_API_BASE}/access-requests/apps`;
export const ENDPOINT_ACCESS_REQUESTS_REVOKE = `${BODHI_API_BASE}/access-requests/{id}/revoke`;

export const appAccessRequestKeys = {
  all: ['app-access-request'] as const,
  detail: (id: string) => [...appAccessRequestKeys.all, id] as const,
  list: () => [...appAccessRequestKeys.all, 'list'] as const,
};
