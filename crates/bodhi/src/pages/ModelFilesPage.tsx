import { lazy, Suspense } from 'react';

// Lazy load the actual page component
const ModelFilesPageContent = lazy(() => import('@/app/ui/modelfiles/page'));

export default function ModelFilesPage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <ModelFilesPageContent />
    </Suspense>
  );
}
