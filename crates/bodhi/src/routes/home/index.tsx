import { createFileRoute } from '@tanstack/react-router';

import HomePage from '@/app/home/page';

export const Route = createFileRoute('/home/')({
  component: HomePage,
});
