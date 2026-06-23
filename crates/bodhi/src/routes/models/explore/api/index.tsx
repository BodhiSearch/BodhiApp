import { createFileRoute } from '@tanstack/react-router';

import AppInitializer from '@/components/AppInitializer';

import { MultiTenantGuard } from '../-shared/MultiTenantGuard';

import { ExploreApiScreen } from './-components/ExploreApiScreen';

export const Route = createFileRoute('/models/explore/api/')({
  component: ExploreApiPage,
});

export default function ExploreApiPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <MultiTenantGuard>
        <ExploreApiScreen />
      </MultiTenantGuard>
    </AppInitializer>
  );
}
