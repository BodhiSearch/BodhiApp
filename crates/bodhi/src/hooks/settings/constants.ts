import { BODHI_API_BASE } from '@/hooks/constants';

export const ENDPOINT_SETTINGS = `${BODHI_API_BASE}/settings`;
export const ENDPOINT_SETTING_KEY = `${BODHI_API_BASE}/settings/{key}`;

export const settingKeys = {
  all: ['settings'] as const,
};
