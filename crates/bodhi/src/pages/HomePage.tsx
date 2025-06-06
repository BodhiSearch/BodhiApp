import { lazy, Suspense } from 'react';

// Lazy load the actual page component
const HomePageContent = lazy(() => import('@/components/home/HomePage'));

export default function HomePage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <HomePageContent />
    </Suspense>
  );
}
