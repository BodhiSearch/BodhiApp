import { createFileRoute } from '@tanstack/react-router';

import AppInitializer from '@/components/AppInitializer';
import ModelRouterForm from '@/routes/models/router/-components/ModelRouterForm';

export const Route = createFileRoute('/models/router/new/')({
  component: NewModelRouter,
});

const NEW_ROUTER_BREADCRUMB = [
  { label: 'Bodhi' },
  { label: 'Models', href: '/models/' },
  { label: 'New Model Router', current: true },
];

export default function NewModelRouter() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <div className="container mx-auto max-w-3xl px-4 py-6" data-testid="router-form-page" data-pagestatus="ready">
        <ModelRouterForm mode="create" breadcrumb={NEW_ROUTER_BREADCRUMB} />
      </div>
    </AppInitializer>
  );
}
