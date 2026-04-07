import { AliasResponse } from '@bodhiapp/ts-client';
import { createFileRoute } from '@tanstack/react-router';
import { useSearch } from '@tanstack/react-router';
import { z } from 'zod';

import AliasForm from '../-components/AliasForm';
import AppInitializer from '@/components/AppInitializer';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Loading } from '@/components/ui/Loading';
import { useGetModel } from '@/hooks/models';

export const Route = createFileRoute('/models/alias/edit/')({
  validateSearch: z.object({ id: z.string().optional() }),
  component: EditAliasPage,
});

function EditAliasContent() {
  const search = useSearch({ strict: false });
  const id = search.id;

  const { data: modelData, isLoading, error } = useGetModel(id ?? '');

  if (isLoading) {
    return <Loading message="Loading model data..." />;
  }

  if (error) {
    return <ErrorPage message="Error loading model data" />;
  }

  if (!modelData) {
    return <ErrorPage title="Not Found" message="No model data found" />;
  }

  return (
    <div className="container mx-auto px-4 sm:px-6 lg:px-8">
      <AliasForm isEditMode={true} initialData={modelData as AliasResponse} />
    </div>
  );
}

export default function EditAliasPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <EditAliasContent />
    </AppInitializer>
  );
}
