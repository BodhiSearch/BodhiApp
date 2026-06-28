import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

import AppInitializer from '@/components/AppInitializer';

import { ExploreMcpScreen } from './-components/ExploreMcpScreen';

// Open-ended string array param (category/auth values are data-driven from the API facets, not a
// fixed enum, so we accept any string and let the backend reject unknowns).
function stringArrayParam() {
  return z.preprocess((v) => {
    if (v == null) return undefined;
    const arr = (Array.isArray(v) ? v : [v]).filter((x): x is string => typeof x === 'string' && x !== '');
    return arr.length ? arr : undefined;
  }, z.array(z.string()).optional());
}

// Single source of truth for the Explore · MCP Servers page. Only NON-DEFAULT values appear: the
// screen strips order=asc / page=1 before navigating, so the URL stays clean and Back/Forward
// round-trips exactly what the user changed.
export const exploreMcpSearchSchema = z.object({
  q: z.string().optional(),
  // The open detail rail: the selected server's `id`. Written with replace (no history entries);
  // Back/Forward restores the rail from whatever the target URL carries.
  select: z.string().optional(),
  // The reference API only supports sort=name today.
  sort: z.enum(['name']).optional(),
  order: z.enum(['asc', 'desc']).optional(),
  page: z.number().int().positive().optional(),
  // Facets: category + auth are server-side (repeatable OR); verified + installed are client-side
  // (the catalog API has no such params — installed is derived by joining the user's instances).
  category: stringArrayParam(),
  auth: stringArrayParam(),
  verified: z.boolean().optional(),
  installed: z.enum(['installed', 'not_installed']).optional(),
});

export type ExploreMcpSearch = z.infer<typeof exploreMcpSearchSchema>;

export const Route = createFileRoute('/mcps/explore/')({
  staticData: { section: 'mcp', subPage: 'explore-mcp' },
  validateSearch: exploreMcpSearchSchema,
  component: ExploreMcpPage,
});

export default function ExploreMcpPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <ExploreMcpScreen />
    </AppInitializer>
  );
}
