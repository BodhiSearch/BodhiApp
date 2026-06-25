import { describe, expect, it } from 'vitest';

import type { ExploreProvidersSearch } from '../index';

import { facetsToSearch, PAGE_SIZE, searchToFacets, searchToParams } from './explore-providers-search';
import type { ProviderFacets } from './ExploreProvidersSidebar';

describe('explore-providers-search mappers', () => {
  describe('searchToFacets', () => {
    it('maps every facet field from the URL search and ignores non-facet fields', () => {
      const search: ExploreProvidersSearch = {
        capability: ['reasoning', 'vision'],
        api_format: ['anthropic'],
        pricing: 'free',
        is_lab: 'true',
        q: 'nano',
        sort: 'name',
        order: 'asc',
        page: 2,
      };
      expect(searchToFacets(search)).toEqual({
        capability: ['reasoning', 'vision'],
        api_format: ['anthropic'],
        pricing: 'free',
        is_lab: true,
      });
    });

    it('returns {} for an empty search and omits empty arrays', () => {
      expect(searchToFacets({})).toEqual({});
      expect(searchToFacets({ capability: [] } as ExploreProvidersSearch)).toEqual({});
    });
  });

  describe('facetsToSearch round-trips with searchToFacets', () => {
    it('preserves a representative facet set', () => {
      const facets: ProviderFacets = {
        capability: ['reasoning'],
        api_format: ['openai', 'gemini'],
        pricing: 'paid',
        is_lab: true,
      };
      expect(searchToFacets(facetsToSearch(facets) as ExploreProvidersSearch)).toEqual(facets);
    });

    it('maps is_lab to the literal "true" query param', () => {
      expect(facetsToSearch({ is_lab: true })).toEqual({ is_lab: 'true' });
      expect(facetsToSearch({})).toEqual({});
    });
  });

  describe('searchToParams', () => {
    it('omits sort/order when none is set (API natural order)', () => {
      expect(searchToParams({})).toEqual({ page: 1, page_size: PAGE_SIZE });
    });

    it('passes through URL sort/order/page/q and the facet slice', () => {
      const params = searchToParams({
        q: 'nano',
        sort: 'model_count',
        order: 'desc',
        page: 3,
        capability: ['tool_call'],
        is_lab: 'true',
      });
      expect(params).toMatchObject({
        q: 'nano',
        sort: 'model_count',
        order: 'desc',
        page: 3,
        page_size: PAGE_SIZE,
        capability: ['tool_call'],
        is_lab: 'true',
      });
    });

    it('lets the effective sort override the URL slice (localStorage applied to the request only)', () => {
      expect(searchToParams({}, { sort: 'name', order: 'asc' })).toMatchObject({
        sort: 'name',
        order: 'asc',
        page: 1,
        page_size: PAGE_SIZE,
      });
    });
  });
});
