import { lazy, Suspense } from 'react';

// Lazy load the actual page component
const UsersPageContent = lazy(() => import('@/components/users/page'));

export default function UsersPage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <UsersPageContent />
    </Suspense>
  );
}
