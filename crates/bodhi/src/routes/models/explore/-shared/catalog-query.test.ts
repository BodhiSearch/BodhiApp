import { describe, expect, it } from 'vitest';

import { buildCatalogModelsQuery } from '@/hooks/reference';
import { modelFacetsToQuery } from '@/routes/models/explore/api/-components/ExploreApiSidebar';
import { providerFacetsToQuery } from '@/routes/models/explore/providers/-components/ExploreProvidersSidebar';

/**
 * Catalog params flow from facet state → query string. Guards the serialization seam so the wiring
 * can't silently drop a param. Model pricing uses the four-param input/output ranges + a free/paid
 * filter; provider pricing keeps its own `pricing`/`pricing_max`.
 */
describe('catalog query mappers', () => {
  it('modelFacetsToQuery emits the four-param pricing + family', () => {
    const q = modelFacetsToQuery({
      pricing_in_min: 1,
      pricing_in_max: 5,
      pricing_out_min: 2,
      pricing_out_max: 20,
      family: ['claude', 'gpt'],
    });
    expect(q).toMatchObject({
      pricing_in_min: 1,
      pricing_in_max: 5,
      pricing_out_min: 2,
      pricing_out_max: 20,
      family: ['claude', 'gpt'],
    });

    const sp = new URLSearchParams(buildCatalogModelsQuery(q));
    expect(sp.get('pricing_in_min')).toBe('1');
    expect(sp.get('pricing_in_max')).toBe('5');
    expect(sp.get('pricing_out_min')).toBe('2');
    expect(sp.get('pricing_out_max')).toBe('20');
    expect(sp.getAll('family')).toEqual(['claude', 'gpt']);
  });

  it('modelFacetsToQuery omits unset params', () => {
    expect(modelFacetsToQuery({})).toEqual({});
  });

  it('pricing=free drops the price ranges (no redundant bounds)', () => {
    const q = modelFacetsToQuery({ pricing: 'free', pricing_in_max: 5, pricing_out_max: 20 });
    expect(q).toEqual({ pricing: 'free' });
  });

  it('providerFacetsToQuery emits provider price params', () => {
    const q = providerFacetsToQuery({ pricing_max: 10, pricing: 'free' });
    expect(q).toMatchObject({ pricing_max: 10, pricing: 'free' });

    const sp = new URLSearchParams(buildCatalogModelsQuery(q));
    expect(sp.get('pricing_max')).toBe('10');
    expect(sp.get('pricing')).toBe('free');
  });
});
