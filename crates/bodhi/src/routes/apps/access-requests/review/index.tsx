import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

import ReviewAccessRequestPage from '@/app/apps/access-requests/review/page';

export const Route = createFileRoute('/apps/access-requests/review/')({
  validateSearch: z.object({ id: z.string().optional() }),
  component: ReviewAccessRequestPage,
});
