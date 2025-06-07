import { lazy, Suspense } from 'react';

// Lazy load the actual page component
const SettingsPageContent = lazy(
  () => import('@/components/settings/SettingsPage')
);

export default function SettingsPage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <SettingsPageContent />
    </Suspense>
  );
}
