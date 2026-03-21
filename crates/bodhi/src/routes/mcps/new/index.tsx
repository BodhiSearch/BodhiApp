import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

import NewMcpPage from '@/app/mcps/new/page';

export const Route = createFileRoute('/mcps/new/')({
  validateSearch: z.object({
    id: z.string().optional(),
  }),
  component: NewMcpPage,
});
