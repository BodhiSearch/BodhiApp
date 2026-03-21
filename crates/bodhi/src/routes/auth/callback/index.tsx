import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

import AuthCallbackPage from '@/app/auth/callback/page';

export const Route = createFileRoute('/auth/callback/')({
  validateSearch: z.object({}).passthrough(),
  component: AuthCallbackPage,
});
