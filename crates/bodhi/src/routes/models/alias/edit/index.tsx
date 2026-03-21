import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

import EditAliasPage from '@/app/models/alias/edit/page';

export const Route = createFileRoute('/models/alias/edit/')({
  validateSearch: z.object({ id: z.string().optional() }),
  component: EditAliasPage,
});
