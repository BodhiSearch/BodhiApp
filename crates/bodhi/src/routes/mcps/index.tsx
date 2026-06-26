import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

import AppInitializer from '@/components/AppInitializer';

import { MyMcpsScreen } from './-components/MyMcpsScreen';

// Single source of truth for the My MCPs page URL state. Only non-default values appear (order=asc and
// the empty scope are stripped before navigating) so Back/Forward round-trips exactly what changed.
export const myMcpsSearchSchema = z.object({
  q: z.string().optional(),
  // The open detail rail: the selected server's id (written with replace — no history entries).
  select: z.string().optional(),
  order: z.enum(['asc', 'desc']).optional(),
  // Scope facet: undefined/all = every registered server; mine = only servers with ≥1 instance.
  scope: z.enum(['all', 'mine']).optional(),
});

export type MyMcpsSearch = z.infer<typeof myMcpsSearchSchema>;

export const Route = createFileRoute('/mcps/')({
  validateSearch: myMcpsSearchSchema,
  component: McpsPage,
});

export default function McpsPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <MyMcpsScreen />
    </AppInitializer>
  );
}
