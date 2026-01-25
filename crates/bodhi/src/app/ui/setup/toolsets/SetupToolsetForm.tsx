'use client';

import { useEffect, useMemo, useState } from 'react';

import { zodResolver } from '@hookform/resolvers/zod';
import { Info } from 'lucide-react';
import { useForm } from 'react-hook-form';
import * as z from 'zod';

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
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Skeleton } from '@/components/ui/skeleton';
import { Switch } from '@/components/ui/switch';
import { Textarea } from '@/components/ui/textarea';
import { toast } from '@/hooks/use-toast';
import {
  useCreateToolset,
  useDisableToolsetType,
  useEnableToolsetType,
  useToolsets,
  useToolsetTypes,
} from '@/hooks/useToolsets';

const TOOLSET_SCOPE = 'scope_toolset-builtin-exa-web-search';

const createToolsetSchema = z.object({
  name: z
    .string()
    .min(1, 'Name is required')
    .max(24, 'Name must be 24 characters or less')
    .regex(/^[a-zA-Z0-9-]+$/, 'Only letters, numbers, and hyphens'),
  description: z.string().max(255).optional(),
  api_key: z.string().min(1, 'API key is required'),
  enabled: z.boolean().default(true),
});

type CreateToolsetFormData = z.infer<typeof createToolsetSchema>;

interface SetupToolsetFormProps {
  onSuccess?: () => void;
}

export function SetupToolsetForm({ onSuccess }: SetupToolsetFormProps) {
  const { data: typesData, isLoading: typesLoading } = useToolsetTypes();
  const { data: toolsetsData } = useToolsets();
  const [enableDialogOpen, setEnableDialogOpen] = useState(false);
  const [disableDialogOpen, setDisableDialogOpen] = useState(false);

  const toolsetType = typesData?.types?.find((t) => t.scope === TOOLSET_SCOPE);

  // Check if admin enabled using toolset_types
  const isAppEnabled = useMemo(() => {
    if (!toolsetType || !toolsetsData?.toolset_types) return false;
    const scopeEnabledMap = new Map<string, boolean>();
    toolsetsData.toolset_types.forEach((config) => scopeEnabledMap.set(config.scope, config.enabled));
    return scopeEnabledMap.get(toolsetType.scope) ?? false;
  }, [toolsetType, toolsetsData?.toolset_types]);

  const enableMutation = useEnableToolsetType({
    onSuccess: () => {
      toast({ title: 'Success', description: 'Toolset enabled for server' });
      setEnableDialogOpen(false);
    },
    onError: (message) => {
      toast({ title: 'Error', description: message, variant: 'destructive' });
      setEnableDialogOpen(false);
    },
  });

  const disableMutation = useDisableToolsetType({
    onSuccess: () => {
      toast({ title: 'Success', description: 'Toolset disabled for server' });
      setDisableDialogOpen(false);
    },
    onError: (message) => {
      toast({ title: 'Error', description: message, variant: 'destructive' });
      setDisableDialogOpen(false);
    },
  });

  const createMutation = useCreateToolset({
    onSuccess: (toolset) => {
      toast({ title: 'Success', description: `Created ${toolset.name}` });
      onSuccess?.();
    },
    onError: (message) => {
      toast({ title: 'Error', description: message, variant: 'destructive' });
    },
  });

  const form = useForm<CreateToolsetFormData>({
    resolver: zodResolver(createToolsetSchema),
    defaultValues: {
      name: '',
      description: '',
      api_key: '',
      enabled: true,
    },
  });

  // Update form name when toolsetType loads - derive from scope
  useEffect(() => {
    if (toolsetType?.scope) {
      const derivedName = toolsetType.scope.replace(/^scope_toolset-/, '');
      form.setValue('name', derivedName);
    }
  }, [toolsetType?.scope, form]);

  const handleToggleClick = (checked: boolean) => {
    if (checked) {
      setEnableDialogOpen(true);
    } else {
      setDisableDialogOpen(true);
    }
  };

  const handleEnableConfirm = () => {
    if (toolsetType) {
      enableMutation.mutate({ scope: toolsetType.scope });
    }
  };

  const handleDisableConfirm = () => {
    if (toolsetType) {
      disableMutation.mutate({ scope: toolsetType.scope });
    }
  };

  const onSubmit = (data: CreateToolsetFormData) => {
    if (!toolsetType) return;
    createMutation.mutate({
      scope_uuid: toolsetType.scope_uuid,
      name: data.name,
      description: data.description || undefined,
      api_key: data.api_key,
      enabled: data.enabled,
    });
  };

  if (typesLoading) {
    return (
      <Card className="w-full" data-testid="setup-toolset-form">
        <CardHeader className="text-center">
          <CardTitle>Configure Toolsets</CardTitle>
          <CardDescription>Enhance your AI with web search capabilities</CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <Skeleton className="h-24 w-full" />
          <Skeleton className="h-64 w-full" />
        </CardContent>
      </Card>
    );
  }

  const isToggling = enableMutation.isLoading || disableMutation.isLoading;
  const isCreating = createMutation.isLoading;
  const isFormDisabled = !isAppEnabled || isToggling || isCreating;

  return (
    <>
      <Card className="w-full" data-testid="setup-toolset-form">
        <CardHeader className="text-center">
          <CardTitle>Configure Toolsets</CardTitle>
          <CardDescription>Enhance your AI with web search capabilities</CardDescription>
        </CardHeader>

        <CardContent className="space-y-6">
          {/* Toolset Type Toggle Section */}
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <div>
                  <CardTitle className="text-lg">{toolsetType?.name || 'Web Search'}</CardTitle>
                  <CardDescription>{toolsetType?.description || 'Configure web search capabilities'}</CardDescription>
                </div>
                <div className="flex items-center gap-2">
                  <Switch
                    checked={isAppEnabled}
                    onCheckedChange={handleToggleClick}
                    disabled={isToggling}
                    data-testid="app-enabled-toggle"
                  />
                  <span className="text-sm text-muted-foreground">{isAppEnabled ? 'Enabled' : 'Disabled'}</span>
                </div>
              </div>
            </CardHeader>
          </Card>

          {/* Create Toolset Form */}
          <Card className={!isAppEnabled ? 'opacity-60' : ''}>
            <CardHeader>
              <CardTitle className="text-lg">Create Toolset</CardTitle>
              <CardDescription
                data-testid="app-disabled-message"
                data-test-state={isAppEnabled ? 'enabled' : 'disabled'}
              >
                {isAppEnabled
                  ? `Configure your first ${toolsetType?.name || 'toolset'} instance`
                  : 'Enable the toolset type above to create a toolset'}
              </CardDescription>
            </CardHeader>
            <CardContent>
              <Form {...form}>
                <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
                  <FormField
                    control={form.control}
                    name="name"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Name</FormLabel>
                        <FormControl>
                          <Input
                            {...field}
                            placeholder={toolsetType?.scope.replace(/^scope_toolset-/, '') || 'my-toolset'}
                            disabled={isFormDisabled}
                            data-testid="toolset-name-input"
                          />
                        </FormControl>
                        <FormDescription>A unique name for this toolset</FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="description"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Description (Optional)</FormLabel>
                        <FormControl>
                          <Textarea
                            {...field}
                            placeholder="Describe what this toolset is used for"
                            disabled={isFormDisabled}
                            data-testid="toolset-description-input"
                          />
                        </FormControl>
                        <FormDescription>Optional description for this toolset</FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="api_key"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>API Key</FormLabel>
                        <FormControl>
                          <Input
                            {...field}
                            type="password"
                            placeholder="Enter API key"
                            disabled={isFormDisabled}
                            data-testid="toolset-api-key-input"
                          />
                        </FormControl>
                        <FormDescription>The API key for this toolset</FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="enabled"
                    render={({ field }) => (
                      <FormItem className="flex flex-row items-center justify-between rounded-lg border p-4">
                        <div className="space-y-0.5">
                          <FormLabel className="text-base">Enable Toolset</FormLabel>
                          <FormDescription>Make this toolset available for use in chats</FormDescription>
                        </div>
                        <FormControl>
                          <Switch
                            checked={field.value}
                            onCheckedChange={field.onChange}
                            disabled={isFormDisabled}
                            data-testid="toolset-enabled-toggle"
                          />
                        </FormControl>
                      </FormItem>
                    )}
                  />

                  <Button type="submit" disabled={isFormDisabled} data-testid="create-toolset-button">
                    {isCreating ? 'Creating...' : 'Create Toolset'}
                  </Button>
                </form>
              </Form>
            </CardContent>
          </Card>

          {/* Info Box - Only show for Exa */}
          {toolsetType?.scope === TOOLSET_SCOPE && (
            <Card className="bg-muted/50">
              <CardContent className="pt-6">
                <div className="flex items-start gap-3">
                  <Info className="h-5 w-5 text-muted-foreground mt-0.5" />
                  <div className="space-y-1">
                    <p className="text-sm font-medium">Don&apos;t have an Exa API key?</p>
                    <p className="text-sm text-muted-foreground">
                      Get one at{' '}
                      <a href="https://exa.ai" target="_blank" rel="noopener noreferrer" className="underline">
                        exa.ai
                      </a>
                    </p>
                  </div>
                </div>
              </CardContent>
            </Card>
          )}
        </CardContent>

        <CardFooter className="flex justify-center">
          <p className="text-sm text-muted-foreground">You can always skip this step and configure later</p>
        </CardFooter>
      </Card>

      {/* Enable Confirmation Dialog */}
      <AlertDialog open={enableDialogOpen} onOpenChange={setEnableDialogOpen}>
        <AlertDialogContent data-testid="enable-confirm-dialog">
          <AlertDialogHeader>
            <AlertDialogTitle>Enable Toolset for Server</AlertDialogTitle>
            <AlertDialogDescription>
              This will enable {toolsetType?.name || 'this toolset'} for all users on this server.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleEnableConfirm} disabled={enableMutation.isLoading}>
              {enableMutation.isLoading ? 'Enabling...' : 'Enable'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Disable Confirmation Dialog */}
      <AlertDialog open={disableDialogOpen} onOpenChange={setDisableDialogOpen}>
        <AlertDialogContent data-testid="disable-confirm-dialog">
          <AlertDialogHeader>
            <AlertDialogTitle>Disable Toolset for Server</AlertDialogTitle>
            <AlertDialogDescription>
              This will disable {toolsetType?.name || 'this toolset'} for all users. Existing instances will stop
              working.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleDisableConfirm} disabled={disableMutation.isLoading}>
              {disableMutation.isLoading ? 'Disabling...' : 'Disable'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
