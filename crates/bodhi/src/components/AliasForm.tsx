'use client';

import React, { useState, useRef, useMemo, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useToast } from '@/hooks/use-toast';
import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';
import { createAliasSchema, AliasFormData } from '@/schemas/alias';
import { Model } from '@/types/models';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { requestParamsSchema, contextParamsSchema } from '@/schemas/alias';
import { z } from 'zod';
import { ChevronDown, ChevronUp } from 'lucide-react';
import {
  useCreateModel,
  useUpdateModel,
  useChatTemplates,
  useModelFiles,
} from '@/hooks/useQuery';
import { AxiosError } from 'axios';
import { AutocompleteInput } from '@/components/AutocompleteInput';

interface AliasFormProps {
  isEditMode: boolean;
  initialData?: Model;
}

const AliasForm: React.FC<AliasFormProps> = ({ isEditMode, initialData }) => {
  const router = useRouter();
  const { toast } = useToast();
  const [isRequestExpanded, setIsRequestExpanded] = useState(
    isEditMode && Object.keys(initialData?.request_params || {}).length > 0
  );
  const [isContextExpanded, setIsContextExpanded] = useState(
    isEditMode && Object.keys(initialData?.context_params || {}).length > 0
  );

  const { data: chatTemplates, isLoading: chatTemplatesLoading } =
    useChatTemplates();
  const { data: modelsData, isLoading: modelsLoading } = useModelFiles(
    1,
    100,
    'alias',
    'asc'
  );

  const repoInputRef = useRef<HTMLInputElement>(null);
  const filenameInputRef = useRef<HTMLInputElement>(null);
  const chatTemplateInputRef = useRef<HTMLInputElement>(null);

  const [currentRepo, setCurrentRepo] = useState(initialData?.repo || '');

  const repos = useMemo(() => {
    if (!modelsData) return [];
    const repoSet = new Set(modelsData.data.map((model) => model.repo));
    return Array.from(repoSet).sort((a, b) =>
      a.toLowerCase().localeCompare(b.toLowerCase())
    );
  }, [modelsData]);

  const filenames = useMemo(() => {
    if (!modelsData || !currentRepo) return [];
    const filenameSet = new Set(
      modelsData.data
        .filter((model) => model.repo === currentRepo)
        .map((model) => model.filename)
    );
    return Array.from(filenameSet).sort((a, b) =>
      a.toLowerCase().localeCompare(b.toLowerCase())
    );
  }, [modelsData, currentRepo]);

  const form = useForm<AliasFormData>({
    resolver: zodResolver(createAliasSchema),
    mode: 'onSubmit',
    defaultValues: {
      alias: initialData?.alias || '',
      repo: initialData?.repo || '',
      filename: initialData?.filename || '',
      chat_template: initialData?.chat_template || '',
      request_params: initialData?.request_params || {},
      context_params: initialData?.context_params || {},
    },
  });

  const createModel = useCreateModel();
  const updateModel = useUpdateModel(initialData?.alias || '');

  useEffect(() => {
    const subscription = form.watch((value, { name }) => {
      if (name === 'repo') {
        setCurrentRepo(value.repo || '');
      }
    });
    return () => subscription.unsubscribe();
  }, [form]);

  const onSubmit = async (data: AliasFormData) => {
    try {
      const mutationFn = isEditMode ? updateModel : createModel;
      await mutationFn.mutateAsync(data);

      toast({
        title: 'Success',
        description: `Alias ${data.alias} successfully ${isEditMode ? 'updated' : 'created'}`,
        duration: 5000,
      });
      router.push('/ui/models');
    } catch (error) {
      console.error(
        `Error ${isEditMode ? 'updating' : 'creating'} alias:`,
        error
      );
      let errorMessage = `Failed to ${isEditMode ? 'update' : 'create'} alias. Please try again.`;

      if (error instanceof AxiosError && error.response?.data?.message) {
        errorMessage = error.response.data.message;
      }

      toast({
        title: 'Error',
        description: errorMessage,
        variant: 'destructive',
      });
    }
  };

  const handleSubmit = form.handleSubmit(
    (data) => {
      const requestErrors = form.formState.errors.request_params;
      const contextErrors = form.formState.errors.context_params;

      if (requestErrors) setIsRequestExpanded(true);
      if (contextErrors) setIsContextExpanded(true);

      if (!requestErrors && !contextErrors) {
        onSubmit(data);
      }
    },
    (errors) => {
      // This function is called when form validation fails
      if (errors.request_params) setIsRequestExpanded(true);
      if (errors.context_params) setIsContextExpanded(true);
    }
  );

  const renderParamFields = (
    paramType: 'request_params' | 'context_params'
  ) => {
    const schema =
      paramType === 'request_params'
        ? requestParamsSchema
        : contextParamsSchema;
    return Object.entries(schema.shape).map(([key, field]) => (
      <FormField
        key={key}
        control={form.control}
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        name={`${paramType}.${key}` as any}
        render={({ field: formField }) => (
          <FormItem>
            <FormLabel>{key}</FormLabel>
            <FormControl>
              {Array.isArray(field) ? (
                <Input
                  {...formField}
                  value={
                    formField.value
                      ? Array.isArray(formField.value)
                        ? formField.value.join(',')
                        : formField.value
                      : ''
                  }
                  onChange={(e) =>
                    formField.onChange(
                      e.target.value
                        ? e.target.value.split(',').map((item) => item.trim())
                        : undefined
                    )
                  }
                  placeholder="Comma-separated values"
                />
              ) : (
                <Input
                  {...formField}
                  type={field instanceof z.ZodNumber ? 'number' : 'text'}
                  min={
                    field instanceof z.ZodNumber
                      ? (field.minValue ?? undefined)
                      : undefined
                  }
                  max={
                    field instanceof z.ZodNumber
                      ? (field.maxValue ?? undefined)
                      : undefined
                  }
                  step={
                    field instanceof z.ZodNumber && !field.isInt
                      ? 0.1
                      : undefined
                  }
                  value={formField.value ?? ''}
                  onChange={(e) => {
                    const inputValue = e.target.value;
                    if (inputValue === '') {
                      formField.onChange(undefined);
                    } else if (field instanceof z.ZodNumber) {
                      const numValue = field.isInt
                        ? parseInt(inputValue, 10)
                        : parseFloat(inputValue);
                      formField.onChange(
                        isNaN(numValue) ? undefined : numValue
                      );
                    } else {
                      formField.onChange(inputValue);
                    }
                  }}
                />
              )}
            </FormControl>
            <FormMessage />
          </FormItem>
        )}
      />
    ));
  };

  return (
    <Form {...form}>
      <form onSubmit={handleSubmit} className="space-y-8 mx-4 my-6">
        <Card>
          <CardHeader>
            <CardTitle>{isEditMode ? 'Edit' : 'New'} Model Alias</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {/* Alias field */}
            <FormField
              control={form.control}
              name="alias"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Alias</FormLabel>
                  <FormControl>
                    <Input {...field} disabled={isEditMode} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            {/* Repo field with AutocompleteInput */}
            <FormField
              control={form.control}
              name="repo"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Repo</FormLabel>
                  <FormControl>
                    <Input
                      {...field}
                      ref={repoInputRef}
                      placeholder="Enter repo"
                    />
                  </FormControl>
                  <AutocompleteInput
                    value={field.value}
                    onChange={(value) => field.onChange(value)}
                    suggestions={repos}
                    loading={modelsLoading}
                    inputRef={repoInputRef}
                  />
                  <FormMessage />
                </FormItem>
              )}
            />

            {/* Filename field with AutocompleteInput */}
            <FormField
              control={form.control}
              name="filename"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Filename</FormLabel>
                  <FormControl>
                    <Input
                      {...field}
                      ref={filenameInputRef}
                      placeholder="Enter filename"
                    />
                  </FormControl>
                  <AutocompleteInput
                    value={field.value}
                    onChange={(value) => field.onChange(value)}
                    suggestions={filenames}
                    loading={modelsLoading}
                    inputRef={filenameInputRef}
                  />
                  <FormMessage />
                </FormItem>
              )}
            />

            {/* Chat Template field with AutocompleteInput */}
            <FormField
              control={form.control}
              name="chat_template"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Chat Template</FormLabel>
                  <FormControl>
                    <Input
                      {...field}
                      ref={chatTemplateInputRef}
                      placeholder="Enter chat template"
                    />
                  </FormControl>
                  <AutocompleteInput
                    value={field.value}
                    onChange={(value) => field.onChange(value)}
                    suggestions={chatTemplates || []}
                    loading={chatTemplatesLoading}
                    inputRef={chatTemplateInputRef}
                  />
                  <FormMessage />
                </FormItem>
              )}
            />
          </CardContent>
        </Card>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="flex flex-col">
            <Card className="h-auto">
              <CardHeader
                className="cursor-pointer flex flex-row items-center justify-between"
                onClick={() => setIsRequestExpanded(!isRequestExpanded)}
              >
                <CardTitle>Request Parameters</CardTitle>
                {isRequestExpanded ? (
                  <ChevronUp size={20} />
                ) : (
                  <ChevronDown size={20} />
                )}
              </CardHeader>
              <CardContent
                className={`overflow-hidden transition-all duration-300 ease-in-out ${isRequestExpanded ? 'max-h-[1000px] opacity-100' : 'max-h-0 opacity-0'}`}
              >
                {renderParamFields('request_params')}
              </CardContent>
            </Card>
          </div>

          <div className="flex flex-col">
            <Card className="h-auto">
              <CardHeader
                className="cursor-pointer flex flex-row items-center justify-between"
                onClick={() => setIsContextExpanded(!isContextExpanded)}
              >
                <CardTitle>Context Parameters</CardTitle>
                {isContextExpanded ? (
                  <ChevronUp size={20} />
                ) : (
                  <ChevronDown size={20} />
                )}
              </CardHeader>
              <CardContent
                className={`overflow-hidden transition-all duration-300 ease-in-out ${isContextExpanded ? 'max-h-[1000px] opacity-100' : 'max-h-0 opacity-0'}`}
              >
                {renderParamFields('context_params')}
              </CardContent>
            </Card>
          </div>
        </div>

        <div className="flex justify-center mt-8">
          <Button type="submit">
            {isEditMode ? 'Update' : 'Create'} Model Alias
          </Button>
        </div>
      </form>
    </Form>
  );
};

export default AliasForm;
