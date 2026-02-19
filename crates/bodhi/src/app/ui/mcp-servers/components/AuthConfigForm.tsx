'use client';

import { useEffect, useState } from 'react';

import { Loader2 } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { useDiscoverMcp } from '@/hooks/useMcps';

type AuthConfigType = 'header' | 'oauth';
type OAuthRegistrationType = 'pre-registered' | 'dynamic-registration';

interface AuthConfigFormProps {
  // Server context
  serverUrl: string;

  // Form state
  type: AuthConfigType;
  name: string;
  onTypeChange: (type: AuthConfigType) => void;
  onNameChange: (name: string) => void;

  // UI control
  showTypeSelector?: boolean;
  showActions?: boolean;

  // Header auth state
  headerKey: string;
  headerValue: string;
  onHeaderKeyChange: (value: string) => void;
  onHeaderValueChange: (value: string) => void;

  // OAuth state
  registrationType: OAuthRegistrationType;
  clientId: string;
  clientSecret: string;
  authEndpoint: string;
  tokenEndpoint: string;
  registrationEndpoint: string;
  scopes: string;
  onRegistrationTypeChange: (type: OAuthRegistrationType) => void;
  onClientIdChange: (value: string) => void;
  onClientSecretChange: (value: string) => void;
  onAuthEndpointChange: (value: string) => void;
  onTokenEndpointChange: (value: string) => void;
  onRegistrationEndpointChange: (value: string) => void;
  onScopesChange: (value: string) => void;

  // Auto-DCR control (optional - for new page only)
  enableAutoDcr?: boolean;

  // Actions
  onSubmit: () => void;
  onCancel: () => void;
  isSubmitting: boolean;
}

export function AuthConfigForm(props: AuthConfigFormProps) {
  const [isDiscovering, setIsDiscovering] = useState(false);
  const [discoverError, setDiscoverError] = useState('');
  const [hasAttemptedAutoDcr, setHasAttemptedAutoDcr] = useState(false);
  const [autoDcrFailed, setAutoDcrFailed] = useState(false);

  const discoverMcp = useDiscoverMcp({
    onSuccess: (data) => {
      setIsDiscovering(false);
      setDiscoverError('');
      setAutoDcrFailed(false);

      // Ensure registration type is set to dynamic-registration after successful discovery
      if (props.enableAutoDcr && data.registration_endpoint) {
        props.onRegistrationTypeChange('dynamic-registration');
      }

      if (data.authorization_endpoint) props.onAuthEndpointChange(data.authorization_endpoint);
      if (data.token_endpoint) props.onTokenEndpointChange(data.token_endpoint);
      if (data.registration_endpoint) props.onRegistrationEndpointChange(data.registration_endpoint);
      if (data.scopes_supported) props.onScopesChange(data.scopes_supported.join(' '));
    },
    onError: (message) => {
      setIsDiscovering(false);

      if (props.enableAutoDcr && !autoDcrFailed) {
        // First auto-DCR failure - silent switch
        props.onRegistrationTypeChange('pre-registered');
        setAutoDcrFailed(true);
        setDiscoverError('');
      } else {
        // Show error
        setDiscoverError(message);
      }
    },
  });

  // Auto-populate config name based on type
  useEffect(() => {
    if (props.type === 'header' && (!props.name || props.name === 'oauth-default')) {
      props.onNameChange('header-default');
    } else if (props.type === 'oauth' && (!props.name || props.name === 'header-default')) {
      props.onNameChange('oauth-default');
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [props.type]);

  // Auto-DCR on first OAuth selection (new page only)
  useEffect(() => {
    if (props.enableAutoDcr && props.type === 'oauth' && !hasAttemptedAutoDcr && props.serverUrl) {
      props.onRegistrationTypeChange('dynamic-registration');
      setHasAttemptedAutoDcr(true);
      setIsDiscovering(true);
      discoverMcp.mutate({ mcp_server_url: props.serverUrl });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [props.type, hasAttemptedAutoDcr, props.serverUrl, props.enableAutoDcr]);

  // Auto-discover on OAuth type selection (view page only)
  useEffect(() => {
    if (!props.enableAutoDcr && props.type === 'oauth' && props.serverUrl) {
      setIsDiscovering(true);
      discoverMcp.mutate({ mcp_server_url: props.serverUrl });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [props.type, props.serverUrl, props.enableAutoDcr]);

  // Manual retry after auto-fail
  useEffect(() => {
    if (
      props.enableAutoDcr &&
      props.type === 'oauth' &&
      props.registrationType === 'dynamic-registration' &&
      autoDcrFailed &&
      props.serverUrl
    ) {
      setIsDiscovering(true);
      setDiscoverError('');
      discoverMcp.mutate({ mcp_server_url: props.serverUrl });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [props.registrationType, props.type, autoDcrFailed, props.serverUrl, props.enableAutoDcr]);

  return (
    <div className="space-y-4">
      {/* Type selector - only show if not handled by parent */}
      {(props.showTypeSelector ?? true) && (
        <div className="space-y-2">
          <Label>Type</Label>
          <Select value={props.type} onValueChange={(val) => props.onTypeChange(val as AuthConfigType)}>
            <SelectTrigger data-testid="auth-config-type-select">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="header">Header</SelectItem>
              <SelectItem value="oauth">OAuth</SelectItem>
            </SelectContent>
          </Select>
        </div>
      )}

      {/* Name input */}
      <div className="space-y-2">
        <Label>Name</Label>
        <Input
          value={props.name}
          onChange={(e) => props.onNameChange(e.target.value)}
          placeholder="e.g. My Auth Config"
          data-testid="auth-config-name-input"
        />
      </div>

      {/* Header fields */}
      {props.type === 'header' && (
        <>
          <div className="space-y-2">
            <Label>Header Key</Label>
            <Input
              value={props.headerKey}
              onChange={(e) => props.onHeaderKeyChange(e.target.value)}
              placeholder="e.g. Authorization"
              data-testid="auth-config-header-key-input"
            />
          </div>
          <div className="space-y-2">
            <Label>Header Value</Label>
            <Input
              type="password"
              value={props.headerValue}
              onChange={(e) => props.onHeaderValueChange(e.target.value)}
              placeholder="e.g. Bearer sk-..."
              data-testid="auth-config-header-value-input"
            />
          </div>
        </>
      )}

      {/* OAuth fields */}
      {props.type === 'oauth' && (
        <>
          {/* Registration Type (only for new page with enableAutoDcr) */}
          {props.enableAutoDcr && (
            <div className="space-y-2">
              <Label>Registration Type</Label>
              <Select
                value={props.registrationType}
                onValueChange={(val) => props.onRegistrationTypeChange(val as OAuthRegistrationType)}
              >
                <SelectTrigger data-testid="oauth-registration-type-select">
                  <SelectValue placeholder="Select registration type" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="pre-registered">Pre-Registered</SelectItem>
                  <SelectItem value="dynamic-registration">Dynamic Registration</SelectItem>
                </SelectContent>
              </Select>
            </div>
          )}

          {/* Discovery status */}
          {isDiscovering && (
            <div
              className="flex items-center gap-2 text-sm text-muted-foreground"
              data-testid="auth-config-discover-status"
            >
              <Loader2 className="h-4 w-4 animate-spin" />
              Discovering OAuth endpoints...
            </div>
          )}

          {/* Discovery error */}
          {discoverError && (
            <div data-testid="auth-config-discover-error">
              <p className="text-sm text-destructive">{discoverError}</p>
              {autoDcrFailed && (
                <button
                  type="button"
                  className="text-sm text-primary underline mt-1"
                  onClick={() => props.onRegistrationTypeChange('pre-registered')}
                  data-testid="auth-config-switch-to-prereg"
                >
                  Switch to Pre-Registered (manual entry)
                </button>
              )}
            </div>
          )}

          {/* Pre-registered fields */}
          {(!props.enableAutoDcr || props.registrationType === 'pre-registered') && (
            <>
              <div className="space-y-2">
                <Label>Client ID</Label>
                <Input
                  value={props.clientId}
                  onChange={(e) => props.onClientIdChange(e.target.value)}
                  placeholder="Client ID"
                  data-testid="auth-config-client-id-input"
                />
              </div>
              <div className="space-y-2">
                <Label>Client Secret (Optional)</Label>
                <Input
                  type="password"
                  value={props.clientSecret}
                  onChange={(e) => props.onClientSecretChange(e.target.value)}
                  placeholder="Client Secret"
                  data-testid="auth-config-client-secret-input"
                />
              </div>
            </>
          )}

          {/* Shared OAuth fields */}
          <div className="space-y-2">
            <Label>Authorization Endpoint</Label>
            <Input
              value={props.authEndpoint}
              onChange={(e) => props.onAuthEndpointChange(e.target.value)}
              placeholder="https://auth.example.com/authorize"
              data-testid="auth-config-auth-endpoint-input"
            />
          </div>
          <div className="space-y-2">
            <Label>Token Endpoint</Label>
            <Input
              value={props.tokenEndpoint}
              onChange={(e) => props.onTokenEndpointChange(e.target.value)}
              placeholder="https://auth.example.com/token"
              data-testid="auth-config-token-endpoint-input"
            />
          </div>

          {/* Dynamic registration endpoint */}
          {(!props.enableAutoDcr || props.registrationType === 'dynamic-registration') && (
            <div className="space-y-2">
              <Label>Registration Endpoint</Label>
              <Input
                value={props.registrationEndpoint}
                onChange={(e) => props.onRegistrationEndpointChange(e.target.value)}
                placeholder="https://auth.example.com/register"
                data-testid="auth-config-registration-endpoint-input"
              />
            </div>
          )}

          {/* Scopes */}
          <div className="space-y-2">
            <Label>Scopes (Optional)</Label>
            <Input
              value={props.scopes}
              onChange={(e) => props.onScopesChange(e.target.value)}
              placeholder="e.g. mcp:tools mcp:read"
              data-testid="auth-config-scopes-input"
            />
          </div>
        </>
      )}

      {/* Actions */}
      {(props.showActions ?? true) && (
        <div className="flex gap-2">
          <Button
            size="sm"
            onClick={props.onSubmit}
            disabled={props.isSubmitting || !props.name}
            data-testid="auth-config-save-button"
          >
            {props.isSubmitting ? 'Saving...' : 'Save'}
          </Button>
          <Button size="sm" variant="outline" onClick={props.onCancel} data-testid="auth-config-cancel-button">
            Cancel
          </Button>
        </div>
      )}
    </div>
  );
}
