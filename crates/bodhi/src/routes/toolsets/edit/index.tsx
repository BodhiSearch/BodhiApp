import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

import EditToolsetPage from '@/app/toolsets/edit/page';

export const Route = createFileRoute('/toolsets/edit/')({
  validateSearch: z.object({ id: z.string().optional() }),
  component: EditToolsetPage,
});
