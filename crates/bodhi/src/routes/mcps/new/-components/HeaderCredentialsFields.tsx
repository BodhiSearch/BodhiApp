import { Eye, EyeOff } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import type { McpAuthConfigResponse } from '@/hooks/mcps';

type HeaderConfig = Extract<McpAuthConfigResponse, { type: 'header' }>;

export function HeaderCredentialsFields({
  configName,
  config,
  credentialValues,
  onCredentialChange,
  visibleCredentials,
  onToggleVisibility,
  isSubmitting,
}: {
  configName: string;
  config: HeaderConfig;
  credentialValues: Record<string, string>;
  onCredentialChange: (key: string, value: string) => void;
  visibleCredentials: Set<string>;
  onToggleVisibility: (key: string) => void;
  isSubmitting: boolean;
}) {
  return (
    <div className="rounded-lg border p-3 text-sm space-y-3 bg-muted/50" data-testid="auth-config-header-credentials">
      <p>
        <span className="font-medium">Config:</span> {configName}
      </p>
      {config.entries.length > 0 ? (
        config.entries.map((entry) => {
          const isVisible = visibleCredentials.has(entry.param_key);
          return (
            <div key={entry.param_key} className="space-y-1" data-testid={`credential-field-${entry.param_key}`}>
              <label className="text-xs font-medium text-muted-foreground">
                {entry.param_key} <span className="text-xs opacity-60">({entry.param_type})</span>
              </label>
              <div className="flex items-center gap-1">
                <Input
                  type={isVisible ? 'text' : 'password'}
                  placeholder={`Enter ${entry.param_key} value`}
                  value={credentialValues[entry.param_key] || ''}
                  onChange={(e) => onCredentialChange(entry.param_key, e.target.value)}
                  disabled={isSubmitting}
                  data-testid={`credential-input-${entry.param_key}`}
                  className="flex-1"
                />
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  className="h-9 w-9 shrink-0"
                  onClick={() => onToggleVisibility(entry.param_key)}
                  data-testid={`credential-toggle-${entry.param_key}`}
                >
                  {isVisible ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                </Button>
              </div>
            </div>
          );
        })
      ) : (
        <p className="text-muted-foreground">No credential keys defined for this config.</p>
      )}
    </div>
  );
}

export default HeaderCredentialsFields;
