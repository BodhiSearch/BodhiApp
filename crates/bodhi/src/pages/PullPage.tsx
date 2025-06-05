import { lazy, Suspense } from 'react';

// Lazy load the actual page component
const PullPageContent = lazy(() => import('@/app/ui/pull/page'));

export default function PullPage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <PullPageContent />
    </Suspense>
  );
}
