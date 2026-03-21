import { createFileRoute } from '@tanstack/react-router';

import McpServersPage from '@/app/mcps/servers/page';

export const Route = createFileRoute('/mcps/servers/')({
  component: McpServersPage,
});
