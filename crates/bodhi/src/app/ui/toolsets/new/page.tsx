'use client';

import { useMemo } from 'react';

import { zodResolver } from '@hookform/resolvers/zod';
import { useRouter } from 'next/navigation';
import { useForm } from 'react-hook-form';
import * as z from 'zod';

import AppInitializer from '@/components/AppInitializer';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Skeleton } from '@/components/ui/skeleton';
import { Switch } from '@/components/ui/switch';
import { Textarea } from '@/components/ui/textarea';
import { toast } from '@/hooks/use-toast';
import { useCreateToolset, useToolsets, useToolsetTypes } from '@/hooks/useToolsets';

// Form schema matching the plan specification
const createToolsetSchema = z.object({
  toolset_type: z.string().min(1, 'Type is required'),
  name: z
    .string()
    .min(1, 'Name is required')
    .max(24, 'Name must be 24 characters or less')
    .regex(/^[a-zA-Z0-9-]+$/, 'Name can only contain letters, numbers, and hyphens'),
  description: z.string().max(255).optional(),
  api_key: z.string().min(1, 'API key is required'),
  enabled: z.boolean().default(true),
});

type CreateToolsetFormData = z.infer<typeof createToolsetSchema>;

function NewToolsetPageContent() {
  const router = useRouter();
  const { data: toolsetsData } = useToolsets();
  const { data: typesData, isLoading: typesLoading, error: typesError } = useToolsetTypes();

  const createMutation = useCreateToolset({
    onSuccess: (toolset) => {
      toast({ title: 'Toolset created successfully', description: `Created ${toolset.name}` });
      router.push('/ui/toolsets');
    },
    onError: (message) => {
      toast({ title: 'Failed to create toolset', description: message, variant: 'destructive' });
    },
  });

  const form = useForm<CreateToolsetFormData>({
    resolver: zodResolver(createToolsetSchema),
    defaultValues: {
      toolset_type: '',
      name: '',
      description: '',
      api_key: '',
      enabled: true,
    },
  });

  const toolsets = toolsetsData?.toolsets || [];
  const types = typesData?.types || [];

  // Create scope enabled map from toolset_types
  const scopeEnabledMap = useMemo(() => {
    const map = new Map<string, boolean>();
    const toolsetTypes = toolsetsData?.toolset_types || [];
    toolsetTypes.forEach((config) => map.set(config.toolset_type, config.enabled));
    return map;
  }, [toolsetsData?.toolset_types]);

  const availableTypes = types.filter((type) => scopeEnabledMap.get(type.toolset_type) ?? false);

  // Name prefill logic when type changes
  const handleTypeChange = (toolsetType: string) => {
    form.setValue('toolset_type', toolsetType);

    // Check if user has any toolsets of this type
    const hasToolsetsOfType = toolsets.some((t) => t.toolset_type === toolsetType);
    if (!hasToolsetsOfType) {
      const selectedType = types.find((t) => t.toolset_type === toolsetType);
      if (selectedType) {
        form.setValue('name', selectedType.name);
      }
    }
  };

  const onSubmit = (data: CreateToolsetFormData) => {
    createMutation.mutate({
      name: data.name,
      toolset_type: data.toolset_type,
      description: data.description || undefined,
      api_key: data.api_key,
      enabled: data.enabled,
    });
  };

  if (typesError) {
    const errorMessage =
      typesError.response?.data?.error?.message || typesError.message || 'Failed to load toolset types';
    return <ErrorPage message={errorMessage} />;
  }

  if (typesLoading) {
    return (
      <div className="container mx-auto p-4 max-w-2xl" data-testid="new-toolset-loading">
        <Skeleton className="h-10 w-full mb-4" />
        <Skeleton className="h-64 w-full" />
      </div>
    );
  }

  if (availableTypes.length === 0) {
    return (
      <div className="container mx-auto p-4 max-w-2xl" data-testid="new-toolset-no-types">
        <Card>
          <CardHeader>
            <CardTitle>No Toolset Types Available</CardTitle>
            <CardDescription>
              There are no toolset types available to create. Contact your administrator.
            </CardDescription>
          </CardHeader>
        </Card>
      </div>
    );
  }

  return (
    <div className="container mx-auto p-4 max-w-2xl" data-testid="new-toolset-page">
      <Card>
        <CardHeader>
          <CardTitle>Create New Toolset</CardTitle>
          <CardDescription>Configure a new toolset instance to use in your chats.</CardDescription>
        </CardHeader>
        <CardContent>
          <Form {...form}>
            <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
              <FormField
                control={form.control}
                name="toolset_type"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Toolset Type</FormLabel>
                    <Select
                      onValueChange={handleTypeChange}
                      value={field.value}
                      disabled={createMutation.isLoading}
                      data-testid="toolset-type-select"
                    >
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue placeholder="Select a toolset type" />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        {availableTypes.map((type) => (
                          <SelectItem
                            key={type.toolset_type}
                            value={type.toolset_type}
                            data-testid={`type-option-${type.toolset_type}`}
                          >
                            {type.name}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                    <FormDescription>The type of toolset you want to configure</FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="name"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Name</FormLabel>
                    <FormControl>
                      <Input
                        {...field}
                        placeholder="my-exa-search"
                        disabled={createMutation.isLoading}
                        data-testid="toolset-name-input"
                      />
                    </FormControl>
                    <FormDescription>
                      A unique name for this toolset instance (letters, numbers, and hyphens only)
                    </FormDescription>
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
                        disabled={createMutation.isLoading}
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
                        disabled={createMutation.isLoading}
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
                        disabled={createMutation.isLoading}
                        data-testid="toolset-enabled-switch"
                      />
                    </FormControl>
                  </FormItem>
                )}
              />

              <div className="flex gap-4">
                <Button type="submit" disabled={createMutation.isLoading} data-testid="toolset-create-button">
                  {createMutation.isLoading ? 'Creating...' : 'Create Toolset'}
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => router.push('/ui/toolsets')}
                  disabled={createMutation.isLoading}
                  data-testid="toolset-cancel-button"
                >
                  Cancel
                </Button>
              </div>
            </form>
          </Form>
        </CardContent>
      </Card>
    </div>
  );
}

export default function NewToolsetPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <NewToolsetPageContent />
    </AppInitializer>
  );
}
