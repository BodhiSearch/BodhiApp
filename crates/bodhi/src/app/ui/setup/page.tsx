import { lazy, Suspense } from 'react';

// Lazy load the actual page component
const SetupPageContent = lazy(() => import('@/app/ui/setup/SetupPage'));

export default function SetupPage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <SetupPageContent />
    </Suspense>
  );
}
