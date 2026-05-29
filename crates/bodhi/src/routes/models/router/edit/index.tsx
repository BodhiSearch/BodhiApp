import { createFileRoute, useSearch } from '@tanstack/react-router';
import { z } from 'zod';

import AppInitializer from '@/components/AppInitializer';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Loading } from '@/components/ui/Loading';
import { useGetModelRouter } from '@/hooks/models';
import ModelRouterForm from '@/routes/models/router/-components/ModelRouterForm';

export const Route = createFileRoute('/models/router/edit/')({
  validateSearch: z.object({ id: z.string().optional() }),
  component: EditModelRouter,
});

function EditModelRouterContent() {
  const search = useSearch({ from: '/models/router/edit/' });
  const id = search.id;

  const { data: router, isLoading, error } = useGetModelRouter(id || '', { enabled: !!id });

  if (!id) {
    return <ErrorPage message="No model router ID provided" />;
  }
  if (isLoading) {
    return <Loading message="Loading model router..." />;
  }
  if (error) {
    const errorMessage = error.response?.data?.error?.message || error.message || 'An unexpected error occurred';
    return <ErrorPage message={errorMessage} />;
  }
  if (!router) {
    return <ErrorPage message="Model router not found" />;
  }
  return <ModelRouterForm mode="edit" initialData={router} />;
}

export default function EditModelRouter() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <EditModelRouterContent />
    </AppInitializer>
  );
}
