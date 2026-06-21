/**
 * Query-key factory for external reference-API data (https://api.getbodhi.app/).
 * Domain hooks added per-batch (e.g. the MCP Discover catalog) extend this.
 */
export const referenceKeys = {
  all: ['reference'] as const,
  discover: () => [...referenceKeys.all, 'discover'] as const,
  discoverList: (paramsKey: string) => [...referenceKeys.discover(), 'list', paramsKey] as const,
  discoverDetail: (source: string, namespace: string, repo: string) =>
    [...referenceKeys.discover(), 'detail', source, namespace, repo] as const,
};

/** Reference-API paths (relative to `AppInfo.reference_api_url`). */
export const REF_ENDPOINT_MODELS = '/api/v1/models';
export const refEndpointModel = (source: string, namespace: string, repo: string) =>
  `/api/v1/models/${source}/${encodeURIComponent(namespace)}/${encodeURIComponent(repo)}`;
