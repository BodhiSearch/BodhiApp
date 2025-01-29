'use client';

import React from 'react';
import { useSearchParams } from 'next/navigation';
import AliasForm from '@/components/AliasForm';
import AppInitializer from '@/components/AppInitializer';

export default function CreateAliasPage() {
  const searchParams = useSearchParams();

  const initialData = {
    alias: '',
    repo: searchParams.get('repo') || '',
    filename: searchParams.get('filename') || '',
    snapshot: searchParams.get('snapshot') || '',
    chat_template: '',
    request_params: {},
    context_params: {},
  };

  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <AliasForm isEditMode={false} initialData={initialData} />
    </AppInitializer>
  );
}
