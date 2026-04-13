import { useEffect, useState } from 'react';

import { ApiAliasResponse, ApiFormat, ApiKey, TestCreds } from '@bodhiapp/ts-client';
import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';

import { useCreateApiModel, useUpdateApiModel, useListApiFormats } from '@/hooks/models';
import {
  getApiModelId,
  ApiModelFormData,
  UpdateApiModelFormData,
  createApiModelSchema,
  updateApiModelSchema,
  convertFormToCreateRequest,
  convertFormToUpdateRequest,
  parseJsonField,
  serializeJsonField,
  API_FORMAT_PRESETS,
} from '@/schemas/apiModel';

import { ApiProvider, API_PROVIDERS, DEFAULT_TEST_PROMPT } from '../providers/constants';

import { useFetchModels } from './useFetchModels';
import { useTestConnection } from './useTestConnection';

interface UseApiModelFormProps {
  mode: 'create' | 'edit' | 'setup';
  initialData?: ApiAliasResponse;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
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
          models: initialData?.models?.map((m) => getApiModelId(m, initialData?.prefix)) || [],
          prefix: initialData?.prefix || '',
          usePrefix: Boolean(initialData?.prefix),
          useApiKey: initialData?.has_api_key === true, // true = has key, checkbox checked
          forward_all_with_prefix: initialData?.forward_all_with_prefix || false,
          extra_headers: serializeJsonField(initialData?.extra_headers),
          extra_body: serializeJsonField(initialData?.extra_body),
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
            extra_headers: '',
            extra_body: '',
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
            extra_headers: '',
            extra_body: '',
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
  const { data: apiFormatsData } = useListApiFormats();

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

  // API format change handler: treat format change as dirty — reset to preset
  // defaults (or empty when no preset). No revert-to-initial tracking; user
  // must cancel or refresh to restore stored values.
  const handleApiFormatChange = (apiFormat: string) => {
    const preset = API_FORMAT_PRESETS[apiFormat as keyof typeof API_FORMAT_PRESETS];
    const presetHeaders =
      preset && 'defaultHeaders' in preset && preset.defaultHeaders
        ? JSON.stringify(preset.defaultHeaders, null, 2)
        : '';
    const presetBody =
      preset && 'defaultBody' in preset && preset.defaultBody ? JSON.stringify(preset.defaultBody, null, 2) : '';
    setValue('api_format', apiFormat);
    setValue('models', []);
    setValue('prefix', '');
    setValue('usePrefix', false);
    setValue('useApiKey', false);
    setValue('api_key', '');
    setValue('base_url', preset?.baseUrl ?? '');
    setValue('extra_headers', presetHeaders);
    setValue('extra_body', presetBody);
    fetchModels.clearModels();
    testConnection.resetStatus();
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

  // Test connection wrapper — build TestPromptRequest from form state
  const handleTestConnection = async () => {
    if (!watchedValues.base_url) return;

    let creds: TestCreds | undefined;
    const apiKey = watchedValues.useApiKey ? watchedValues.api_key : undefined;
    const id = !watchedValues.useApiKey && isEditMode ? initialData?.id : undefined;
    if (apiKey) {
      creds = { type: 'api_key' as const, value: apiKey as ApiKey };
    } else if (id) {
      creds = { type: 'id' as const, value: id };
    } else {
      creds = { type: 'api_key' as const, value: null };
    }

    const extraHeaders = parseJsonField(watchedValues.extra_headers);
    const extraBody = parseJsonField(watchedValues.extra_body);

    await testConnection.testConnection({
      creds,
      base_url: watchedValues.base_url,
      model: watchedValues.models?.[0] || 'gpt-3.5-turbo',
      prompt: DEFAULT_TEST_PROMPT,
      api_format: watchedValues.api_format as ApiFormat,
      ...(extraHeaders !== null ? { extra_headers: extraHeaders } : {}),
      ...(extraBody !== null ? { extra_body: extraBody } : {}),
    });
  };

  // Fetch models wrapper
  const handleFetchModels = async () => {
    await fetchModels.fetchModels({
      apiKey: watchedValues.useApiKey ? watchedValues.api_key : undefined,
      id: !watchedValues.useApiKey && isEditMode ? initialData?.id : undefined,
      baseUrl: watchedValues.base_url || '',
      apiFormat: watchedValues.api_format as ApiFormat,
      extraHeaders: parseJsonField(watchedValues.extra_headers),
      extraBody: parseJsonField(watchedValues.extra_body),
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
  const canTest = Boolean(watchedValues.base_url);

  const canFetch = Boolean(watchedValues.base_url);

  // Show extras section when the preset declares defaults for either field.
  const showExtras = Boolean(
    watchedValues.api_format &&
      API_FORMAT_PRESETS[watchedValues.api_format as keyof typeof API_FORMAT_PRESETS] &&
      ('defaultHeaders' in API_FORMAT_PRESETS[watchedValues.api_format as keyof typeof API_FORMAT_PRESETS] ||
        'defaultBody' in API_FORMAT_PRESETS[watchedValues.api_format as keyof typeof API_FORMAT_PRESETS])
  );

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
    showExtras,

    // Provider state
    selectedProvider,

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
        base_url: watchedValues.base_url || '',
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
        apiFormat: watchedValues.api_format as ApiFormat,
      }),
    },

    // Actions
    handleCancel,
    isLoading: isSubmitting || createMutation.isPending || updateMutation.isPending,
    mode,
    isEditMode,
  };
}
