import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

import AppInitializer from '@/components/AppInitializer';

import { ExploreMcpScreen } from './-components/ExploreMcpScreen';

// Single source of truth for the Explore · MCP Servers page. Only NON-DEFAULT values appear: the
// screen strips order=asc / page=1 before navigating, so the URL stays clean and Back/Forward
// round-trips exactly what the user changed. Facets/select added in later phases.
export const exploreMcpSearchSchema = z.object({
  q: z.string().optional(),
  // The open detail rail: the selected server's `id`. Written with replace (no history entries);
  // Back/Forward restores the rail from whatever the target URL carries.
  select: z.string().optional(),
  // The reference API only supports sort=name today.
  sort: z.enum(['name']).optional(),
  order: z.enum(['asc', 'desc']).optional(),
  page: z.number().int().positive().optional(),
});

export type ExploreMcpSearch = z.infer<typeof exploreMcpSearchSchema>;

export const Route = createFileRoute('/mcps/explore/')({
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
