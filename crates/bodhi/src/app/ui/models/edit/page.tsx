import { lazy, Suspense } from 'react';

// Lazy load the actual page component
const EditModelPageContent = lazy(() => import('@/app/ui/models/edit/EditModelPage'));

export default function EditModelPage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <EditModelPageContent />
    </Suspense>
  );
}
