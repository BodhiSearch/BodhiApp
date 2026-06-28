import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

import AppInitializer from '@/components/AppInitializer';

import { PlaygroundScreen } from './-components/PlaygroundScreen';

export const playgroundSearchSchema = z.object({
  id: z.string().optional(),
  feature: z.enum(['overview', 'tools', 'prompts', 'resources', 'templates']).optional(),
  item: z.string().optional(),
});

export type PlaygroundSearch = z.infer<typeof playgroundSearchSchema>;

export const Route = createFileRoute('/mcps/playground/')({
  staticData: { section: 'mcp' },
  validateSearch: playgroundSearchSchema,
  component: McpPlaygroundPage,
});

export default function McpPlaygroundPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <PlaygroundScreen />
    </AppInitializer>
  );
}
