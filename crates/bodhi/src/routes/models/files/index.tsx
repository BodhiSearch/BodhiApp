import { createFileRoute } from '@tanstack/react-router';

import FilesPage from '@/app/models/files/page';

export const Route = createFileRoute('/models/files/')({
  component: FilesPage,
});
