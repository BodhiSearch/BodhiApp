import { createFileRoute } from '@tanstack/react-router';

import ApiModelForm from '@/components/api-models/ApiModelForm';
import AppInitializer from '@/components/AppInitializer';

export const Route = createFileRoute('/models/api/new/')({
  component: NewApiModel,
});

export default function NewApiModel() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <ApiModelForm mode="create" />
    </AppInitializer>
  );
}
