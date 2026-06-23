import { useEffect, useState } from 'react';

import { ApiAliasResponse, ApiFormat, ApiKey, LlmLibertyEnvelope, TestCreds } from '@bodhiapp/ts-client';
import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';

import { useCreateApiModel, useUpdateApiModel, useListApiFormats } from '@/hooks/models';
import { extractErrorMessage } from '@/lib/errorUtils';
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
} from '@/schemas/apiModel';

import { ApiProvider, API_PROVIDERS, DEFAULT_TEST_PROMPT } from '../providers/constants';
import type { ApiModelPrefill } from '../types';
import {
  computeCanFetch,
  computeCanTest,
  computeShowExtras,
  llmLibertyFetchDisabledReason,
  llmLibertyTestDisabledReason,
  presetBaseUrl,
  presetExtras,
} from '../validation';

import { useFetchModels } from './useFetchModels';
import { useTestConnection } from './useTestConnection';

interface UseApiModelFormProps {
  mode: 'create' | 'edit' | 'setup';
  initialData?: ApiAliasResponse;
  prefill?: ApiModelPrefill;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  onSuccess?: (data: any) => void;
  onError?: (error: string) => void;
  onCancel?: () => void;
  autoSelectCommon?: boolean;
}

export function useApiModelForm({
  mode,
  initialData,
  prefill,
  onSuccess,
  onError,
  onCancel,
  autoSelectCommon = false,
}: UseApiModelFormProps) {
  const [selectedProvider, setSelectedProvider] = useState<ApiProvider | null>(null);

  const isEditMode = mode === 'edit';
  const schema = isEditMode ? updateApiModelSchema : createApiModelSchema;
  type FormData = typeof isEditMode extends true ? UpdateApiModelFormData : ApiModelFormData;

  const form = useForm<FormData>({
    resolver: zodResolver(schema),
    defaultValues: isEditMode
      ? {
          name: initialData?.name || '',
          api_format: initialData?.api_format || 'openai',
          base_url: initialData?.base_url || 'https://api.openai.com/v1',
          api_key: '',
          models: initialData?.models?.map((m) => getApiModelId(m, initialData?.prefix)) || [],
          prefix: initialData?.prefix || '',
          usePrefix: Boolean(initialData?.prefix),
          useApiKey: initialData?.has_api_key === true,
          forward_all_with_prefix: initialData?.forward_all_with_prefix || false,
          extra_headers: serializeJsonField(initialData?.extra_headers),
          extra_body: serializeJsonField(initialData?.extra_body),
          llm_liberty_envelope: '',
        }
      : mode === 'setup'
        ? {
            name: '',
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
            llm_liberty_envelope: '',
          }
        : {
            name: '',
            // Prefill (Explore catalog bridge) overrides OpenAI defaults; null base_url keeps the preset.
            api_format: prefill?.api_format || 'openai',
            base_url: prefill?.base_url || 'https://api.openai.com/v1',
            api_key: '', // never prefilled — the user always supplies their own key
            models: prefill?.model ? [prefill.model] : [],
            prefix: '',
            usePrefix: false,
            useApiKey: false,
            forward_all_with_prefix: false,
            extra_headers: '',
            extra_body: '',
            llm_liberty_envelope: '',
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

  const createMutation = useCreateApiModel({
    onSuccess: (response) => {
      if (onSuccess) {
        onSuccess(response.data);
      }
    },
    onError: (error) => {
      if (onError) {
        onError(extractErrorMessage(error, 'Failed to create API model'));
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
      if (onError) {
        onError(extractErrorMessage(error, 'Failed to update API model'));
      }
    },
  });
  const { data: apiFormatsData } = useListApiFormats();

  const testConnection = useTestConnection({ mode, initialData });
  const fetchModels = useFetchModels({
    mode,
    initialData,
    autoSelectCommon,
    provider: selectedProvider,
    onModelsUpdated: (models) => setValue('models', models),
  });

  useEffect(() => {
    if (watchedValues.api_format && !selectedProvider) {
      const provider = API_PROVIDERS.find((p) => p.format === watchedValues.api_format);
      if (provider) {
        setSelectedProvider(provider);
      }
    } else if (!watchedValues.api_format && mode === 'setup' && selectedProvider) {
      setSelectedProvider(null);
    }
  }, [watchedValues.api_format, mode, selectedProvider]);

  // Format change is destructive: reset all fields to the new preset's defaults; stored values are not recoverable without cancel/refresh.
  const handleApiFormatChange = (apiFormat: string) => {
    const { headers: presetHeaders, body: presetBody } = presetExtras(apiFormat);
    setValue('api_format', apiFormat);
    setValue('models', []);
    setValue('prefix', '');
    setValue('usePrefix', false);
    setValue('useApiKey', false);
    setValue('api_key', '');
    setValue('base_url', presetBaseUrl(apiFormat));
    setValue('extra_headers', presetHeaders);
    setValue('extra_body', presetBody);
    setValue('llm_liberty_envelope', '');
    fetchModels.clearModels();
    testConnection.resetStatus();
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
    setValue('models', allModels);
  };

  const parseLlmLibertyEnvelope = (): LlmLibertyEnvelope | null => {
    const text = watchedValues.llm_liberty_envelope;
    if (!text?.trim()) return null;
    try {
      return JSON.parse(text) as LlmLibertyEnvelope;
    } catch {
      return null;
    }
  };

  const handleTestConnection = async () => {
    const fmt = watchedValues.api_format as ApiFormat;

    if (fmt === 'llm_liberty_oauth') {
      const model = watchedValues.models?.[0];
      if (!model) return;
      const aliasId = isEditMode ? initialData?.id : undefined;
      if (aliasId) {
        await testConnection.testConnection({
          api_format: 'llm_liberty_oauth',
          id: aliasId,
          model,
          prompt: DEFAULT_TEST_PROMPT,
        });
        return;
      }
      const env = parseLlmLibertyEnvelope();
      if (!env) return;
      await testConnection.testConnection({
        api_format: 'llm_liberty_oauth',
        envelope: env,
        model,
        prompt: DEFAULT_TEST_PROMPT,
      });
      return;
    }

    if (!watchedValues.base_url) return;

    let creds: TestCreds;
    const apiKey = watchedValues.useApiKey ? watchedValues.api_key : undefined;
    const id = !watchedValues.useApiKey && isEditMode ? initialData?.id : undefined;
    if (apiKey) {
      creds = { type: 'api_key', value: apiKey as ApiKey };
    } else if (id) {
      creds = { type: 'id', value: id };
    } else {
      creds = { type: 'api_key', value: null };
    }

    const extraHeaders = parseJsonField(watchedValues.extra_headers);
    const extraBody = parseJsonField(watchedValues.extra_body);

    await testConnection.testConnection({
      api_format: fmt,
      creds,
      base_url: watchedValues.base_url,
      model: watchedValues.models?.[0] || 'gpt-3.5-turbo',
      prompt: DEFAULT_TEST_PROMPT,
      ...(extraHeaders !== null ? { extra_headers: extraHeaders } : {}),
      ...(extraBody !== null ? { extra_body: extraBody } : {}),
    });
  };

  const handleFetchModels = async () => {
    const fmt = watchedValues.api_format as ApiFormat;

    if (fmt === 'llm_liberty_oauth') {
      const aliasId = isEditMode ? initialData?.id : undefined;
      const env = aliasId ? undefined : (parseLlmLibertyEnvelope() ?? undefined);
      if (!aliasId && !env) return;
      await fetchModels.fetchModels({
        apiFormat: 'llm_liberty_oauth',
        baseUrl: '',
        id: aliasId,
        llmLibertyEnvelope: env,
      });
      return;
    }

    await fetchModels.fetchModels({
      apiKey: watchedValues.useApiKey ? watchedValues.api_key : undefined,
      id: !watchedValues.useApiKey && isEditMode ? initialData?.id : undefined,
      baseUrl: watchedValues.base_url || '',
      apiFormat: fmt,
      extraHeaders: parseJsonField(watchedValues.extra_headers),
      extraBody: parseJsonField(watchedValues.extra_body),
    });
  };

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

  const handleCancel = () => {
    if (onCancel) {
      onCancel();
    }
  };

  const canTest = computeCanTest(watchedValues, isEditMode);
  const canFetch = computeCanFetch(watchedValues, isEditMode);
  const showExtras = computeShowExtras(watchedValues.api_format);

  return {
    form,
    register,
    handleSubmit: handleSubmit(onSubmit),
    watch,
    setValue,
    errors,
    isSubmitting,
    watchedValues,
    showExtras,

    selectedProvider,

    apiFormatsData: apiFormatsData?.data || ['openai'],
    handleApiFormatChange,

    handleModelSelect,
    handleModelRemove,
    handleModelsSelectAll,

    testConnection: {
      onTest: handleTestConnection,
      canTest,
      isLoading: testConnection.isLoading,
      status: testConnection.status,
      disabledReason:
        llmLibertyTestDisabledReason(watchedValues, isEditMode) ??
        testConnection.getMissingRequirements({
          base_url: watchedValues.base_url || '',
          model: watchedValues.models?.[0] || '',
        }),
    },

    fetchModels: {
      onFetch: handleFetchModels,
      canFetch,
      isLoading: fetchModels.isLoading,
      status: fetchModels.status,
      availableModels: fetchModels.availableModels,
      disabledReason:
        llmLibertyFetchDisabledReason(watchedValues, isEditMode) ??
        fetchModels.getFetchDisabledReason({
          apiKey: watchedValues.api_key,
          baseUrl: watchedValues.base_url || '',
          apiFormat: watchedValues.api_format as ApiFormat,
        }),
    },

    handleCancel,
    isLoading: isSubmitting || createMutation.isPending || updateMutation.isPending,
    mode,
    isEditMode,
  };
}
