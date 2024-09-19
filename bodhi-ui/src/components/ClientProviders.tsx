'use client';

import { QueryClient, QueryClientProvider } from 'react-query';
import { useState } from 'react';
import { UserProvider } from '@/hooks/useUserContext';

export default function ClientProviders({
  children,
}: {
  children: React.ReactNode;
}) {
  const [queryClient] = useState(() => new QueryClient());

  return (
    <QueryClientProvider client={queryClient}>
      <UserProvider>{children}</UserProvider>
    </QueryClientProvider>
  );
}
