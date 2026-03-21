import { createFileRoute } from '@tanstack/react-router';

import SetupToolsetsPage from '@/app/setup/toolsets/page';

export const Route = createFileRoute('/setup/toolsets/')({
  component: SetupToolsetsPage,
});
