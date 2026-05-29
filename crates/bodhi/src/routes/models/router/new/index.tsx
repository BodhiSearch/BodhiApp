import { createFileRoute } from '@tanstack/react-router';

import AppInitializer from '@/components/AppInitializer';
import ModelRouterForm from '@/routes/models/router/-components/ModelRouterForm';

export const Route = createFileRoute('/models/router/new/')({
  component: NewModelRouter,
});

export default function NewModelRouter() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <ModelRouterForm mode="create" />
    </AppInitializer>
  );
}
