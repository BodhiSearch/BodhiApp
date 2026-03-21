import { createFileRoute } from '@tanstack/react-router';

import NewApiModelPage from '@/app/models/api/new/page';

export const Route = createFileRoute('/models/api/new/')({
  component: NewApiModelPage,
});
