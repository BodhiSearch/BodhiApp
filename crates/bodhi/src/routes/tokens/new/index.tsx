import { useMemo, useState } from 'react';

import { TokenCreated } from '@bodhiapp/ts-client';
import { createFileRoute, useNavigate } from '@tanstack/react-router';

import AppInitializer from '@/components/AppInitializer';
import { useShellChrome } from '@/components/shell';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

import { TokenDialog } from '../-components/TokenDialog';
import { TokenForm } from '../-components/TokenForm';
import '@/components/shell/api-keys.css';

export const Route = createFileRoute('/tokens/new/')({
  component: NewTokenPage,
});

const NEW_TOKEN_BREADCRUMB = [
  { label: 'Bodhi' },
  { label: 'Access Tokens', href: '/tokens/' },
  { label: 'New API Token', current: true },
];

function NewTokenContent() {
  const navigate = useNavigate();
  const [createdToken, setCreatedToken] = useState<TokenCreated | null>(null);

  useShellChrome({ breadcrumb: useMemo(() => NEW_TOKEN_BREADCRUMB, []) });

  const handleDone = () => {
    setCreatedToken(null);
    navigate({ to: '/tokens/' });
  };

  return (
    <div className="api-keys-screen container mx-auto max-w-2xl p-6" data-testid="new-token-page">
      <Card>
        <CardHeader>
          <CardTitle>New API Token</CardTitle>
          <CardDescription>Generate a scoped token for programmatic access to the Bodhi API.</CardDescription>
        </CardHeader>
        <CardContent>
          <TokenForm onTokenCreated={setCreatedToken} />
        </CardContent>
      </Card>
      {createdToken && <TokenDialog token={createdToken} open={true} onClose={handleDone} />}
    </div>
  );
}

function NewTokenPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <NewTokenContent />
    </AppInitializer>
  );
}
