import { lazy, Suspense } from 'react';

// Lazy load the actual page component
const SettingsPageContent = lazy(() => import('@/app/ui/settings/page'));

export default function SettingsPage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <SettingsPageContent />
    </Suspense>
  );
}
