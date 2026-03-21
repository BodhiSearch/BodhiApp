import { BODHI_API_BASE } from '@/hooks/constants';

export const ENDPOINT_APP_INFO = `${BODHI_API_BASE}/info`;
export const ENDPOINT_APP_SETUP = `${BODHI_API_BASE}/setup`;

export const appInfoKeys = {
  all: ['appInfo'] as const,
};
