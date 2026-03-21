import { BODHI_API_BASE } from '@/hooks/constants';

export const API_TOKENS_ENDPOINT = `${BODHI_API_BASE}/tokens`;
export const ENDPOINT_TOKEN_ID = `${BODHI_API_BASE}/tokens/{id}`;

export const tokenKeys = {
  all: ['tokens'] as const,
  lists: () => [...tokenKeys.all, 'list'] as const,
  list: (page: number, pageSize: number) => [...tokenKeys.lists(), page, pageSize] as const,
};
