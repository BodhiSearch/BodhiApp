import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

import NewAliasPage from '@/app/models/alias/new/page';

export const Route = createFileRoute('/models/alias/new/')({
  validateSearch: z.object({
    repo: z.string().optional(),
    filename: z.string().optional(),
    snapshot: z.string().optional(),
  }),
  component: NewAliasPage,
});
