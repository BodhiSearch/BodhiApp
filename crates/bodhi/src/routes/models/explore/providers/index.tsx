import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

import AppInitializer from '@/components/AppInitializer';
import { MultiTenantGuard } from '@/routes/models/explore/-shared/MultiTenantGuard';

import { ExploreProvidersScreen } from './-components/ExploreProvidersScreen';

// `select` is the cross-link target from the API Models page ("Served by" → a provider).
// `q` seeds the search box from the "View" cross-link (one-shot; the page's own back/forward + field
// prepop is a follow-up iteration).
const providersSearchSchema = z.object({ select: z.string().optional(), q: z.string().optional() });

export const Route = createFileRoute('/models/explore/providers/')({
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
