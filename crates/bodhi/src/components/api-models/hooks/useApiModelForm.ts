import { useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import {
  ApiModelFormData,
  UpdateApiModelFormData,
  createApiModelSchema,
  updateApiModelSchema,
  convertFormToCreateRequest,
  convertFormToUpdateRequest,
  API_FORMAT_PRESETS,
} from '@/schemas/apiModel';
import { useCreateApiModel, useUpdateApiModel, useApiFormats } from '@/hooks/useApiModels';
import { ApiModelResponse } from '@bodhiapp/ts-client';
import { ApiProvider, API_PROVIDERS } from '../providers/constants';
import { useTestConnection } from './useTestConnection';
import { useFetchModels } from './useFetchModels';

interface UseApiModelFormProps {
  mode: 'create' | 'edit' | 'setup';
  initialData?: ApiModelResponse;
  onSuccess?: (data: any) => void;
  onError?: (error: string) => void;
  onCancel?: () => void;
  autoSelectCommon?: boolean;
}

export function useApiModelForm({
  mode,
  initialData,
  onSuccess,
  onError,
  onCancel,
  autoSelectCommon = false,
}: UseApiModelFormProps) {
  // Provider state
  const [selectedProvider, setSelectedProvider] = useState<ApiProvider | null>(null);

  // Form setup
  const isEditMode = mode === 'edit';
  const schema = isEditMode ? updateApiModelSchema : createApiModelSchema;
  type FormData = typeof isEditMode extends true ? UpdateApiModelFormData : ApiModelFormData;

  const form = useForm<FormData>({
    resolver: zodResolver(schema),
    defaultValues: isEditMode
      ? {
          api_format: initialData?.api_format || 'openai',
          base_url: initialData?.base_url || 'https://api.openai.com/v1',
          api_key: '',
          models: initialData?.models || [],
          prefix: initialData?.prefix || '',
          usePrefix: Boolean(initialData?.prefix),
          useApiKey: initialData?.api_key_masked != null, // "***" = has key, checkbox checked
          forward_all_with_prefix: initialData?.forward_all_with_prefix || false,
        }
      : mode === 'setup'
        ? {
            api_format: '',
            base_url: '',
            api_key: '',
            models: [],
            prefix: '',
            usePrefix: false,
            useApiKey: false,
            forward_all_with_prefix: false,
          }
        : {
            api_format: 'openai',
            base_url: 'https://api.openai.com/v1',
            api_key: '',
            models: [],
            prefix: '',
            usePrefix: false,
            useApiKey: false,
            forward_all_with_prefix: false,
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

  // API mutations with callbacks
  const createMutation = useCreateApiModel({
    onSuccess: (response) => {
      if (onSuccess) {
        onSuccess(response.data);
      }
    },
    onError: (error) => {
      const errorMessage = error.response?.data?.error?.message || error.message || 'Failed to create API model';
      if (onError) {
        onError(errorMessage);
      }
    },
  });

  const updateMutation = useUpdateApiModel({
    onSuccess: (response) => {
      if (onSuccess) {
        onSuccess(response.data);
      }
    },
    onError: (error) => {
      const errorMessage = error.response?.data?.error?.message || error.message || 'Failed to update API model';
      if (onError) {
        onError(errorMessage);
      }
    },
  });
  const { data: apiFormatsData } = useApiFormats();

  // Business logic hooks
  const testConnection = useTestConnection({ mode, initialData });
  const fetchModels = useFetchModels({
    mode,
    initialData,
    autoSelectCommon,
    provider: selectedProvider,
    onModelsUpdated: (models) => setValue('models', models),
  });

  // Initialize provider selection
  useEffect(() => {
    if (watchedValues.api_format && !selectedProvider) {
      // Only auto-select provider if no provider is currently selected
      const provider = API_PROVIDERS.find((p) => p.format === watchedValues.api_format);
      if (provider) {
        setSelectedProvider(provider);
      }
    } else if (!watchedValues.api_format && mode === 'setup' && selectedProvider) {
      // For setup mode, reset provider when api_format becomes empty
      setSelectedProvider(null);
    }
  }, [watchedValues.api_format, mode, selectedProvider]);

  // Provider selection handler
  const handleProviderSelect = (provider: ApiProvider | null) => {
    if (provider) {
      setSelectedProvider(provider);
      setValue('api_format', provider.format);
      setValue('base_url', provider.baseUrl);
      setValue('models', []);
      setValue('prefix', '');
      setValue('usePrefix', false);
      setValue('useApiKey', false);
      fetchModels.clearModels();
      testConnection.resetStatus();
    }
  };

  // API format change handler
  const handleApiFormatChange = (apiFormat: string) => {
    const preset = API_FORMAT_PRESETS[apiFormat as keyof typeof API_FORMAT_PRESETS];
    if (preset) {
      setValue('api_format', apiFormat);
      setValue('base_url', preset.baseUrl);
      setValue('models', []);
      setValue('prefix', '');
      setValue('usePrefix', false);
      setValue('useApiKey', false);
      fetchModels.clearModels();
      testConnection.resetStatus();
    }
  };

  // Model selection handlers
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
    setValue('models', allModels);
  };

  // Test connection wrapper
  const handleTestConnection = async () => {
    if (!watchedValues.base_url) return;

    await testConnection.testConnection({
      apiKey: watchedValues.useApiKey ? watchedValues.api_key : undefined,
      id: !watchedValues.useApiKey && isEditMode ? initialData?.id : undefined,
      baseUrl: watchedValues.base_url,
      model: watchedValues.models?.[0] || 'gpt-3.5-turbo',
    });
  };

  // Fetch models wrapper
  const handleFetchModels = async () => {
    await fetchModels.fetchModels({
      apiKey: watchedValues.useApiKey ? watchedValues.api_key : undefined,
      id: !watchedValues.useApiKey && isEditMode ? initialData?.id : undefined,
      baseUrl: watchedValues.base_url || '',
    });
  };

  // Form submission
  const onSubmit = async (data: FormData) => {
    if (isEditMode && initialData) {
      const updateData = convertFormToUpdateRequest(data as UpdateApiModelFormData, initialData);
      updateMutation.mutate({
        id: initialData.id,
        data: updateData,
      });
    } else {
      const createData = convertFormToCreateRequest(data as ApiModelFormData);
      createMutation.mutate(createData);
    }
  };

  // Cancel handler
  const handleCancel = () => {
    if (onCancel) {
      onCancel();
    }
  };

  // Computed values
  const canTest = Boolean(watchedValues.base_url && watchedValues.models?.length > 0);

  const canFetch = Boolean(watchedValues.base_url);

  return {
    // Form state
    form,
    register,
    handleSubmit: handleSubmit(onSubmit),
    watch,
    setValue,
    errors,
    isSubmitting,
    watchedValues,

    // Provider state
    selectedProvider,
    handleProviderSelect,

    // API formats
    apiFormatsData: apiFormatsData?.data || ['openai'],
    handleApiFormatChange,

    // Model management
    handleModelSelect,
    handleModelRemove,
    handleModelsSelectAll,

    // Test connection
    testConnection: {
      onTest: handleTestConnection,
      canTest,
      isLoading: testConnection.isLoading,
      status: testConnection.status,
      disabledReason: testConnection.getMissingRequirements({
        apiKey: watchedValues.api_key,
        baseUrl: watchedValues.base_url || '',
        model: watchedValues.models?.[0] || '',
      }),
    },

    // Fetch models
    fetchModels: {
      onFetch: handleFetchModels,
      canFetch,
      isLoading: fetchModels.isLoading,
      status: fetchModels.status,
      availableModels: fetchModels.availableModels,
      disabledReason: fetchModels.getFetchDisabledReason({
        apiKey: watchedValues.api_key,
        baseUrl: watchedValues.base_url || '',
      }),
    },

    // Actions
    handleCancel,
    isLoading: isSubmitting || createMutation.isLoading || updateMutation.isLoading,
    mode,
    isEditMode,
  };
}
