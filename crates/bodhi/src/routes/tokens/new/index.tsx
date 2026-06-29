import { useMemo, useState } from 'react';

import { TokenCreated } from '@bodhiapp/ts-client';
import { createFileRoute, useNavigate } from '@tanstack/react-router';

import AppInitializer from '@/components/AppInitializer';
import { useShellChrome } from '@/components/shell';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { TokenDialog } from '@/routes/tokens/-components/TokenDialog';
import { TokenForm } from '@/routes/tokens/-components/TokenForm';
import '@/components/shell/api-keys.css';
import '@/components/shell/new-token.css';

export const Route = createFileRoute('/tokens/new/')({
  staticData: { section: 'api-keys', subPage: 'new-token' },
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
    <div className="new-token api-keys-screen container mx-auto max-w-2xl p-6" data-testid="new-token-page">
      <Card>
        <CardHeader>
          <CardTitle>New App Token</CardTitle>
          <CardDescription>
            Generate a scoped token for API access — pick the models, MCPs, and capabilities it can use.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <TokenForm onTokenCreated={setCreatedToken} onCancel={handleDone} />
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
