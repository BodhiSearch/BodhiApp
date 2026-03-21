import { createFileRoute } from '@tanstack/react-router';

import PullFilePage from '@/app/models/files/pull/page';

export const Route = createFileRoute('/models/files/pull/')({
  component: PullFilePage,
});
