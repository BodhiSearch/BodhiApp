import { describe, expect, it } from 'vitest';

import { buildCatalogModelsQuery } from '@/hooks/reference';
import { modelFacetsToQuery } from '@/routes/models/explore/api/-components/ExploreApiSidebar';
import { providerFacetsToQuery } from '@/routes/models/explore/api-providers/-components/ExploreProvidersSidebar';

/**
 * Phase-1 plumbing: the enriched catalog params (`pricing_min`, `pricing_out_max`, `family`, provider
 * `pricing`/`pricing_max`) flow from facet state → query string. The UI controls that *set* these land
 * in later phases; this guards the serialization seam now so the wiring can't silently drop a param.
 */
describe('catalog query mappers (Phase 1)', () => {
  it('modelFacetsToQuery emits the new model params', () => {
    const q = modelFacetsToQuery({
      pricing_min: 1,
      pricing_max: 5,
      pricing_out_max: 20,
      family: ['claude', 'gpt'],
    });
    expect(q).toMatchObject({ pricing_min: 1, pricing_max: 5, pricing_out_max: 20, family: ['claude', 'gpt'] });

    const sp = new URLSearchParams(buildCatalogModelsQuery(q));
    expect(sp.get('pricing_min')).toBe('1');
    expect(sp.get('pricing_max')).toBe('5');
    expect(sp.get('pricing_out_max')).toBe('20');
    expect(sp.getAll('family')).toEqual(['claude', 'gpt']);
  });

  it('modelFacetsToQuery omits unset params', () => {
    expect(modelFacetsToQuery({})).toEqual({});
    // pricing_max=0 (the "Free" case) must survive — it is a meaningful value, not "unset".
    expect(modelFacetsToQuery({ pricing_max: 0 })).toEqual({ pricing_max: 0 });
  });

  it('providerFacetsToQuery emits provider price params', () => {
    const q = providerFacetsToQuery({ pricing_max: 10, pricing: 'free' });
    expect(q).toMatchObject({ pricing_max: 10, pricing: 'free' });

    const sp = new URLSearchParams(buildCatalogModelsQuery(q));
    expect(sp.get('pricing_max')).toBe('10');
    expect(sp.get('pricing')).toBe('free');
  });
});
