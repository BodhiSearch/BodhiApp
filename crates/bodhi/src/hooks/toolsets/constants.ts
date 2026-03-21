import { BODHI_API_BASE } from '@/hooks/constants';

export const TOOLSETS_ENDPOINT = `${BODHI_API_BASE}/toolsets`;
export const TOOLSET_TYPES_ENDPOINT = `${BODHI_API_BASE}/toolset_types`;

export const toolsetKeys = {
  all: ['toolsets'] as const,
  details: () => [...toolsetKeys.all, 'detail'] as const,
  detail: (id: string) => [...toolsetKeys.details(), id] as const,
};

export const toolsetTypeKeys = {
  all: ['toolset-types'] as const,
};
