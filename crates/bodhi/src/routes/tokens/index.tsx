import { createFileRoute } from '@tanstack/react-router';

import TokensPage from '@/app/tokens/page';

export const Route = createFileRoute('/tokens/')({
  component: TokensPage,
});
