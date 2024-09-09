'use client';

import { Suspense } from 'react';
import AppInitializer from '@/components/AppInitializer';

export default function UiPage() {
  return (
    <main className="min-h-screen flex items-center justify-center">
      <Suspense fallback={<div>Loading...</div>}>
        <AppInitializer />
      </Suspense>
    </main>
  );
}
