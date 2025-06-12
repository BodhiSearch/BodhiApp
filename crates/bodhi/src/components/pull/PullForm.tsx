import React, { useRef } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { pullModelSchema, type PullModelFormData } from '@/schemas/pull';
import { usePullModel, useModelFiles } from '@/hooks/useQuery';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { AutocompleteInput } from '@/components/AutocompleteInput';
import { useToastMessages } from '@/hooks/use-toast-messages';

export function PullForm() {
  const { showSuccess, showError } = useToastMessages();
  const repoInputRef = useRef<HTMLInputElement>(null);
  const filenameInputRef = useRef<HTMLInputElement>(null);

  const form = useForm<PullModelFormData>({
    resolver: zodResolver(pullModelSchema),
    defaultValues: {
      repo: '',
      filename: '',
    },
  });

  const { mutate: pullModel, isLoading } = usePullModel({
    onSuccess: () => {
      showSuccess('Success', 'Model pull request submitted successfully');
      form.reset();
    },
    onError: (message, code) => {
      if (code === 'pull_error-file_already_exists') {
        form.setError('filename', { message });
      } else {
        form.setError('repo', { message });
        form.setError('filename', { message });
      }
      showError('Error', message);
    },
  });

  const { data: modelsData, isLoading: modelsLoading } = useModelFiles(1, 100);

  const repos = Array.from(new Set(modelsData?.data.map((model) => model.repo) || [])).sort();

  const filenames = Array.from(
    new Set(modelsData?.data.filter((model) => model.repo === form.watch('repo')).map((model) => model.filename) || [])
  ).sort();

  const onSubmit = (data: PullModelFormData) => {
    form.clearErrors();
    pullModel(data);
  };

  const handleReset = () => {
    form.reset();
    form.clearErrors();
  };

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-8">
        <Card>
          <CardHeader>
            <CardTitle>Download Model</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <FormField
              control={form.control}
              name="repo"
              render={({ field }) => (
                <FormItem>
                  <FormLabel htmlFor="repo">Repository</FormLabel>
                  <FormControl>
                    <Input {...field} id="repo" ref={repoInputRef} placeholder="Enter repository" />
                  </FormControl>
                  <AutocompleteInput
                    value={field.value}
                    onChange={(value) => field.onChange(value)}
                    suggestions={repos}
                    loading={modelsLoading}
                    inputRef={repoInputRef}
                  />
                  <FormMessage role="alert" />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="filename"
              render={({ field }) => (
                <FormItem>
                  <FormLabel htmlFor="filename">Filename</FormLabel>
                  <FormControl>
                    <Input {...field} id="filename" ref={filenameInputRef} placeholder="Enter filename" />
                  </FormControl>
                  <AutocompleteInput
                    value={field.value}
                    onChange={(value) => field.onChange(value)}
                    suggestions={filenames}
                    loading={modelsLoading}
                    inputRef={filenameInputRef}
                  />
                  <FormMessage role="alert" />
                </FormItem>
              )}
            />

            <div className="flex justify-end gap-2">
              <Button type="button" variant="outline" onClick={handleReset} disabled={isLoading}>
                Reset
              </Button>
              <Button type="submit" disabled={isLoading}>
                {isLoading ? 'Pulling...' : 'Pull Model'}
              </Button>
            </div>
          </CardContent>
        </Card>
      </form>
    </Form>
  );
}
