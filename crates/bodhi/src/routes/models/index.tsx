import { createFileRoute } from '@tanstack/react-router';

import ModelsPage from '@/app/models/page';

export const Route = createFileRoute('/models/')({
  component: ModelsPage,
});
