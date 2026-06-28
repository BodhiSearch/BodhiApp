import { createFileRoute, redirect } from '@tanstack/react-router';
import { z } from 'zod';

/**
 * The V2 "Configure server" hub (`/mcps/servers/view/`) edits basic details inline (per-section),
 * so there's no separate edit page anymore. Keep the route as a redirect for any old deep-links.
 */
export const Route = createFileRoute('/mcps/servers/edit/')({
  staticData: { section: 'mcp' },
  validateSearch: z.object({ id: z.string().optional() }),
  beforeLoad: ({ search }) => {
    throw redirect({ to: '/mcps/servers/view/', search: { id: (search as { id?: string }).id } });
  },
});
