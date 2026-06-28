import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

import AppInitializer from '@/components/AppInitializer';
import { MultiTenantGuard } from '@/routes/models/explore/-shared/MultiTenantGuard';
import { arrayParam } from '@/routes/models/explore/-shared/search-params';

import { ExploreProvidersScreen } from './-components/ExploreProvidersScreen';

const CAPABILITY = ['reasoning', 'tool_call', 'structured_output', 'attachment', 'vision'] as const;
const API_FORMAT = ['openai', 'openai_responses', 'anthropic', 'anthropic_oauth', 'gemini', 'other'] as const;
const SORT = ['name', 'model_count', 'api_format'] as const;

// Single source of truth for Explore · API Providers. Only NON-DEFAULT values appear: the screen
// strips order/page=1 before navigating, and never writes a localStorage-sourced sort.
//   - `select` opens a provider's rail on mount (cross-link from API Models "Served by").
//   - `q` is the committed search (also the landing param for the Models "View" cross-link).
export const exploreProvidersSearchSchema = z.object({
  select: z.string().optional(),
  q: z.string().optional(),
  sort: z.enum(SORT).optional(),
  order: z.enum(['asc', 'desc']).optional(),
  page: z.number().int().positive().optional(),
  capability: arrayParam(CAPABILITY),
  api_format: arrayParam(API_FORMAT),
  pricing: z.enum(['free', 'paid']).optional(),
  is_lab: z.enum(['true']).optional(),
});

export type ExploreProvidersSearch = z.infer<typeof exploreProvidersSearchSchema>;

export const Route = createFileRoute('/models/explore/providers/')({
  staticData: { section: 'models', subPage: 'explore-api-providers' },
  validateSearch: exploreProvidersSearchSchema,
  component: ExploreProvidersPage,
});

export default function ExploreProvidersPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <MultiTenantGuard>
        <ExploreProvidersScreen />
      </MultiTenantGuard>
    </AppInitializer>
  );
}
