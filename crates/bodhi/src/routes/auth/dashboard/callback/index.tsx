import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

import DashboardCallbackPage from '@/app/auth/dashboard/callback/page';

export const Route = createFileRoute('/auth/dashboard/callback/')({
  validateSearch: z.object({
    code: z.string().optional(),
    state: z.string().optional(),
  }),
  component: DashboardCallbackPage,
});
