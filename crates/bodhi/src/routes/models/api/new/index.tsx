import { useMemo } from 'react';

import { createFileRoute, useSearch } from '@tanstack/react-router';
import { z } from 'zod';

import ApiModelForm, { type ApiModelPrefill } from '@/components/api-models/ApiModelForm';
import AppInitializer from '@/components/AppInitializer';
import { useShellChrome } from '@/components/shell';

// Prefill params for the Explore catalog "Configure in Bodhi" bridge. All optional — a bare
// /models/api/new still renders the empty create form.
const newApiModelSearchSchema = z.object({
  api_format: z.string().optional(),
  base_url: z.string().optional(),
  model: z.string().optional(),
  name: z.string().optional(),
});

export const Route = createFileRoute('/models/api/new/')({
  staticData: { section: 'models', subPage: 'new-api-model' },
  validateSearch: newApiModelSearchSchema,
  component: NewApiModel,
});

const NEW_API_MODEL_BREADCRUMB = [
  { label: 'Bodhi' },
  { label: 'Models', href: '/models/' },
  { label: 'New API Model', current: true },
];

function NewApiModelContent() {
  useShellChrome({ breadcrumb: useMemo(() => NEW_API_MODEL_BREADCRUMB, []) });
  // Loose useSearch (not Route.useSearch) so component tests that mock @tanstack/react-router's
  // useSearch intercept it without a router context.
  const search = useSearch({ strict: false }) as {
    api_format?: string;
    base_url?: string;
    model?: string;
    name?: string;
  };

  const prefill: ApiModelPrefill | undefined = useMemo(() => {
    if (!search.api_format && !search.base_url && !search.model && !search.name) return undefined;
    return { api_format: search.api_format, base_url: search.base_url, model: search.model, name: search.name };
  }, [search.api_format, search.base_url, search.model, search.name]);

  return (
    <div className="container mx-auto max-w-3xl px-4 py-6" data-testid="new-api-model-page">
      <ApiModelForm mode="create" prefill={prefill} />
    </div>
  );
}

export default function NewApiModel() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <NewApiModelContent />
    </AppInitializer>
  );
}
