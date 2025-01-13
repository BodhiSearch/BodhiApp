'use client';

import { TokenDialog } from '@/app/ui/tokens/TokenDialog';
import { TokenForm } from '@/app/ui/tokens/TokenForm';
import AppInitializer from '@/components/AppInitializer';
import { Alert, AlertDescription } from '@/components/ui/alert';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Skeleton } from '@/components/ui/skeleton';
import { TokenResponse } from '@/hooks/useCreateToken';
import { useAppInfo } from '@/hooks/useQuery';
import { Shield } from 'lucide-react';
import { useState } from 'react';

export function TokenPageContent() {
  const [token, setToken] = useState<TokenResponse | null>(null);
  const { data: appInfo, isLoading: appLoading } = useAppInfo();

  const handleTokenCreated = (newToken: TokenResponse) => {
    setToken(newToken);
  };

  const handleDialogClose = () => {
    setToken(null);
  };

  if (appLoading) {
    return (
      <div
        className="container mx-auto px-4 sm:px-6 lg:px-8 py-6"
        data-testid="token-page-loading"
      >
        <Card>
          <CardHeader>
            <div className="flex items-center gap-2">
              <Skeleton className="h-5 w-5" />
              <Skeleton className="h-8 w-32" />
            </div>
            <Skeleton className="h-4 w-3/4 mt-2" />
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <Skeleton className="h-10 w-full" />
              <Skeleton className="h-10 w-1/4" />
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  if (!appInfo?.authz) {
    return (
      <div className="container mx-auto px-4 sm:px-6 lg:px-8 py-6">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Shield className="h-5 w-5" />
              API Tokens Not Supported
            </CardTitle>
            <CardDescription>
              Non-authenticated setup doesn&apos;t need API Tokens. Either
              ignore the Auth header or pass an empty/random Bearer token. They
              are not validated.
            </CardDescription>
          </CardHeader>
        </Card>
      </div>
    );
  }

  return (
    <div className="container mx-auto px-4 sm:px-6 lg:px-8 py-6">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            API Tokens
          </CardTitle>
          <CardDescription>
            Generate and manage API tokens for programmatic access to the API
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Alert>
            <Shield className="h-4 w-4" />
            <AlertDescription>
              API tokens provide full access to the API. Keep them secure and
              never share them. Tokens cannot be viewed again after creation.
            </AlertDescription>
          </Alert>
          <div className="mt-6">
            <TokenForm onTokenCreated={handleTokenCreated} />
          </div>
        </CardContent>
      </Card>

      {token && (
        <TokenDialog token={token} open={!!token} onClose={handleDialogClose} />
      )}
    </div>
  );
}

export default function TokenPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <TokenPageContent />
    </AppInitializer>
  );
}
