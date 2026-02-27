'use client';

import { useEffect, useState } from 'react';

import { ChevronDown, ChevronRight } from 'lucide-react';
import { useRouter } from 'next/navigation';

import AppInitializer from '@/components/AppInitializer';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { Textarea } from '@/components/ui/textarea';
import { toast } from '@/hooks/use-toast';
import { useCreateMcpServer, useStandaloneDynamicRegister, type CreateMcpAuthConfigRequest } from '@/hooks/useMcps';
import { extractSecondLevelDomain } from '@/lib/urlUtils';
import { AuthConfigForm } from '../components/AuthConfigForm';

type AuthConfigType = 'none' | 'header' | 'oauth';
type OAuthRegistrationType = 'pre_registered' | 'dynamic_registration';

function NewMcpServerContent() {
  const router = useRouter();
  const [url, setUrl] = useState('');
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [enabled, setEnabled] = useState(true);
  const [errors, setErrors] = useState<Record<string, string>>({});

  // Auth config state
  const [showAuthConfig, setShowAuthConfig] = useState(false);
  const [authConfigType, setAuthConfigType] = useState<AuthConfigType>('none');
  const [oauthRegistrationType, setOauthRegistrationType] = useState<OAuthRegistrationType>('pre_registered');
  const [authName, setAuthName] = useState('');
  const [headerKey, setHeaderKey] = useState('');
  const [headerValue, setHeaderValue] = useState('');
  const [clientId, setClientId] = useState('');
  const [clientSecret, setClientSecret] = useState('');
  const [authEndpoint, setAuthEndpoint] = useState('');
  const [tokenEndpoint, setTokenEndpoint] = useState('');
  const [registrationEndpoint, setRegistrationEndpoint] = useState('');
  const [scopes, setScopes] = useState('');

  const standaloneDcr = useStandaloneDynamicRegister({
    onError: (message) => {
      toast({ title: 'Dynamic registration failed', description: message, variant: 'destructive' });
    },
  });

  const createMutation = useCreateMcpServer({
    onSuccess: () => {
      toast({ title: 'MCP server created' });
      router.push('/ui/mcp-servers');
    },
    onError: (message) => {
      toast({ title: 'Failed to create MCP server', description: message, variant: 'destructive' });
    },
  });

  const validate = () => {
    const newErrors: Record<string, string> = {};
    if (!name.trim()) newErrors.name = 'Name is required';
    if (name.length > 100) newErrors.name = 'Name cannot exceed 100 characters';
    if (!url.trim()) newErrors.url = 'URL is required';
    if (url.length > 2048) newErrors.url = 'URL cannot exceed 2048 characters';
    try {
      if (url.trim()) new URL(url.trim());
    } catch {
      newErrors.url = 'URL is not valid';
    }
    if (description.length > 255) newErrors.description = 'Description cannot exceed 255 characters';
    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const buildAuthConfig = (): CreateMcpAuthConfigRequest | undefined => {
    if (!showAuthConfig || authConfigType === 'none') return undefined;

    if (authConfigType === 'header') {
      return {
        type: 'header',
        name: authName || 'Header',
        header_key: headerKey,
        header_value: headerValue,
      };
    }

    if (authConfigType === 'oauth' && oauthRegistrationType === 'pre_registered') {
      return {
        type: 'oauth',
        name: authName || 'OAuth',
        client_id: clientId,
        client_secret: clientSecret || undefined,
        authorization_endpoint: authEndpoint,
        token_endpoint: tokenEndpoint,
        scopes: scopes || undefined,
        registration_type: 'pre_registered',
      };
    }

    // dynamic-registration: client_id comes from DCR result (handled in handleSubmit)
    return undefined;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!validate()) return;

    if (showAuthConfig && authConfigType === 'oauth' && oauthRegistrationType === 'dynamic_registration') {
      // Step 1: Call standalone DCR to get client credentials
      if (!registrationEndpoint) {
        toast({ title: 'Registration endpoint is required for dynamic registration', variant: 'destructive' });
        return;
      }
      const redirectUri = `${window.location.origin}/ui/mcps/oauth/callback`;
      try {
        const dcrResponse = await standaloneDcr.mutateAsync({
          registration_endpoint: registrationEndpoint,
          redirect_uri: redirectUri,
          scopes: scopes || undefined,
        });
        const dcrResult = dcrResponse.data;

        // Step 2: Create server with OAuth auth config containing DCR results
        createMutation.mutate({
          url: url.trim(),
          name: name.trim(),
          description: description.trim() || undefined,
          enabled,
          auth_config: {
            type: 'oauth',
            name: authName || 'OAuth',
            registration_type: 'dynamic_registration',
            authorization_endpoint: authEndpoint,
            token_endpoint: tokenEndpoint,
            registration_endpoint: registrationEndpoint || undefined,
            scopes: scopes || undefined,
            client_id: dcrResult.client_id,
            client_secret: dcrResult.client_secret ?? undefined,
            token_endpoint_auth_method: dcrResult.token_endpoint_auth_method ?? undefined,
            client_id_issued_at: dcrResult.client_id_issued_at ?? undefined,
            registration_access_token: dcrResult.registration_access_token ?? undefined,
          },
        });
      } catch {
        // Error already handled by hook's onError
      }
      return;
    }

    // Non-DCR path: directly create server with optional auth config
    createMutation.mutate({
      url: url.trim(),
      name: name.trim(),
      description: description.trim() || undefined,
      enabled,
      auth_config: buildAuthConfig(),
    });
  };

  const isSaving = createMutation.isLoading || standaloneDcr.isLoading;

  return (
    <div className="container mx-auto p-4 max-w-2xl" data-testid="new-mcp-server-page">
      <Card>
        <CardHeader>
          <CardTitle>New MCP Server</CardTitle>
          <CardDescription>Register a new MCP server for users to connect to.</CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="url">URL *</Label>
              <Input
                id="url"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                onBlur={() => {
                  if (!name.trim() && url.trim()) {
                    setName(extractSecondLevelDomain(url.trim()));
                  }
                }}
                placeholder="https://mcp.example.com/mcp"
                data-testid="mcp-server-url-input"
              />
              {errors.url && <p className="text-sm text-destructive">{errors.url}</p>}
            </div>

            <div className="space-y-2">
              <Label htmlFor="name">Name *</Label>
              <Input
                id="name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="My MCP Server"
                data-testid="mcp-server-name-input"
              />
              {errors.name && <p className="text-sm text-destructive">{errors.name}</p>}
            </div>

            <div className="space-y-2">
              <Label htmlFor="description">Description</Label>
              <Textarea
                id="description"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="Optional description"
                data-testid="mcp-server-description-input"
              />
              {errors.description && <p className="text-sm text-destructive">{errors.description}</p>}
            </div>

            <div className="flex items-center space-x-2">
              <Switch
                id="enabled"
                checked={enabled}
                onCheckedChange={setEnabled}
                data-testid="mcp-server-enabled-switch"
              />
              <Label htmlFor="enabled">Enabled</Label>
            </div>

            {/* Auth Config Section */}
            <div className="border rounded-lg">
              <button
                type="button"
                className="flex items-center gap-2 w-full p-3 text-left text-sm font-medium hover:bg-muted/50 rounded-lg"
                onClick={() => setShowAuthConfig(!showAuthConfig)}
                data-testid="auth-config-section-toggle"
              >
                {showAuthConfig ? <ChevronDown className="h-4 w-4" /> : <ChevronRight className="h-4 w-4" />}
                Authentication Configuration (Optional)
              </button>

              {showAuthConfig && (
                <div className="px-3 pb-3 space-y-4">
                  <div className="space-y-2">
                    <Label htmlFor="auth-config-type">Auth Type</Label>
                    <Select value={authConfigType} onValueChange={(val) => setAuthConfigType(val as AuthConfigType)}>
                      <SelectTrigger data-testid="auth-config-type-select">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="none">None (Public)</SelectItem>
                        <SelectItem value="header">Header</SelectItem>
                        <SelectItem value="oauth">OAuth</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>

                  {authConfigType !== 'none' && (
                    <AuthConfigForm
                      serverUrl={url}
                      type={authConfigType}
                      name={authName}
                      onTypeChange={(type) => setAuthConfigType(type)}
                      onNameChange={setAuthName}
                      headerKey={headerKey}
                      headerValue={headerValue}
                      onHeaderKeyChange={setHeaderKey}
                      onHeaderValueChange={setHeaderValue}
                      registrationType={oauthRegistrationType}
                      clientId={clientId}
                      clientSecret={clientSecret}
                      authEndpoint={authEndpoint}
                      tokenEndpoint={tokenEndpoint}
                      registrationEndpoint={registrationEndpoint}
                      scopes={scopes}
                      onRegistrationTypeChange={setOauthRegistrationType}
                      onClientIdChange={setClientId}
                      onClientSecretChange={setClientSecret}
                      onAuthEndpointChange={setAuthEndpoint}
                      onTokenEndpointChange={setTokenEndpoint}
                      onRegistrationEndpointChange={setRegistrationEndpoint}
                      onScopesChange={setScopes}
                      enableAutoDcr={true}
                      showTypeSelector={false}
                      showActions={false}
                      onSubmit={() => {
                        /* handled by parent form */
                      }}
                      onCancel={() => setShowAuthConfig(false)}
                      isSubmitting={false}
                    />
                  )}
                </div>
              )}
            </div>

            <div className="flex gap-2 justify-end">
              <Button type="button" variant="outline" onClick={() => router.push('/ui/mcp-servers')}>
                Cancel
              </Button>
              <Button type="submit" disabled={isSaving} data-testid="mcp-server-save-button">
                {isSaving ? 'Saving...' : 'Save'}
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}

export default function NewMcpServerPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <NewMcpServerContent />
    </AppInitializer>
  );
}
