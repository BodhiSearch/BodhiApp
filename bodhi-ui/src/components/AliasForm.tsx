'use client';

import React, { useMemo, useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useToast } from '@/hooks/use-toast';
import { useQuery } from 'react-query';
import axios, { AxiosError } from 'axios';
import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';
import { createAliasSchema, AliasFormData } from '@/schemas/alias';
import { ModelsResponse, Model } from '@/types/models';
import { Input } from '@/components/ui/input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
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

interface AliasFormProps {
  isEditMode: boolean;
  initialData?: Model;
}

const AliasForm: React.FC<AliasFormProps> = ({ isEditMode, initialData }) => {
  const router = useRouter();
  const { toast } = useToast();
  const [isRequestExpanded, setIsRequestExpanded] = useState(false);
  const [isContextExpanded, setIsContextExpanded] = useState(false);
  const [initialDataLoaded, setInitialDataLoaded] = useState(false);

  const form = useForm<AliasFormData>({
    resolver: zodResolver(createAliasSchema),
    mode: 'onSubmit',
    defaultValues: {
      alias: '',
      repo: '',
      filename: '',
      chat_template: '',
      request_params: undefined,
      context_params: undefined,
    },
  });

  const { data: modelsData, isLoading: isModelsLoading } =
    useQuery<ModelsResponse>(
      'models',
      async () => {
        const response = await axios.get('/api/ui/models');
        return response.data;
      },
      {
        refetchOnMount: true,
        refetchOnWindowFocus: false,
      }
    );

  const { data: chatTemplates, isLoading: isTemplatesLoading } = useQuery<
    string[]
  >(
    'chatTemplates',
    async () => {
      const response = await axios.get('/api/ui/chat_templates');
      return response.data;
    },
    {
      refetchOnMount: true,
      refetchOnWindowFocus: false,
    }
  );

  useEffect(() => {
    if (
      isEditMode &&
      initialData &&
      !initialDataLoaded &&
      !isModelsLoading &&
      !isTemplatesLoading
    ) {
      form.reset({
        alias: initialData.alias,
        repo: initialData.repo,
        filename: initialData.filename,
        chat_template: initialData.chat_template,
        request_params: initialData.request_params || {},
        context_params: initialData.context_params || {},
      });
      setInitialDataLoaded(true);
    }
  }, [
    isEditMode,
    initialData,
    form,
    initialDataLoaded,
    isModelsLoading,
    isTemplatesLoading,
  ]);

  const uniqueRepos = useMemo(() => {
    if (!modelsData?.data) return [];
    return Array.from(new Set(modelsData.data.map((model) => model.repo)));
  }, [modelsData?.data]);

  const selectedRepo = form.watch('repo');

  const availableFilenames = useMemo(() => {
    if (!modelsData?.data || !selectedRepo) return [];
    return Array.from(
      new Set(
        modelsData.data
          .filter((model) => model.repo === selectedRepo)
          .map((model) => model.filename)
      )
    );
  }, [modelsData?.data, selectedRepo]);

  useEffect(() => {
    if (selectedRepo && initialData && initialDataLoaded) {
      if (availableFilenames.includes(initialData.filename)) {
        form.setValue('filename', initialData.filename);
      } else if (!availableFilenames.includes(form.getValues('filename'))) {
        form.setValue('filename', '');
      }
    }
  }, [selectedRepo, availableFilenames, form, initialData, initialDataLoaded]);

  const onSubmit = async (data: AliasFormData) => {
    try {
      let response;
      if (isEditMode) {
        response = await axios.put(
          `/api/ui/models/${initialData?.alias}`,
          data
        );
      } else {
        response = await axios.post('/api/ui/models', data);
      }

      if (response.status === 200 || response.status === 201) {
        toast({
          title: 'Success',
          description: `Alias ${data.alias} successfully ${isEditMode ? 'updated' : 'created'}`,
          duration: 5000,
        });
        router.push('/ui/models');
      }
    } catch (error) {
      console.error(
        `Error ${isEditMode ? 'updating' : 'creating'} alias:`,
        error
      );
      let errorMessage = `Failed to ${isEditMode ? 'update' : 'create'} alias. Please try again.`;

      if (axios.isAxiosError(error)) {
        const axiosError = error as AxiosError<{ message: string }>;
        if (axiosError.response?.data?.message) {
          errorMessage = axiosError.response.data.message;
        }
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

            <FormField
              control={form.control}
              name="repo"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Repo</FormLabel>
                  <Select onValueChange={field.onChange} value={field.value}>
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue placeholder="Select a repo" />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      {uniqueRepos.map((repo) => (
                        <SelectItem key={repo} value={repo}>
                          {repo}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="filename"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Filename</FormLabel>
                  <Select
                    onValueChange={field.onChange}
                    value={field.value}
                    disabled={!selectedRepo}
                  >
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue placeholder="Select a filename" />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      {availableFilenames.map((filename) => (
                        <SelectItem key={filename} value={filename}>
                          {filename}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="chat_template"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Chat Template</FormLabel>
                  <Select onValueChange={field.onChange} value={field.value}>
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue placeholder="Select a chat template" />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      {chatTemplates?.map((template) => (
                        <SelectItem key={template} value={template}>
                          {template}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
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
