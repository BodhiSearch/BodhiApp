import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

import McpPlaygroundPage from '@/app/mcps/playground/page';

export const Route = createFileRoute('/mcps/playground/')({
  validateSearch: z.object({ id: z.string().optional() }),
  component: McpPlaygroundPage,
});
