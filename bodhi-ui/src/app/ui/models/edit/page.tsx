'use client';

import { Suspense } from 'react';
import { useSearchParams } from 'next/navigation';
import AliasForm from '@/components/AliasForm';
import AppHeader from '@/components/AppHeader';
import { useModel } from '@/hooks/useQuery';

function EditAliasContent() {
  const searchParams = useSearchParams();
  const alias = searchParams.get('alias');

  const { data: modelData, isLoading, error } = useModel(alias ?? '');

  if (isLoading) return <div>Loading...</div>;
  if (error) return <div>Error loading model data</div>;
  if (!modelData) return <div>No model data found</div>;

  return (
    <>{modelData && <AliasForm isEditMode={true} initialData={modelData} />}</>
  );
}

export default function EditAliasPage() {
  return (
    <div className="container mx-auto px-4 sm:px-6 lg:px-8">
      <AppHeader />
      <Suspense fallback={<div>Loading...</div>}>
        <EditAliasContent />
      </Suspense>
    </div>
  );
}
