'use client';

import { useSearchParams } from 'next/navigation';
import AliasForm from '@/app/ui/models/AliasForm';
import { useModel } from '@/hooks/useModels';
import AppInitializer from '@/components/AppInitializer';
import { Loading } from '@/components/ui/Loading';
import { ErrorPage } from '@/components/ui/ErrorPage';

function EditAliasContent() {
  const searchParams = useSearchParams();
  const alias = searchParams?.get('alias');

  const { data: modelData, isLoading, error } = useModel(alias ?? '');

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
      <AliasForm isEditMode={true} initialData={modelData} />
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
