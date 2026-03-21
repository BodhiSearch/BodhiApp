import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

import ViewMcpServerPage from '@/app/mcps/servers/view/page';

export const Route = createFileRoute('/mcps/servers/view/')({
  validateSearch: z.object({ id: z.string().optional() }),
  component: ViewMcpServerPage,
});
