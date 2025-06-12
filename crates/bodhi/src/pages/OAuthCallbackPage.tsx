import { lazy, Suspense } from 'react';

// Lazy load the actual page component
const OAuthCallbackPageContent = lazy(() => import('@/components/auth/OAuthCallbackPage'));

export default function OAuthCallbackPage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <OAuthCallbackPageContent />
    </Suspense>
  );
}
