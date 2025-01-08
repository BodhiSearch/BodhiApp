'use client';

import { useSearchParams } from 'next/navigation';
import AliasForm from '@/components/AliasForm';
import { useModel } from '@/hooks/useQuery';
import AppInitializer from '@/components/AppInitializer';
import { MainLayout } from '@/components/layout/MainLayout';

function EditAliasContent() {
  const searchParams = useSearchParams();
  const alias = searchParams.get('alias');

  const { data: modelData, isLoading, error } = useModel(alias ?? '');

  if (isLoading) return <div>Loading...</div>;
  if (error) return <div>Error loading model data</div>;
  if (!modelData) return <div>No model data found</div>;

  return (
    <MainLayout>
      <div className="container mx-auto px-4 sm:px-6 lg:px-8">
        {modelData && <AliasForm isEditMode={true} initialData={modelData} />}
      </div>
    </MainLayout>
  );
}

export default function EditAliasPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <EditAliasContent />
    </AppInitializer>
  );
}
