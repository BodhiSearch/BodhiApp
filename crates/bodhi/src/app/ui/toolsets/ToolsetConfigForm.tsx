'use client';

import { useEffect, useState } from 'react';

import { ExternalLink, Info, Loader2 } from 'lucide-react';

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { useToast } from '@/hooks/use-toast';
import {
  useDeleteToolsetConfig,
  useSetAppToolsetDisabled,
  useSetAppToolsetEnabled,
  useToolsetConfig,
  useUpdateToolsetConfig,
} from '@/hooks/useToolsets';
import { useUser } from '@/hooks/useUsers';
import { cn } from '@/lib/utils';

// Toolset metadata - hardcoded for now (can be moved to registry later)
const TOOLSET_METADATA: Record<string, { name: string; description: string; apiKeyUrl: string }> = {
  'builtin-exa-web-search': {
    name: 'Exa Web Search',
    description: 'Search the web using Exa AI for real-time information',
    apiKeyUrl: 'https://exa.ai',
  },
};

interface ToolsetConfigFormProps {
  toolsetId: string;
  onSuccess?: () => void;
}

export function ToolsetConfigForm({ toolsetId, onSuccess }: ToolsetConfigFormProps) {
  const { toast } = useToast();
  const { data: userInfo } = useUser();
  const isAdmin = userInfo?.auth_status === 'logged_in' && userInfo?.role === 'resource_admin';

  // Fetch toolset config
  const { data: toolsetConfig, isLoading: isLoadingConfig, error: configError, refetch } = useToolsetConfig(toolsetId);

  // Treat 404 as "no config exists" - not an error
  const is404Error = configError?.response?.status === 404;

  // Mutations
  const updateConfig = useUpdateToolsetConfig({
    onSuccess: () => {
      toast({
        title: 'Success',
        description: 'Toolset configuration saved',
      });
      setFormState('saved');
      refetch();
      onSuccess?.();
    },
    onError: (message) => {
      toast({
        title: 'Error',
        description: message,
        variant: 'destructive',
      });
      setFormState('idle');
    },
  });

  const deleteConfig = useDeleteToolsetConfig({
    onSuccess: () => {
      toast({
        title: 'Success',
        description: 'API key removed',
      });
      refetch();
    },
    onError: (message) => {
      toast({
        title: 'Error',
        description: message,
        variant: 'destructive',
      });
    },
  });

  const enableAppToolset = useSetAppToolsetEnabled({
    onSuccess: () => {
      toast({
        title: 'Success',
        description: 'Toolset enabled for all users',
      });
      refetch();
    },
    onError: (message) => {
      toast({
        title: 'Error',
        description: message,
        variant: 'destructive',
      });
    },
  });

  const disableAppToolset = useSetAppToolsetDisabled({
    onSuccess: () => {
      toast({
        title: 'Success',
        description: 'Toolset disabled for all users',
      });
      refetch();
    },
    onError: (message) => {
      toast({
        title: 'Error',
        description: message,
        variant: 'destructive',
      });
    },
  });

  // Form state
  const [apiKey, setApiKey] = useState('');
  const [enabled, setEnabled] = useState(false);
  const [formState, setFormState] = useState<'idle' | 'saving' | 'saved'>('idle');

  // Dialog states
  const [showClearDialog, setShowClearDialog] = useState(false);
  const [showAppEnableDialog, setShowAppEnableDialog] = useState(false);
  const [showAppDisableDialog, setShowAppDisableDialog] = useState(false);

  // Sync form state with loaded config
  useEffect(() => {
    if (toolsetConfig?.config) {
      setEnabled(toolsetConfig.config.enabled);
    }
  }, [toolsetConfig]);

  const toolsetMeta = TOOLSET_METADATA[toolsetId] || {
    name: toolsetId,
    description: 'Toolset configuration',
    apiKeyUrl: '#',
  };

  // App-level enabled state from backend
  const isAppEnabled = toolsetConfig?.app_enabled ?? false;

  // Check if user has configured API key - only true if we have actual config data with enabled field
  const hasApiKey = toolsetConfig?.config !== undefined && toolsetConfig.config.enabled !== undefined;

  // Form should be disabled when app-level is disabled
  const isFormDisabled = !isAppEnabled;

  const isSaving = updateConfig.isLoading || deleteConfig.isLoading;
  const isAppToggling = enableAppToolset.isLoading || disableAppToolset.isLoading;

  // Show loading while fetching config
  const shouldShowLoading = isLoadingConfig && !is404Error;

  const handleSave = () => {
    setFormState('saving');
    const request: { toolsetId: string; enabled: boolean; api_key?: string } = {
      toolsetId,
      enabled,
    };

    // Only include API key if user entered something
    if (apiKey.trim()) {
      request.api_key = apiKey;
    }

    updateConfig.mutate(request);
  };

  const handleClearApiKey = () => {
    deleteConfig.mutate({ toolsetId });
    setShowClearDialog(false);
    setApiKey('');
    setEnabled(false);
  };

  const handleAppEnableConfirm = () => {
    enableAppToolset.mutate({ toolsetId });
    setShowAppEnableDialog(false);
  };

  const handleAppDisableConfirm = () => {
    disableAppToolset.mutate({ toolsetId });
    setShowAppDisableDialog(false);
  };

  // Show loading while fetching config
  if (shouldShowLoading) {
    return (
      <Card className="w-full">
        <CardContent className="flex items-center justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
        </CardContent>
      </Card>
    );
  }

  return (
    <TooltipProvider>
      <Card className="w-full" data-testid="toolset-config-form" data-form-state={formState}>
        <CardHeader>
          <CardTitle>{toolsetMeta.name}</CardTitle>
          <CardDescription>{toolsetMeta.description}</CardDescription>
        </CardHeader>

        <CardContent className="space-y-6">
          {/* App Enable/Disable Toggle - Prominent at top for admins */}
          {isAdmin && (
            <div className="space-y-4 pb-4 border-b">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <Label htmlFor="app-enabled" className="text-base font-medium">
                    Enable for Server
                  </Label>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Info className="h-4 w-4 text-muted-foreground cursor-help" />
                    </TooltipTrigger>
                    <TooltipContent>
                      <p>Enable/disable {toolsetMeta.name} toolset for this server</p>
                    </TooltipContent>
                  </Tooltip>
                </div>
                <div className="flex items-center gap-2">
                  {isAppToggling && <Loader2 className="h-4 w-4 animate-spin" />}
                  <Switch
                    id="app-enabled"
                    checked={isAppEnabled}
                    onCheckedChange={(checked) => {
                      if (checked) {
                        setShowAppEnableDialog(true);
                      } else {
                        setShowAppDisableDialog(true);
                      }
                    }}
                    disabled={isAppToggling}
                    data-testid="app-enabled-toggle"
                  />
                  <Badge variant={isAppEnabled ? 'default' : 'secondary'}>
                    {isAppEnabled ? 'Enabled' : 'Disabled'}
                  </Badge>
                </div>
              </div>
            </div>
          )}

          {/* Info message when app is disabled */}
          {!isAppEnabled && (
            <div
              className="text-sm text-muted-foreground bg-muted/50 p-3 rounded-md"
              data-testid="app-disabled-message"
            >
              {isAdmin
                ? 'Enable the toolset for this server to configure it.'
                : 'This toolset is disabled by administrator. Contact your admin to enable it.'}
            </div>
          )}

          {/* User Config Section - Gated on app enabled */}
          <div className={cn('space-y-4', isFormDisabled && 'opacity-50 pointer-events-none')}>
            {/* API Key Input */}
            <div className="space-y-2">
              <Label htmlFor="api-key">API Key</Label>
              <div className="flex gap-2">
                <Input
                  id="api-key"
                  type="password"
                  placeholder={hasApiKey ? '••••••••••••••••' : 'Enter your API key'}
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
                  disabled={isFormDisabled || isSaving}
                  data-testid="toolset-api-key-input"
                />
                {hasApiKey && (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setShowClearDialog(true)}
                    disabled={isFormDisabled || isSaving}
                    data-testid="clear-api-key-button"
                  >
                    Clear
                  </Button>
                )}
              </div>
              <div className="flex flex-col gap-1">
                <p className="text-sm text-muted-foreground">
                  Get your API key from{' '}
                  <a
                    href={toolsetMeta.apiKeyUrl}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-primary hover:underline inline-flex items-center gap-1"
                  >
                    {toolsetMeta.apiKeyUrl.replace('https://', '')}
                    <ExternalLink className="h-3 w-3" />
                  </a>
                </p>
                <p className="text-xs text-muted-foreground">Each user must configure their own API key</p>
              </div>
            </div>

            {/* Enable Toggle */}
            <div className="flex items-center justify-between">
              <Label htmlFor="enabled">Enable Toolset</Label>
              <Switch
                id="enabled"
                checked={enabled}
                onCheckedChange={setEnabled}
                disabled={isFormDisabled || isSaving || (!hasApiKey && !apiKey.trim())}
                data-testid="toolset-enabled-toggle"
              />
            </div>
          </div>
        </CardContent>

        <CardFooter>
          <Button
            onClick={handleSave}
            disabled={isFormDisabled || isSaving || !apiKey.trim()}
            data-testid="save-toolset-config"
          >
            {isSaving ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Saving...
              </>
            ) : (
              'Save'
            )}
          </Button>
        </CardFooter>
      </Card>

      {/* Clear API Key Confirmation Dialog */}
      <AlertDialog open={showClearDialog} onOpenChange={setShowClearDialog}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Clear API Key</AlertDialogTitle>
            <AlertDialogDescription>
              This will remove your API key and disable the toolset. You will need to configure it again to use this
              toolset.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleClearApiKey}>Clear API Key</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* App Enable Confirmation Dialog */}
      <AlertDialog open={showAppEnableDialog} onOpenChange={setShowAppEnableDialog}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Enable Toolset for Server</AlertDialogTitle>
            <AlertDialogDescription>
              This will enable {toolsetMeta.name} for all users on this server. Users will still need to configure their
              own API keys to use the toolset.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleAppEnableConfirm}>Enable</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* App Disable Confirmation Dialog */}
      <AlertDialog open={showAppDisableDialog} onOpenChange={setShowAppDisableDialog}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Disable Toolset for Server</AlertDialogTitle>
            <AlertDialogDescription>
              This will disable {toolsetMeta.name} for all users on this server. Users will not be able to use this
              toolset until it is re-enabled.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleAppDisableConfirm}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              Disable
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </TooltipProvider>
  );
}
