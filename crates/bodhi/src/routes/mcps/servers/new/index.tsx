import { createFileRoute } from '@tanstack/react-router';

import NewMcpServerPage from '@/app/mcps/servers/new/page';

export const Route = createFileRoute('/mcps/servers/new/')({
  component: NewMcpServerPage,
});
