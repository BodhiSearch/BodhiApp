import { useMemo } from 'react';

import { createFileRoute, useSearch } from '@tanstack/react-router';
import { z } from 'zod';

import AppInitializer from '@/components/AppInitializer';
import { useShellChrome } from '@/components/shell';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Loading } from '@/components/ui/Loading';
import { useGetModelRouter } from '@/hooks/models';
import { extractErrorMessage } from '@/lib/errorUtils';
import ModelRouterForm from '@/routes/models/router/-components/ModelRouterForm';

export const Route = createFileRoute('/models/router/edit/')({
  validateSearch: z.object({ id: z.string().optional() }),
  component: EditModelRouter,
});

const EDIT_ROUTER_BREADCRUMB = [
  { label: 'Bodhi' },
  { label: 'Models', href: '/models/' },
  { label: 'Edit Model Router', current: true },
];

/** Publishes the breadcrumb during loading/error states (the form publishes it once mounted). */
function BreadcrumbOnly({ children }: { children: React.ReactNode }) {
  useShellChrome({ breadcrumb: useMemo(() => EDIT_ROUTER_BREADCRUMB, []) });
  return <>{children}</>;
}

function EditModelRouterContent() {
  const search = useSearch({ from: '/models/router/edit/' });
  const id = search.id;

  const { data: router, isLoading, error } = useGetModelRouter(id || '', { enabled: !!id });

  if (!id) {
    return (
      <BreadcrumbOnly>
        <ErrorPage message="No model router ID provided" />
      </BreadcrumbOnly>
    );
  }
  if (isLoading) {
    return (
      <BreadcrumbOnly>
        <Loading message="Loading model router..." />
      </BreadcrumbOnly>
    );
  }
  if (error) {
    const errorMessage = extractErrorMessage(error, 'An unexpected error occurred');
    return (
      <BreadcrumbOnly>
        <ErrorPage message={errorMessage} />
      </BreadcrumbOnly>
    );
  }
  if (!router) {
    return (
      <BreadcrumbOnly>
        <ErrorPage message="Model router not found" />
      </BreadcrumbOnly>
    );
  }
  return (
    <div className="container mx-auto max-w-3xl px-4 py-6" data-testid="router-form-page" data-pagestatus="ready">
      <ModelRouterForm mode="edit" initialData={router} breadcrumb={EDIT_ROUTER_BREADCRUMB} />
    </div>
  );
}

export default function EditModelRouter() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <EditModelRouterContent />
    </AppInitializer>
  );
}
