import { lazy, Suspense } from 'react';

// Lazy load the 404 page component
const NotFoundPageContent = lazy(() => import('@/app/_not-found/page'));

export default function NotFoundPage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <NotFoundPageContent />
    </Suspense>
  );
}
