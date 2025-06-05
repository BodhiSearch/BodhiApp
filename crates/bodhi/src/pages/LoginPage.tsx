import { lazy, Suspense } from 'react';

// Lazy load the actual page component
const LoginPageContent = lazy(() => import('@/app/ui/login/page'));

export default function LoginPage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <LoginPageContent />
    </Suspense>
  );
}
