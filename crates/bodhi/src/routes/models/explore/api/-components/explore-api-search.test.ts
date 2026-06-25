import { describe, expect, it } from 'vitest';

import type { ExploreApiSearch } from '../index';

import {
  DEFAULT_ORDER,
  DEFAULT_SORT,
  facetsToSearch,
  PAGE_SIZE,
  searchToFacets,
  searchToParams,
} from './explore-api-search';
import type { ModelFacetsState } from './ExploreApiSidebar';

describe('explore-api-search mappers', () => {
  describe('searchToFacets', () => {
    it('maps every facet field from the URL search', () => {
      const search: ExploreApiSearch = {
        capability: ['reasoning', 'vision'],
        modality: ['image'],
        status: ['stable'],
        provider: ['anthropic'],
        family: ['claude'],
        open_weights: 'open',
        pricing: 'paid',
        pricing_in_min: 1,
        pricing_in_max: 5,
        pricing_out_min: 2,
        pricing_out_max: 20,
        context_min: 8000,
        // non-facet fields are ignored by searchToFacets
        q: 'claude',
        sort: 'price',
        order: 'asc',
        page: 3,
      };
      expect(searchToFacets(search)).toEqual({
        capability: ['reasoning', 'vision'],
        modality: ['image'],
        status: ['stable'],
        provider: ['anthropic'],
        family: ['claude'],
        open_weights: 'open',
        pricing: 'paid',
        pricing_in_min: 1,
        pricing_in_max: 5,
        pricing_out_min: 2,
        pricing_out_max: 20,
        context_min: 8000,
      });
    });

    it('returns {} for an empty search and omits empty arrays', () => {
      expect(searchToFacets({})).toEqual({});
      expect(searchToFacets({ capability: [], provider: [] } as ExploreApiSearch)).toEqual({});
    });

    it('keeps a 0 price bound (not treated as absent)', () => {
      expect(searchToFacets({ pricing_in_min: 0 })).toEqual({ pricing_in_min: 0 });
    });
  });

  describe('facetsToSearch round-trips with searchToFacets', () => {
    it('preserves a representative facet set', () => {
      const facets: ModelFacetsState = {
        capability: ['reasoning'],
        provider: ['anthropic', 'openrouter'],
        open_weights: 'closed',
        pricing_in_min: 1.5,
        pricing_out_max: 30,
        context_min: 16000,
      };
      expect(searchToFacets(facetsToSearch(facets) as ExploreApiSearch)).toEqual(facets);
    });

    it('drops price ranges when pricing=free (free ANDs with $0 server-side)', () => {
      const facets: ModelFacetsState = {
        pricing: 'free',
        pricing_in_min: 1,
        pricing_out_max: 5,
      };
      // facetsToSearch reuses modelFacetsToQuery, which omits the ranges under free.
      expect(facetsToSearch(facets)).toEqual({ pricing: 'free' });
    });
  });

  describe('searchToParams', () => {
    it('applies defaults (sort/order/page/page_size) and omits an empty q', () => {
      expect(searchToParams({})).toEqual({
        sort: DEFAULT_SORT,
        order: DEFAULT_ORDER,
        page: 1,
        page_size: PAGE_SIZE,
      });
    });

    it('passes through non-default sort/order/page/q and the facet slice', () => {
      const params = searchToParams({
        q: 'gpt',
        sort: 'price',
        order: 'asc',
        page: 2,
        capability: ['tool_call'],
        provider: ['openai'],
      });
      expect(params).toMatchObject({
        q: 'gpt',
        sort: 'price',
        order: 'asc',
        page: 2,
        page_size: PAGE_SIZE,
        capability: ['tool_call'],
        provider: ['openai'],
      });
    });
  });
});
