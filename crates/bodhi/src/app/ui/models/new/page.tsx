'use client';

import React from 'react';
import { useSearchParams } from 'next/navigation';
import AliasForm from '@/app/ui/models/AliasForm';
import AppInitializer from '@/components/AppInitializer';

function NewModelContent() {
  const searchParams = useSearchParams();

  const initialData =
    searchParams?.get('repo') || searchParams?.get('filename')
      ? {
          alias: '',
          repo: searchParams?.get('repo') || '',
          filename: searchParams?.get('filename') || '',
          snapshot: searchParams?.get('snapshot') || '',
          source: 'user',
          model_params: {},
          request_params: {},
          context_params: [],
        }
      : undefined;

  return <AliasForm isEditMode={false} initialData={initialData} />;
}

export default function NewModel() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <NewModelContent />
    </AppInitializer>
  );
}
