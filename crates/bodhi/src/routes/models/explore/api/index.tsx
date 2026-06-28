import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

import AppInitializer from '@/components/AppInitializer';
import { MultiTenantGuard } from '@/routes/models/explore/-shared/MultiTenantGuard';
import { arrayParam } from '@/routes/models/explore/-shared/search-params';

import { ExploreApiScreen } from './-components/ExploreApiScreen';

// Closed enum sets (mirror @bodhiapp/reference-api-types). Using z.enum drops a hand-edited junk
// param rather than forwarding it to the catalog API. `status` adds the synthetic 'stable' bucket
// (status-absent) on top of the wire ModelStatus. provider/family are open-ended slugs → z.string.
const CAPABILITY = ['reasoning', 'tool_call', 'structured_output', 'attachment', 'vision'] as const;
const MODALITY = ['text', 'audio', 'image', 'video', 'pdf'] as const;
const STATUS = ['stable', 'alpha', 'beta', 'deprecated'] as const;
const SORT = ['relevance', 'updated', 'context', 'providers', 'price', 'price_out', 'name', 'family'] as const;

function stringArrayParam() {
  return z.preprocess((v) => {
    if (v == null) return undefined;
    const arr = (Array.isArray(v) ? v : [v]).filter((x): x is string => typeof x === 'string' && x !== '');
    return arr.length ? arr : undefined;
  }, z.array(z.string()).optional());
}

// Single source of truth for the Explore · API Models page. Only NON-DEFAULT values ever appear here:
// the screen strips sort=updated / order=desc / page=1 before navigating, so the URL stays clean and
// browser Back/Forward round-trips exactly what the user changed.
export const exploreApiSearchSchema = z.object({
  q: z.string().optional(),
  // The open detail rail: composite `${slug}/${model_id}` of the selected row. Written with replace
  // (no history entries); Back/Forward restores the rail from whatever the target URL carries.
  select: z.string().optional(),
  sort: z.enum(SORT).optional(),
  order: z.enum(['asc', 'desc']).optional(),
  page: z.number().int().positive().optional(),
  capability: arrayParam(CAPABILITY),
  modality: arrayParam(MODALITY),
  status: arrayParam(STATUS),
  provider: stringArrayParam(), // provider SLUGs (facet bucket is keyed by slug)
  family: stringArrayParam(),
  open_weights: z.enum(['open', 'closed']).optional(),
  pricing: z.enum(['free', 'paid']).optional(),
  pricing_in_min: z.number().optional(),
  pricing_in_max: z.number().optional(),
  pricing_out_min: z.number().optional(),
  pricing_out_max: z.number().optional(),
  context_min: z.number().optional(),
});

export type ExploreApiSearch = z.infer<typeof exploreApiSearchSchema>;

export const Route = createFileRoute('/models/explore/api/')({
  staticData: { section: 'models', subPage: 'explore-api' },
  validateSearch: exploreApiSearchSchema,
  component: ExploreApiPage,
});

export default function ExploreApiPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <MultiTenantGuard>
        <ExploreApiScreen />
      </MultiTenantGuard>
    </AppInitializer>
  );
}
