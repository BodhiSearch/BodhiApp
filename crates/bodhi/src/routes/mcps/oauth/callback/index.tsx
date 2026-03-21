import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

import McpOAuthCallbackPage from '@/app/mcps/oauth/callback/page';

export const Route = createFileRoute('/mcps/oauth/callback/')({
  validateSearch: z.object({
    code: z.string().optional(),
    state: z.string().optional(),
    error: z.string().optional(),
    error_description: z.string().optional(),
  }),
  component: McpOAuthCallbackPage,
});
