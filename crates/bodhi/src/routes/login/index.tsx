import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

import LoginPage from '@/app/login/page';

export const Route = createFileRoute('/login/')({
  validateSearch: z.object({
    error: z.string().optional(),
    invite: z.string().optional(),
  }),
  component: LoginPage,
});
