'use client';

import { useEffect, useMemo, useState } from 'react';

import { zodResolver } from '@hookform/resolvers/zod';
import { Trash2 } from 'lucide-react';
import { useRouter, useSearchParams } from 'next/navigation';
import { useForm } from 'react-hook-form';
import * as z from 'zod';

import AppInitializer from '@/components/AppInitializer';
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
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { PasswordInput } from '@/components/ui/password-input';
import { Skeleton } from '@/components/ui/skeleton';
import { Switch } from '@/components/ui/switch';
import { Textarea } from '@/components/ui/textarea';
import { toast } from '@/hooks/use-toast';
import { useDeleteToolset, useToolset, useToolsets, useUpdateToolset } from '@/hooks/useToolsets';

// Form schema matching the plan specification
const updateToolsetSchema = z.object({
  slug: z
    .string()
    .min(1, 'Slug is required')
    .max(24, 'Slug must be 24 characters or less')
    .regex(/^[a-zA-Z0-9-]+$/, 'Slug can only contain letters, numbers, and hyphens'),
  description: z.string().max(255).optional().nullable(),
  enabled: z.boolean(),
  api_key: z.union([
    z.literal(''), // Keep existing
    z.string().min(1), // New value
  ]),
});

type UpdateToolsetFormData = z.infer<typeof updateToolsetSchema>;

function EditToolsetContent() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const id = searchParams?.get('id');

  const { data: toolset, isLoading, error } = useToolset(id || '', { enabled: !!id });
  const { data: toolsetsResponse } = useToolsets();
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);

  // Create scope enabled map from toolset_types
  const isAdminEnabled = useMemo(() => {
    if (!toolset || !toolsetsResponse?.toolset_types) return true;
    const scopeEnabledMap = new Map<string, boolean>();
    toolsetsResponse.toolset_types.forEach((config) => scopeEnabledMap.set(config.toolset_type, config.enabled));
    return scopeEnabledMap.get(toolset.toolset_type) ?? true;
  }, [toolset, toolsetsResponse?.toolset_types]);

  const updateMutation = useUpdateToolset({
    onSuccess: (updated) => {
      toast({ title: 'Toolset updated successfully', description: `Updated ${updated.slug}` });
    },
    onError: (message) => {
      toast({ title: 'Failed to update toolset', description: message, variant: 'destructive' });
    },
  });

  const deleteMutation = useDeleteToolset({
    onSuccess: () => {
      toast({ title: 'Toolset deleted successfully' });
      router.push('/ui/toolsets');
    },
    onError: (message) => {
      toast({ title: 'Failed to delete toolset', description: message, variant: 'destructive' });
    },
  });

  const form = useForm<UpdateToolsetFormData>({
    resolver: zodResolver(updateToolsetSchema),
    defaultValues: {
      slug: '',
      description: '',
      enabled: true,
      api_key: '',
    },
  });

  // Populate form when toolset data loads
  useEffect(() => {
    if (toolset) {
      form.reset({
        slug: toolset.slug,
        description: toolset.description || '',
        enabled: toolset.enabled,
        api_key: '', // Always start empty (keep existing)
      });
    }
  }, [toolset, form]);

  // Redirect if app disabled
  useEffect(() => {
    if (toolset && !isAdminEnabled) {
      toast({
        title: 'Toolset disabled',
        description: 'This toolset has been disabled by an administrator.',
        variant: 'destructive',
      });
      router.push('/ui/toolsets');
    }
  }, [toolset, isAdminEnabled, router]);

  const onSubmit = (data: UpdateToolsetFormData) => {
    if (!id) return;

    updateMutation.mutate({
      id,
      slug: data.slug,
      description: data.description || null,
      enabled: data.enabled,
      api_key: data.api_key === '' ? { action: 'Keep' } : { action: 'Set', value: data.api_key },
    });
  };

  const handleDelete = () => {
    if (!id) return;
    deleteMutation.mutate({ id });
  };

  if (!id) {
    return <ErrorPage title="Not Found" message="Toolset ID is required" />;
  }

  if (error) {
    const errorMessage = error.response?.data?.error?.message || error.message || 'Failed to load toolset';
    return <ErrorPage message={errorMessage} />;
  }

  if (isLoading || !toolset) {
    return (
      <div className="container mx-auto p-4 max-w-2xl" data-testid="edit-toolset-loading">
        <Skeleton className="h-10 w-full mb-4" />
        <Skeleton className="h-64 w-full" />
      </div>
    );
  }

  return (
    <div className="container mx-auto p-4 max-w-2xl" data-testid="edit-toolset-page">
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>Edit Toolset</CardTitle>
              <CardDescription>Update the configuration for {toolset.slug}.</CardDescription>
            </div>
            <Button
              variant="destructive"
              size="sm"
              onClick={() => setDeleteDialogOpen(true)}
              disabled={deleteMutation.isLoading}
              data-testid="toolset-delete-button"
            >
              <Trash2 className="h-4 w-4 mr-2" />
              Delete
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          <Form {...form}>
            <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
              <FormItem>
                <FormLabel>Toolset Type</FormLabel>
                <FormControl>
                  <Input value={toolset.toolset_type} disabled data-testid="toolset-type-display" />
                </FormControl>
                <FormDescription>The type of toolset (read-only)</FormDescription>
              </FormItem>

              <FormField
                control={form.control}
                name="slug"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Slug</FormLabel>
                    <FormControl>
                      <Input
                        {...field}
                        placeholder="my-exa-search"
                        disabled={updateMutation.isLoading}
                        data-testid="toolset-slug-input"
                      />
                    </FormControl>
                    <FormDescription>
                      A unique slug for this toolset instance (letters, numbers, and hyphens only)
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
                        value={field.value || ''}
                        placeholder="Describe what this toolset is used for"
                        disabled={updateMutation.isLoading}
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
                      <PasswordInput
                        {...field}
                        placeholder={toolset.has_api_key ? 'Leave empty to keep current key' : 'Enter API key'}
                        disabled={updateMutation.isLoading}
                        data-testid="toolset-api-key-input"
                      />
                    </FormControl>
                    <FormDescription>
                      {toolset.has_api_key
                        ? 'Leave empty to keep the existing API key, or enter a new one to replace it'
                        : 'Enter an API key to configure this toolset'}
                    </FormDescription>
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
                        disabled={updateMutation.isLoading}
                        data-testid="toolset-enabled-switch"
                      />
                    </FormControl>
                  </FormItem>
                )}
              />
            </form>
          </Form>
        </CardContent>
        <CardFooter className="flex gap-4">
          <Button
            onClick={form.handleSubmit(onSubmit)}
            disabled={updateMutation.isLoading}
            data-testid="toolset-save-button"
          >
            {updateMutation.isLoading ? 'Saving...' : 'Save Changes'}
          </Button>
          <Button
            type="button"
            variant="outline"
            onClick={() => router.push('/ui/toolsets')}
            disabled={updateMutation.isLoading}
            data-testid="toolset-cancel-button"
          >
            Cancel
          </Button>
        </CardFooter>
      </Card>

      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Toolset</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete &quot;{toolset.slug}&quot;? This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleDelete} disabled={deleteMutation.isLoading}>
              {deleteMutation.isLoading ? 'Deleting...' : 'Delete'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

export default function EditToolsetPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <EditToolsetContent />
    </AppInitializer>
  );
}
