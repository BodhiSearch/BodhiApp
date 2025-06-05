import { lazy, Suspense } from 'react';

// Lazy load the actual page component
const HomePageContent = lazy(() => import('@/app/ui/home/page'));

export default function HomePage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <HomePageContent />
    </Suspense>
  );
}
