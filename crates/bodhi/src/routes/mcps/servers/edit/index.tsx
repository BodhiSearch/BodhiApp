import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

import EditMcpServerPage from '@/app/mcps/servers/edit/page';

export const Route = createFileRoute('/mcps/servers/edit/')({
  validateSearch: z.object({ id: z.string().optional() }),
  component: EditMcpServerPage,
});
