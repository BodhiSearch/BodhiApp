import { lazy, Suspense } from 'react';

// Lazy load the actual page component
const OAuthCallbackPageContent = lazy(() => import('@/app/ui/auth/callback/OAuthCallbackPage'));

export default function OAuthCallbackPage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <OAuthCallbackPageContent />
    </Suspense>
  );
}
