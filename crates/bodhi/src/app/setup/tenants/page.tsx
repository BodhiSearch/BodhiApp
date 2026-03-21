'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';

import { AxiosResponse } from 'axios';
import { RedirectResponse } from '@bodhiapp/ts-client';

import AppInitializer from '@/components/AppInitializer';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Loading } from '@/components/ui/Loading';
import { Textarea } from '@/components/ui/textarea';
import { useOAuthInitiate } from '@/hooks/auth';
import { useCreateTenant } from '@/hooks/tenants';

function TenantRegistrationContent() {
  const router = useRouter();
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [isRedirecting, setIsRedirecting] = useState(false);

  const { mutate: oauthInitiate } = useOAuthInitiate({
    onSuccess: (response: AxiosResponse<RedirectResponse>) => {
      const location = response.data.location;
      if (location.startsWith('http')) {
        window.location.href = location;
      } else {
        router.push(location);
      }
    },
    onError: (message: string) => {
      setError(message);
      setIsRedirecting(false);
    },
  });

  const { mutate: createTenant, isPending: isCreating } = useCreateTenant({
    onSuccess: (response) => {
      setIsRedirecting(true);
      oauthInitiate({ client_id: response.client_id });
    },
    onError: (message: string) => {
      setError(message);
    },
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (name.length < 3) {
      setError('Name must be at least 3 characters');
      return;
    }

    createTenant({ name, description });
  };

  if (isRedirecting) {
    return <Loading message="Setting up your workspace..." />;
  }

  return (
    <div className="flex min-h-screen items-center justify-center">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle>Create Workspace</CardTitle>
          <CardDescription>Set up your new workspace to get started with Bodhi</CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-4">
            {error && (
              <Alert variant="destructive">
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}
            <div className="space-y-2">
              <Label htmlFor="name">Workspace Name</Label>
              <Input
                id="name"
                data-testid="tenant-name-input"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="My Workspace"
                disabled={isCreating}
                required
                minLength={3}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="description">Description (optional)</Label>
              <Textarea
                id="description"
                data-testid="tenant-description-input"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="A brief description of your workspace"
                disabled={isCreating}
              />
            </div>
            <Button type="submit" data-testid="create-tenant-button" className="w-full" disabled={isCreating}>
              {isCreating ? 'Creating...' : 'Create Workspace'}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}

export default function TenantRegistrationPage() {
  return (
    <AppInitializer allowedStatus="setup" authenticated={false}>
      <TenantRegistrationContent />
    </AppInitializer>
  );
}
