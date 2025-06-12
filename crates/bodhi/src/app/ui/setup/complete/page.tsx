import { lazy, Suspense } from 'react';

// Lazy load the actual page component
const SetupCompletePageContent = lazy(() => import('@/app/ui/setup/complete/SetupCompletePage'));

export default function SetupCompletePage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <SetupCompletePageContent />
    </Suspense>
  );
}
