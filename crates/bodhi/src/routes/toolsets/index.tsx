import { createFileRoute } from '@tanstack/react-router';

import ToolsetsPage from '@/app/toolsets/page';

export const Route = createFileRoute('/toolsets/')({
  component: ToolsetsPage,
});
