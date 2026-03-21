import { BODHI_API_BASE } from '@/hooks/constants';

export const ENDPOINT_USER_INFO = `${BODHI_API_BASE}/user`;
export const ENDPOINT_USERS = `${BODHI_API_BASE}/users`;
export const ENDPOINT_USER_ROLE = `${BODHI_API_BASE}/users/{user_id}/role`;
export const ENDPOINT_USER_ID = `${BODHI_API_BASE}/users/{user_id}`;

export const userKeys = {
  current: ['user'] as const,
  all: ['users'] as const,
  lists: () => [...userKeys.all, 'list'] as const,
  list: (page: number, pageSize: number) => [...userKeys.lists(), page, pageSize] as const,
};

// User access request endpoints
export const ENDPOINT_USER_REQUEST_STATUS = `${BODHI_API_BASE}/user/request-status`;
export const ENDPOINT_USER_REQUEST_ACCESS = `${BODHI_API_BASE}/user/request-access`;
export const ENDPOINT_ACCESS_REQUESTS_PENDING = `${BODHI_API_BASE}/access-requests/pending`;
export const ENDPOINT_ACCESS_REQUESTS = `${BODHI_API_BASE}/access-requests`;
export const ENDPOINT_ACCESS_REQUEST_APPROVE = `${BODHI_API_BASE}/access-requests/{id}/approve`;
export const ENDPOINT_ACCESS_REQUEST_REJECT = `${BODHI_API_BASE}/access-requests/{id}/reject`;

export const accessRequestKeys = {
  all: ['access-request'] as const,
  status: ['access-request', 'status'] as const,
  pending: (page: number, pageSize: number) => ['access-request', 'pending', page, pageSize] as const,
  list: (page: number, pageSize: number) => ['access-request', 'all', page, pageSize] as const,
};
