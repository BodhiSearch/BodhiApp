import { createFileRoute } from '@tanstack/react-router';

import NewToolsetPage from '@/app/toolsets/new/page';

export const Route = createFileRoute('/toolsets/new/')({
  component: NewToolsetPage,
});
