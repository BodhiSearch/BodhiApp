import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

import ChatPage from '@/app/chat/page';

export const Route = createFileRoute('/chat/')({
  validateSearch: z.object({
    model: z.string().optional(),
    id: z.string().optional(),
  }),
  component: ChatPage,
});
