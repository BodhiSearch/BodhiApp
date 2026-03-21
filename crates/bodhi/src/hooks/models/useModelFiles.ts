// External imports
import { PaginatedLocalModelResponse } from '@bodhiapp/ts-client';

// Internal imports
import { useQuery } from '@/hooks/useQuery';

import { modelFileKeys, ENDPOINT_MODEL_FILES } from './constants';

export function useListModelFiles(page?: number, pageSize?: number, sort: string = 'repo', sortOrder: string = 'asc') {
  return useQuery<PaginatedLocalModelResponse>(
    modelFileKeys.list(page ?? -1, pageSize ?? -1, sort, sortOrder),
    ENDPOINT_MODEL_FILES,
    { page, page_size: pageSize, sort, sort_order: sortOrder }
  );
}
