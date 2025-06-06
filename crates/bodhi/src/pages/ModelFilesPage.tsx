import { lazy, Suspense } from 'react';

// Lazy load the actual page component
const ModelFilesPageContent = lazy(() => import('@/components/modelfiles/page'));

export default function ModelFilesPage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <ModelFilesPageContent />
    </Suspense>
  );
}
