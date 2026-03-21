import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

import EditApiModelPage from '@/app/models/api/edit/page';

export const Route = createFileRoute('/models/api/edit/')({
  validateSearch: z.object({ id: z.string().optional() }),
  component: EditApiModelPage,
});
