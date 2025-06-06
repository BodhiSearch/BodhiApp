import { lazy, Suspense } from 'react';

// Lazy load the actual page component
const TokensPageContent = lazy(() => import('@/components/tokens/TokensPage'));

export default function TokensPage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <TokensPageContent />
    </Suspense>
  );
}
