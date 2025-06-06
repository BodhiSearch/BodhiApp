import { lazy, Suspense } from 'react';

// Lazy load the actual page component
const ModelsPageContent = lazy(() => import('@/components/models/ModelsPage'));

export default function ModelsPage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <ModelsPageContent />
    </Suspense>
  );
}
