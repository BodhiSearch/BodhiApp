import { BODHI_API_BASE } from '@/hooks/constants';

export const ENDPOINT_TUNNEL = `${BODHI_API_BASE}/tunnel`;
export const ENDPOINT_TUNNEL_SETUP = `${ENDPOINT_TUNNEL}/setup`;
export const ENDPOINT_TUNNEL_SYNC = `${ENDPOINT_TUNNEL}/sync`;

export const tunnelKeys = {
  all: ['tunnel'] as const,
};
