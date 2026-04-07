import { createFileRoute } from '@tanstack/react-router';

import AppInitializer from '@/components/AppInitializer';

export const Route = createFileRoute('/home/')({
  component: HomePage,
});

export default function HomePage() {
  return <AppInitializer />;
}
