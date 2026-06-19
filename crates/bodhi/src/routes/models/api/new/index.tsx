import { useMemo } from 'react';

import { createFileRoute } from '@tanstack/react-router';

import ApiModelForm from '@/components/api-models/ApiModelForm';
import AppInitializer from '@/components/AppInitializer';
import { useShellChrome } from '@/components/shell';

export const Route = createFileRoute('/models/api/new/')({
  component: NewApiModel,
});

const NEW_API_MODEL_BREADCRUMB = [
  { label: 'Bodhi' },
  { label: 'Models', href: '/models/' },
  { label: 'New API Model', current: true },
];

function NewApiModelContent() {
  useShellChrome({ breadcrumb: useMemo(() => NEW_API_MODEL_BREADCRUMB, []) });

  return (
    <div className="container mx-auto max-w-3xl px-4 py-6" data-testid="new-api-model-page">
      <ApiModelForm mode="create" />
    </div>
  );
}

export default function NewApiModel() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <NewApiModelContent />
    </AppInitializer>
  );
}
