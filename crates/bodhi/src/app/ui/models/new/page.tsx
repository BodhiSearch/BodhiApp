import { lazy, Suspense } from 'react';

// Lazy load the actual page component
const NewModelPageContent = lazy(() => import('@/app/ui/models/new/NewModelPage'));

export default function NewModelPage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <NewModelPageContent />
    </Suspense>
  );
}
