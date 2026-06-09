import { chatModelsCatalog, embeddingModelsCatalog } from './model-catalog-data';
import { ModelCatalog } from './model-catalog-types';

export function useChatModelsCatalog(): { data: ModelCatalog[]; isLoading: boolean } {
  return {
    data: chatModelsCatalog,
    isLoading: false,
  };
}

export function useEmbeddingModelsCatalog(): { data: ModelCatalog[]; isLoading: boolean } {
  return {
    data: embeddingModelsCatalog,
    isLoading: false,
  };
}
