import { createFileRoute } from '@tanstack/react-router';

import AppInitializer from '@/components/AppInitializer';
import { MultiTenantGuard } from '@/routes/models/explore/-shared/MultiTenantGuard';

import { LocalDiscoveryScreen } from './-components/LocalDiscoveryScreen';

export const Route = createFileRoute('/models/explore/local/')({
  component: LocalDiscoverPage,
});

export default function LocalDiscoverPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <MultiTenantGuard>
        <LocalDiscoveryScreen />
      </MultiTenantGuard>
    </AppInitializer>
  );
}
