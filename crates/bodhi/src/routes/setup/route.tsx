import { ReactNode } from 'react';

import { createFileRoute, Outlet } from '@tanstack/react-router';

import { SetupProvider } from './-components/SetupProvider';

function SetupLayoutComponent({ children }: { children: ReactNode }) {
  return (
    <SetupProvider>
      <main className="min-h-screen bg-background">{children}</main>
    </SetupProvider>
  );
}

export const Route = createFileRoute('/setup')({
  component: () => (
    <SetupLayoutComponent>
      <Outlet />
    </SetupLayoutComponent>
  ),
});
