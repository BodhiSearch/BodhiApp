import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

import AppInitializer from '@/components/AppInitializer';
import { MultiTenantGuard } from '@/routes/models/explore/-shared/MultiTenantGuard';
import { arrayParam } from '@/routes/models/explore/-shared/search-params';

import { LocalDiscoveryScreen } from './-components/LocalDiscoveryScreen';

const SORT = ['downloads', 'likes', 'last_modified', 'created_at', 'trending'] as const;
const SPECIALISATION = ['coding', 'reasoning', 'vision'] as const;

function stringArrayParam() {
  return z.preprocess((v) => {
    if (v == null) return undefined;
    const arr = (Array.isArray(v) ? v : [v]).filter((x): x is string => typeof x === 'string' && x !== '');
    return arr.length ? arr : undefined;
  }, z.array(z.string()).optional());
}

// Single source of truth for Explore · Local Models. The catalog is HuggingFace-backed and
// descending-only, so there is NO `order` param. Keyset cursor + Load-More accumulation stay in
// component state (cursors are opaque/volatile) — only sort/facets/search/select live in the URL.
export const localDiscoverySearchSchema = z.object({
  q: z.string().optional(),
  select: z.string().optional(),
  sort: z.enum(SORT).optional(),
  specialisation: arrayParam(SPECIALISATION),
  pipeline_tag: z.string().optional(),
  tag: stringArrayParam(),
  language: stringArrayParam(),
  license: stringArrayParam(),
  author: stringArrayParam(),
});

export type LocalDiscoverySearch = z.infer<typeof localDiscoverySearchSchema>;

export const Route = createFileRoute('/models/explore/local/')({
  staticData: { section: 'models', subPage: 'explore-local' },
  validateSearch: localDiscoverySearchSchema,
  component: LocalDiscoverPage,
});

export default function LocalDiscoverPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <MultiTenantGuard>
        <LocalDiscoveryScreen />
      </MultiTenantGuard>
    </AppInitializer>
  );
}
