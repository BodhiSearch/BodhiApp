import { createFileRoute } from '@tanstack/react-router';

import SetupApiModelsPage from '@/app/setup/api-models/page';

export const Route = createFileRoute('/setup/api-models/')({
  component: SetupApiModelsPage,
});
