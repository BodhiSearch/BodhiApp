import { useMemo } from 'react';

import { AliasResponse } from '@bodhiapp/ts-client';
import { createFileRoute } from '@tanstack/react-router';
import { useSearch } from '@tanstack/react-router';
import { z } from 'zod';

import AppInitializer from '@/components/AppInitializer';
import { useShellChrome } from '@/components/shell';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Loading } from '@/components/ui/Loading';
import { useGetModel } from '@/hooks/models';
import AliasForm from '@/routes/models/alias/-components/AliasForm';

export const Route = createFileRoute('/models/alias/edit/')({
  staticData: { section: 'models' },
  validateSearch: z.object({ id: z.string().optional() }),
  component: EditAliasPage,
});

const EDIT_LOCAL_MODEL_BREADCRUMB = [
  { label: 'Bodhi' },
  { label: 'Models', href: '/models/' },
  { label: 'Edit Local Model', current: true },
];

function EditAliasContent() {
  const search = useSearch({ from: '/models/alias/edit/' });
  const id = search.id;
  useShellChrome({ breadcrumb: useMemo(() => EDIT_LOCAL_MODEL_BREADCRUMB, []) });

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
    <div className="container mx-auto max-w-3xl px-4 py-6" data-testid="edit-local-model-page">
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
