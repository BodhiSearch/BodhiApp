'use client';

import React, { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { TestTube, Eye, EyeOff } from 'lucide-react';
import { useToast } from '@/hooks/use-toast';
import {
  ApiModelFormData,
  UpdateApiModelFormData,
  createApiModelSchema,
  updateApiModelSchema,
  convertFormToCreateRequest,
  convertFormToUpdateRequest,
  PROVIDER_PRESETS,
  ProviderPreset,
} from '@/schemas/apiModel';
import { useCreateApiModel, useUpdateApiModel, useTestApiModel, useFetchApiModels } from '@/hooks/useApiModels';
import { ApiModelResponse, TestPromptRequest, FetchModelsRequest } from '@bodhiapp/ts-client';
import { ModelSelector } from '@/components/ModelSelector';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';

interface ApiModelFormProps {
  isEditMode: boolean;
  initialData?: ApiModelResponse;
}

export default function ApiModelForm({ isEditMode, initialData }: ApiModelFormProps) {
  const router = useRouter();
  const { toast } = useToast();
  const [selectedProvider, setSelectedProvider] = useState<ProviderPreset>('openai');
  const [availableModels, setAvailableModels] = useState<string[]>([]);
  const [showApiKey, setShowApiKey] = useState(false);
  const [fetchModelsState, setFetchModelsState] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');

  const createMutation = useCreateApiModel();
  const updateMutation = useUpdateApiModel();
  const testMutation = useTestApiModel();
  const fetchModelsMutation = useFetchApiModels();

  const schema = isEditMode ? updateApiModelSchema : createApiModelSchema;
  type FormData = typeof isEditMode extends true ? UpdateApiModelFormData : ApiModelFormData;

  const form = useForm<FormData>({
    resolver: zodResolver(schema),
    defaultValues: isEditMode
      ? {
          provider: initialData?.provider || 'OpenAI',
          base_url: initialData?.base_url || 'https://api.openai.com/v1',
          api_key: '',
          models: initialData?.models || [],
          prefix: initialData?.prefix || '',
          usePrefix: Boolean(initialData?.prefix),
        }
      : {
          id: '',
          provider: 'OpenAI',
          base_url: 'https://api.openai.com/v1',
          api_key: '',
          models: [],
          prefix: '',
          usePrefix: false,
        },
  });

  const {
    register,
    handleSubmit,
    watch,
    setValue,
    formState: { errors, isSubmitting },
  } = form;
  const watchedValues = watch();

  useEffect(() => {
    if (initialData) {
      setSelectedProvider('openai');
      // Do not pre-populate available models - user must fetch them
    }
  }, [initialData]);

  const handleProviderChange = (provider: ProviderPreset) => {
    setSelectedProvider(provider);
    const preset = PROVIDER_PRESETS[provider];

    setValue('provider', preset.name);
    setValue('base_url', preset.baseUrl);
    setValue('models', []); // Clear selected models
    setValue('prefix', ''); // Clear prefix when provider changes
    setValue('usePrefix', false); // Uncheck prefix checkbox
    setAvailableModels([]); // Clear available models - user must fetch them
  };

  const handleModelSelect = (modelName: string) => {
    if (!modelName.trim()) return;

    const currentModels = watchedValues.models || [];
    if (!currentModels.includes(modelName.trim())) {
      setValue('models', [...currentModels, modelName.trim()]);
    }
  };

  const handleModelRemove = (modelName: string) => {
    const currentModels = watchedValues.models || [];
    setValue(
      'models',
      currentModels.filter((m) => m !== modelName)
    );
  };

  const handleModelsSelectAll = (allModels: string[]) => {
    // Set all models at once to prevent multiple form updates
    setValue('models', allModels);
  };

  const handleTestConnection = async () => {
    const testData: TestPromptRequest = {
      // In edit mode, use stored model ID if no API key provided
      ...(watchedValues.api_key
        ? { api_key: watchedValues.api_key }
        : isEditMode && initialData?.id
          ? { id: initialData.id }
          : { api_key: watchedValues.api_key || '' }),
      base_url: watchedValues.base_url || '',
      model: watchedValues.models[0], // First selected model
      prompt: 'Hello, how are you?', // Default prompt
    };

    try {
      const response = await testMutation.mutateAsync(testData);
      toast({
        title: response.data.success ? 'Connection Test Successful' : 'Connection Test Failed',
        description: response.data.response || response.data.error || 'Test completed',
      });
    } catch (error: unknown) {
      const errorMessage =
        (error as { response?: { data?: { error?: { message?: string } } }; message?: string }).response?.data?.error
          ?.message ||
        (error as { message?: string }).message ||
        'Failed to test connection';
      toast({
        title: 'Connection Test Failed',
        description: errorMessage,
        variant: 'destructive',
      });
    }
  };

  const handleFetchModels = async () => {
    const fetchData: FetchModelsRequest = {
      // In edit mode, use stored model ID if no API key provided
      ...(watchedValues.api_key
        ? { api_key: watchedValues.api_key }
        : isEditMode && initialData?.id
          ? { id: initialData.id }
          : { api_key: watchedValues.api_key || '' }),
      base_url: watchedValues.base_url || '',
    };

    setFetchModelsState('loading');
    try {
      const response = await fetchModelsMutation.mutateAsync(fetchData);
      const models = response.data.models;
      setAvailableModels(models);
      setFetchModelsState('success');
      toast({
        title: 'Models Fetched Successfully',
        description: `Found ${models.length} available models`,
      });
    } catch (error: unknown) {
      setFetchModelsState('error');
      const errorMessage =
        (error as { response?: { data?: { error?: { message?: string } } }; message?: string }).response?.data?.error
          ?.message ||
        (error as { message?: string }).message ||
        'Failed to fetch models';
      toast({
        title: 'Failed to Fetch Models',
        description: errorMessage,
        variant: 'destructive',
      });
    }
  };

  const onSubmit = async (data: FormData) => {
    try {
      if (isEditMode && initialData) {
        const updateData = convertFormToUpdateRequest(data as UpdateApiModelFormData);
        await updateMutation.mutateAsync({
          id: initialData.id,
          data: updateData,
        });
        toast({
          title: 'API Model Updated',
          description: `Successfully updated ${initialData.id}`,
        });
      } else {
        const createData = convertFormToCreateRequest(data as ApiModelFormData);
        await createMutation.mutateAsync(createData);
        toast({
          title: 'API Model Created',
          description: `Successfully created ${createData.id}`,
        });
      }
      router.push('/ui/models');
    } catch (error: unknown) {
      const errorMessage =
        (error as { response?: { data?: { error?: { message?: string } } }; message?: string }).response?.data?.error
          ?.message ||
        (error as { message?: string }).message ||
        'An unexpected error occurred';
      toast({
        title: isEditMode ? 'Failed to Update API Model' : 'Failed to Create API Model',
        description: errorMessage,
        variant: 'destructive',
      });
    }
  };

  const canTest = Boolean(
    watchedValues.base_url &&
      (watchedValues.api_key || (isEditMode && initialData?.id)) &&
      watchedValues.models?.length > 0
  );
  const canFetch = Boolean(watchedValues.base_url && (watchedValues.api_key || (isEditMode && initialData?.id)));

  // Get missing requirements for tooltip
  const getMissingRequirements = () => {
    const missing = [];
    if (!watchedValues.base_url) missing.push('base URL');
    if (!watchedValues.api_key && !(isEditMode && initialData?.id)) {
      missing.push(isEditMode ? 'API key (or use stored credentials)' : 'API key');
    }
    if (!watchedValues.models?.length) missing.push('at least one model');
    return `You need to add ${missing.join(', ')} to test connection`;
  };

  const getFetchDisabledReason = () => {
    const missing = [];
    if (!watchedValues.base_url) missing.push('base URL');
    if (!watchedValues.api_key && !(isEditMode && initialData?.id)) {
      missing.push(isEditMode ? 'API key (or use stored credentials)' : 'API key');
    }
    return missing.length > 0 ? `You need to add ${missing.join(', ')} to fetch models` : '';
  };

  return (
    <form
      onSubmit={handleSubmit(onSubmit)}
      className="space-y-8 mx-4 my-6"
      data-testid={isEditMode ? 'edit-api-model-form' : 'create-api-model-form'}
    >
      <Card>
        <CardHeader>
          <CardTitle>{isEditMode ? 'Edit API Model' : 'Create New API Model'}</CardTitle>
          <CardDescription>
            {isEditMode ? 'Update the configuration for your API model' : 'Configure a new external AI API model'}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="space-y-2">
            <Label htmlFor="id">ID</Label>
            <Input
              id="id"
              data-testid="api-model-id"
              {...(isEditMode ? { value: initialData?.id || '', disabled: true } : register('id'))}
              placeholder="my-gpt-4"
            />
            {errors.id && <p className="text-sm text-destructive">{errors.id.message}</p>}
          </div>

          <div className="space-y-2">
            <Label htmlFor="provider-preset">Provider Type</Label>
            <Select value={selectedProvider} onValueChange={handleProviderChange}>
              <SelectTrigger id="provider-preset" data-testid="api-model-provider">
                <SelectValue placeholder="Select a provider type" />
              </SelectTrigger>
              <SelectContent>
                {Object.entries(PROVIDER_PRESETS).map(([key, preset]) => (
                  <SelectItem key={key} value={key}>
                    {preset.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-2">
            <Label htmlFor="base_url">Base URL</Label>
            <Input
              id="base_url"
              data-testid="api-model-base-url"
              {...register('base_url')}
              placeholder="https://api.openai.com/v1"
            />
            {errors.base_url && <p className="text-sm text-destructive">{errors.base_url.message}</p>}
          </div>

          <div className="space-y-2">
            <Label htmlFor="api_key">API Key</Label>
            <div className="relative">
              <Input
                id="api_key"
                data-testid="api-model-api-key"
                type={showApiKey ? 'text' : 'password'}
                {...register('api_key')}
                placeholder={isEditMode ? 'Leave empty to keep existing key' : 'Enter your API key'}
              />
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="absolute right-0 top-0 h-full px-3"
                data-testid="toggle-api-key-visibility"
                onClick={() => setShowApiKey(!showApiKey)}
              >
                {showApiKey ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
              </Button>
            </div>
            {errors.api_key && <p className="text-sm text-destructive">{errors.api_key.message}</p>}
          </div>

          <div className="space-y-2">
            <Label htmlFor="prefix">Model Prefix</Label>
            <div className="flex items-center space-x-2">
              <input
                type="checkbox"
                id="usePrefix"
                data-testid="api-model-use-prefix"
                {...register('usePrefix')}
                className="rounded border-gray-300 focus:ring-2 focus:ring-blue-500"
              />
              <Label htmlFor="usePrefix" className="text-sm text-muted-foreground">
                Enable prefix
              </Label>
              <Input
                id="prefix"
                data-testid="api-model-prefix"
                {...register('prefix')}
                placeholder="e.g., 'azure/', 'openai:', 'provider-', 'my.custom_'"
                disabled={!watchedValues.usePrefix}
                className="flex-1"
              />
            </div>
            <p className="text-sm text-muted-foreground">
              Prefix to differentiate models from different providers. You can use any separator you prefer (e.g.,
              "azure/" → "azure/gpt-4", "openai:" → "openai:gpt-4", "provider-" → "provider-gpt-4"). Leave empty or
              uncheck to use no prefix.
            </p>
            {errors.prefix && <p className="text-sm text-destructive">{errors.prefix.message}</p>}
          </div>

          <div
            data-testid="fetch-models-container"
            data-fetch-state={fetchModelsState}
            data-models-fetched={availableModels.length > 0}
            data-can-fetch={canFetch}
          >
            <ModelSelector
              selectedModels={watchedValues.models || []}
              availableModels={availableModels}
              onModelSelect={handleModelSelect}
              onModelRemove={handleModelRemove}
              onModelsSelectAll={handleModelsSelectAll}
              onFetchModels={handleFetchModels}
              isFetchingModels={fetchModelsMutation.isLoading}
              canFetch={canFetch}
              fetchDisabledReason={getFetchDisabledReason()}
            />
          </div>
          {errors.models && <p className="text-sm text-destructive">{errors.models.message}</p>}

          <div className="flex justify-between">
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <span>
                    <Button
                      type="button"
                      variant="outline"
                      data-testid="test-connection-button"
                      onClick={handleTestConnection}
                      disabled={!canTest || testMutation.isLoading}
                    >
                      <TestTube className="h-4 w-4 mr-2" />
                      {testMutation.isLoading ? 'Testing...' : 'Test Connection'}
                    </Button>
                  </span>
                </TooltipTrigger>
                {!canTest && !testMutation.isLoading && (
                  <TooltipContent>
                    <p>{getMissingRequirements()}</p>
                  </TooltipContent>
                )}
              </Tooltip>
            </TooltipProvider>

            <div className="flex gap-2">
              <Button
                type="button"
                variant="outline"
                data-testid="cancel-button"
                onClick={() => router.push('/ui/models')}
              >
                Cancel
              </Button>
              <Button
                type="submit"
                data-testid={isEditMode ? 'update-api-model-button' : 'create-api-model-button'}
                disabled={isSubmitting || createMutation.isLoading || updateMutation.isLoading}
              >
                {isSubmitting || createMutation.isLoading || updateMutation.isLoading
                  ? isEditMode
                    ? 'Updating...'
                    : 'Creating...'
                  : isEditMode
                    ? 'Update API Model'
                    : 'Create API Model'}
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </form>
  );
}
