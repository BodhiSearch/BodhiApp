import { createFileRoute } from '@tanstack/react-router';

import AppInitializer from '@/components/AppInitializer';

import { ModelsScreenV2 } from './-components/ModelsScreenV2';

export const Route = createFileRoute('/models/')({
  component: ModelsPage,
});

export default function ModelsPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <ModelsScreenV2 />
    </AppInitializer>
  );
}
