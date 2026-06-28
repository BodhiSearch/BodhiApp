import { useMemo } from 'react';

import { createFileRoute } from '@tanstack/react-router';
import { useSearch } from '@tanstack/react-router';
import { z } from 'zod';

import ApiModelForm from '@/components/api-models/ApiModelForm';
import AppInitializer from '@/components/AppInitializer';
import { useShellChrome } from '@/components/shell';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Loading } from '@/components/ui/Loading';
import { useGetApiModel } from '@/hooks/models';
import { extractErrorMessage } from '@/lib/errorUtils';

export const Route = createFileRoute('/models/api/edit/')({
  staticData: { section: 'models' },
  validateSearch: z.object({ id: z.string().optional() }),
  component: EditApiModel,
});

const EDIT_API_MODEL_BREADCRUMB = [
  { label: 'Bodhi' },
  { label: 'Models', href: '/models/' },
  { label: 'Edit API Model', current: true },
];

function EditApiModelContent() {
  const search = useSearch({ from: '/models/api/edit/' });
  const id = search.id;

  // Publish the breadcrumb up-front so the shell header is populated through the loading/error states.
  useShellChrome({ breadcrumb: useMemo(() => EDIT_API_MODEL_BREADCRUMB, []) });

  const {
    data: apiModel,
    isLoading,
    error,
  } = useGetApiModel(id || '', {
    enabled: !!id,
  });

  if (!id) {
    return <ErrorPage message="No API model ID provided" />;
  }

  if (isLoading) {
    return <Loading message="Loading API model..." />;
  }

  if (error) {
    const errorMessage = extractErrorMessage(error, 'An unexpected error occurred');
    return <ErrorPage message={errorMessage} />;
  }

  if (!apiModel) {
    return <ErrorPage message="API model not found" />;
  }

  return (
    <div className="container mx-auto max-w-3xl px-4 py-6" data-testid="edit-api-model-page">
      <ApiModelForm mode="edit" initialData={apiModel} />
    </div>
  );
}

export default function EditApiModel() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <EditApiModelContent />
    </AppInitializer>
  );
}
