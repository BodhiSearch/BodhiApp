import { createFileRoute } from '@tanstack/react-router';

import ToolsetsAdminPage from '@/app/toolsets/admin/page';

export const Route = createFileRoute('/toolsets/admin/')({
  component: ToolsetsAdminPage,
});
