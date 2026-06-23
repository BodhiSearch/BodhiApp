import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

import AppInitializer from '@/components/AppInitializer';

import { MultiTenantGuard } from '../-shared/MultiTenantGuard';

import { ExploreProvidersScreen } from './-components/ExploreProvidersScreen';

// `select` is the cross-link target from the API Models page ("Served by" → a provider).
const providersSearchSchema = z.object({ select: z.string().optional() });

export const Route = createFileRoute('/models/explore/api-providers/')({
  validateSearch: providersSearchSchema,
  component: ExploreProvidersPage,
});

export default function ExploreProvidersPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <MultiTenantGuard>
        <ExploreProvidersScreen />
      </MultiTenantGuard>
    </AppInitializer>
  );
}
