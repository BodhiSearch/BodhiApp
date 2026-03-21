import { createFileRoute } from '@tanstack/react-router';

import McpsPage from '@/app/mcps/page';

export const Route = createFileRoute('/mcps/')({
  component: McpsPage,
});
