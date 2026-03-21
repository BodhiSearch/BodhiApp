import { BODHI_API_BASE } from '@/hooks/constants';

export const ENDPOINT_TENANTS = `${BODHI_API_BASE}/tenants`;

export const tenantKeys = {
  all: ['tenants'] as const,
};
