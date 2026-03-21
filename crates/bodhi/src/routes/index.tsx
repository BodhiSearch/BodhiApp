import { createFileRoute } from '@tanstack/react-router';

import RootPage from '@/app/page';

export const Route = createFileRoute('/')({
  component: RootPage,
});
