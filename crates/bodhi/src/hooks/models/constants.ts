import { BODHI_API_BASE } from '@/hooks/constants';

export const modelKeys = {
  all: ['models'] as const,
  lists: () => [...modelKeys.all, 'list'] as const,
  list: (page: number, pageSize: number, sort: string, sortOrder: string) =>
    [...modelKeys.lists(), page, pageSize, sort, sortOrder] as const,
  details: () => [...modelKeys.all, 'detail'] as const,
  detail: (id: string) => [...modelKeys.details(), id] as const,
};

export const modelFileKeys = {
  all: ['modelFiles'] as const,
  lists: () => [...modelFileKeys.all, 'list'] as const,
  list: (page: number, pageSize: number, sort: string, sortOrder: string) =>
    [...modelFileKeys.lists(), page, pageSize, sort, sortOrder] as const,
};

export const downloadKeys = {
  all: ['downloads'] as const,
  lists: () => [...downloadKeys.all, 'list'] as const,
  list: (page: number, pageSize: number) => [...downloadKeys.lists(), page, pageSize] as const,
};

export const apiModelKeys = {
  all: ['api-models'] as const,
  details: () => [...apiModelKeys.all, 'detail'] as const,
  detail: (id: string) => [...apiModelKeys.details(), id] as const,
};

export const apiFormatKeys = {
  all: ['api-formats'] as const,
};

// Endpoint constants (consolidated from hook files)
export const ENDPOINT_MODELS = `${BODHI_API_BASE}/models`;
export const ENDPOINT_MODEL_ID = `${BODHI_API_BASE}/models/{id}`;
export const ENDPOINT_ALIAS = `${BODHI_API_BASE}/models/alias`;
export const ENDPOINT_ALIAS_ID = `${BODHI_API_BASE}/models/alias/{id}`;
export const ENDPOINT_MODEL_FILES = `${BODHI_API_BASE}/models/files`;
export const ENDPOINT_MODEL_FILES_PULL = `${BODHI_API_BASE}/models/files/pull`;
export const ENDPOINT_MODELS_REFRESH = `${BODHI_API_BASE}/models/refresh`;
export const ENDPOINT_QUEUE = `${BODHI_API_BASE}/queue`;
export const ENDPOINT_API_MODELS = `${BODHI_API_BASE}/models/api`;
export const ENDPOINT_API_MODEL_ID = `${BODHI_API_BASE}/models/api/{id}`;
export const ENDPOINT_API_MODELS_TEST = `${BODHI_API_BASE}/models/api/test`;
export const ENDPOINT_API_MODELS_FETCH = `${BODHI_API_BASE}/models/api/fetch-models`;
export const ENDPOINT_API_MODELS_FORMATS = `${BODHI_API_BASE}/models/api/formats`;
