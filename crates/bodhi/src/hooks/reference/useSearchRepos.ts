import type { ListReposQuery, ListReposResponse, RepoSuggestion } from '@bodhiapp/reference-api-types';
import { keepPreviousData, useQuery } from '@tanstack/react-query';

import { REF_ENDPOINT_REPOS, referenceKeys } from './constants';
import { useAnonymousReferenceApi } from './useReferenceApi';

/**
 * `GET /api/v1/repos` — typeahead for full `<author>/<repo>` HuggingFace ids, scoped to GGUF repos.
 *
 * Powers the repo field of the add-local-model form: each returned `id` feeds straight into the
 * single-model endpoint (`useModelDetail`) to list quants. Anonymous client (public read; an invalid
 * id_token is rejected 401) and same fetch path as `useDiscoverModels`.
 *
 * Disabled on empty `search` — the API 422s without it, so the form shows local repos instead.
 */
function buildReposQuery(params: ListReposQuery): string {
  const sp = new URLSearchParams();
  sp.append('search', params.search);
  for (const f of [params.filter].flat()) {
    if (f) sp.append('filter', f);
  }
  if (params.limit != null) sp.append('limit', String(params.limit));
  return sp.toString();
}

export function useSearchRepos(params: ListReposQuery) {
  const client = useAnonymousReferenceApi();
  const search = params.search.trim();
  const query = buildReposQuery({ ...params, search });
  return useQuery<ListReposResponse, unknown, RepoSuggestion[]>({
    queryKey: referenceKeys.repos(query),
    queryFn: () => client!.get<ListReposResponse>(`${REF_ENDPOINT_REPOS}?${query}`),
    enabled: !!client && search.length > 0,
    placeholderData: keepPreviousData,
    staleTime: 30_000,
    select: (data) => data.items,
  });
}
