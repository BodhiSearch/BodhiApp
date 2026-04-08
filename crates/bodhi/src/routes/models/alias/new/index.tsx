import React from 'react';

import type { AliasResponse } from '@bodhiapp/ts-client';
import { createFileRoute } from '@tanstack/react-router';
import { useSearch } from '@tanstack/react-router';
import { z } from 'zod';

import AliasForm from '../-components/AliasForm';
import AppInitializer from '@/components/AppInitializer';

export const Route = createFileRoute('/models/alias/new/')({
  validateSearch: z.object({
    repo: z.string().optional(),
    filename: z.string().optional(),
    snapshot: z.string().optional(),
  }),
  component: NewModel,
});

function NewModelContent() {
  const search = useSearch({ from: '/models/alias/new/' });

  const initialData =
    search.repo || search.filename
      ? ({
          id: '',
          source: 'user',
          alias: '',
          repo: search.repo || '',
          filename: search.filename || '',
          snapshot: search.snapshot || '',
          model_params: {},
          request_params: {},
          context_params: [],
          created_at: '',
          updated_at: '',
        } as AliasResponse)
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
