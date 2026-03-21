import { createFileRoute } from '@tanstack/react-router';

import DownloadModelsPage from '@/app/setup/download-models/page';

export const Route = createFileRoute('/setup/download-models/')({
  component: DownloadModelsPage,
});
