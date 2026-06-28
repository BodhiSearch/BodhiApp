import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

import AppInitializer from '@/components/AppInitializer';
import { arrayParam } from '@/routes/models/explore/-shared/search-params';

import { ModelsScreenV2 } from './-components/ModelsScreenV2';

// Facet token sets mirror the backend query params (see hooks/models/useModels.ts `ModelsFilter`).
const TYPE = ['local_file', 'model_alias', 'api_model', 'fallback'] as const;
const API_FORMAT = ['openai', 'responses', 'anthropic', 'gemini', 'liberty'] as const;
const CAPABILITY = ['vision', 'tool_use', 'reasoning'] as const;
// Name is server-sortable (sort='alias'); provider/base_url are derived columns sorted client-side.
const SORT = ['name', 'provider', 'base_url'] as const;

// Single source of truth for My Models. Only NON-DEFAULT values appear: the screen strips order /
// page=1 before navigating, so the URL stays clean and Back/Forward round-trips what changed.
export const modelsSearchSchema = z.object({
  q: z.string().optional(),
  // The open detail rail: the selected alias id. Written with replace (no history entries);
  // Back/Forward restores the rail from whatever the target URL carries.
  select: z.string().optional(),
  sort: z.enum(SORT).optional(),
  order: z.enum(['asc', 'desc']).optional(),
  page: z.number().int().positive().optional(),
  type: arrayParam(TYPE),
  api_format: arrayParam(API_FORMAT),
  capability: arrayParam(CAPABILITY),
  size_min: z.number().optional(),
  size_max: z.number().optional(),
});

export type ModelsSearch = z.infer<typeof modelsSearchSchema>;

export const Route = createFileRoute('/models/')({
  staticData: { section: 'models', subPage: 'my-models' },
  validateSearch: modelsSearchSchema,
  component: ModelsPage,
});

export default function ModelsPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <ModelsScreenV2 />
    </AppInitializer>
  );
}
