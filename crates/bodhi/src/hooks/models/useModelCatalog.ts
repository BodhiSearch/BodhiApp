import { chatModelsCatalog, embeddingModelsCatalog } from './model-catalog-data';
import { ModelCatalog } from './model-catalog-types';

/**
 * Hook to get chat models catalog
 * Currently returns static data from data.ts
 * Future: Can be updated to fetch from API endpoint
 */
export function useChatModelsCatalog(): { data: ModelCatalog[]; isLoading: boolean } {
  return {
    data: chatModelsCatalog,
    isLoading: false,
  };
}

/**
 * Hook to get embedding models catalog
 * Currently returns static data from data.ts
 * Future: Can be updated to fetch from API endpoint
 */
export function useEmbeddingModelsCatalog(): { data: ModelCatalog[]; isLoading: boolean } {
  return {
    data: embeddingModelsCatalog,
    isLoading: false,
  };
}
