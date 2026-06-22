import React, { useMemo } from 'react';

import type { AliasResponse } from '@bodhiapp/ts-client';
import { createFileRoute } from '@tanstack/react-router';
import { useSearch } from '@tanstack/react-router';
import { z } from 'zod';

import AppInitializer from '@/components/AppInitializer';
import { useShellChrome } from '@/components/shell';

import AliasForm from '../-components/AliasForm';

export const Route = createFileRoute('/models/alias/new/')({
  validateSearch: z.object({
    repo: z.string().optional(),
    filename: z.string().optional(),
    snapshot: z.string().optional(),
  }),
  component: NewModel,
});

const NEW_LOCAL_MODEL_BREADCRUMB = [
  { label: 'Bodhi' },
  { label: 'Models', href: '/models/' },
  { label: 'New Local Model', current: true },
];

function NewModelContent() {
  const search = useSearch({ from: '/models/alias/new/' });
  useShellChrome({ breadcrumb: useMemo(() => NEW_LOCAL_MODEL_BREADCRUMB, []) });

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

  return (
    <div className="container mx-auto max-w-3xl px-4 py-6" data-testid="new-local-model-page">
      <AliasForm isEditMode={false} initialData={initialData} />
    </div>
  );
}

export default function NewModel() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <NewModelContent />
    </AppInitializer>
  );
}
