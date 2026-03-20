'use client';

import { ReactNode } from 'react';

import { SetupProvider } from './components/SetupProvider';

export default function SetupLayout({ children }: { children: ReactNode }) {
  return (
    <SetupProvider>
      <main className="min-h-screen bg-background">{children}</main>
    </SetupProvider>
  );
}
