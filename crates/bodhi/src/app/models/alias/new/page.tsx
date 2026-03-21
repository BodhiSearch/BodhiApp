import React from 'react';

import { useSearch } from '@tanstack/react-router';

import AliasForm from '@/app/models/alias/components/AliasForm';
import AppInitializer from '@/components/AppInitializer';

function NewModelContent() {
  const search = useSearch({ strict: false });

  const initialData =
    search.repo || search.filename
      ? {
          source: 'user' as const,
          alias: '',
          repo: search.repo || '',
          filename: search.filename || '',
          snapshot: search.snapshot || '',
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
