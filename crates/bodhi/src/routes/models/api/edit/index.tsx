import { createFileRoute } from '@tanstack/react-router';
import { useSearch } from '@tanstack/react-router';
import { z } from 'zod';

import ApiModelForm from '@/components/api-models/ApiModelForm';
import AppInitializer from '@/components/AppInitializer';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Loading } from '@/components/ui/Loading';
import { useGetApiModel } from '@/hooks/models';

export const Route = createFileRoute('/models/api/edit/')({
  validateSearch: z.object({ id: z.string().optional() }),
  component: EditApiModel,
});

function EditApiModelContent() {
  const search = useSearch({ from: '/models/api/edit/' });
  const id = search.id;

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
    const errorMessage = error.response?.data?.error?.message || error.message || 'An unexpected error occurred';
    return <ErrorPage message={errorMessage} />;
  }

  if (!apiModel) {
    return <ErrorPage message="API model not found" />;
  }

  return <ApiModelForm mode="edit" initialData={apiModel} />;
}

export default function EditApiModel() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <EditApiModelContent />
    </AppInitializer>
  );
}
