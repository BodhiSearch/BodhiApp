import { lazy, Suspense } from 'react';

// Lazy load the 404 page component
const NotFoundPageContent = lazy(
  () => import('@/components/not-found/NotFoundPageContent')
);

export default function NotFoundPage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <NotFoundPageContent />
    </Suspense>
  );
}
