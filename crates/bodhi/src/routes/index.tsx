import { createFileRoute } from '@tanstack/react-router';

import AppInitializer from '@/components/AppInitializer';

export const Route = createFileRoute('/')({
  component: RootPage,
});

export default function RootPage() {
  return <AppInitializer />;
}
