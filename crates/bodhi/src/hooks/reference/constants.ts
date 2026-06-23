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
  // API-model catalog (`/api/v1/catalog/*`) — the "models.dev inside Bodhi" pages.
  catalog: () => [...referenceKeys.all, 'catalog'] as const,
  catalogProviders: (paramsKey: string) => [...referenceKeys.catalog(), 'providers', paramsKey] as const,
  catalogProvider: (slug: string) => [...referenceKeys.catalog(), 'provider', slug] as const,
  catalogProviderModels: (slug: string, paramsKey: string) =>
    [...referenceKeys.catalog(), 'provider', slug, 'models', paramsKey] as const,
  catalogModels: (paramsKey: string) => [...referenceKeys.catalog(), 'models', paramsKey] as const,
  catalogModel: (slug: string, modelId: string) => [...referenceKeys.catalog(), 'model', slug, modelId] as const,
};

/** Reference-API paths (relative to `AppInfo.reference_api_url`). */
export const REF_ENDPOINT_MODELS = '/api/v1/models';
export const refEndpointModel = (source: string, namespace: string, repo: string) =>
  `/api/v1/models/${source}/${encodeURIComponent(namespace)}/${encodeURIComponent(repo)}`;

/** Catalog (API-model) endpoints. */
export const REF_ENDPOINT_CATALOG_PROVIDERS = '/api/v1/catalog/providers';
export const REF_ENDPOINT_CATALOG_MODELS = '/api/v1/catalog/models';
export const refEndpointCatalogProvider = (slug: string) => `/api/v1/catalog/providers/${encodeURIComponent(slug)}`;
export const refEndpointCatalogProviderModels = (slug: string) =>
  `/api/v1/catalog/providers/${encodeURIComponent(slug)}/models`;
export const refEndpointCatalogModel = (slug: string, modelId: string) =>
  `/api/v1/catalog/models/${encodeURIComponent(slug)}/${encodeURIComponent(modelId)}`;
/** Provider logo SVG (currently 404s upstream — callers fall back to a monogram). */
export const refEndpointCatalogLogo = (slug: string) => `/api/v1/catalog/logos/${encodeURIComponent(slug)}.svg`;
