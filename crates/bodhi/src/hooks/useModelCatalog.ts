import { chatModelsCatalog, embeddingModelsCatalog } from '@/app/ui/setup/download-models/data';
import { ModelCatalog } from '@/app/ui/setup/download-models/types';

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
