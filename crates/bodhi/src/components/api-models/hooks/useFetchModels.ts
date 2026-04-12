import { useState } from 'react';

import { ApiFormat, FetchModelsRequest, TestCreds, ApiKey } from '@bodhiapp/ts-client';

import { useFetchApiModels } from '@/hooks/models';
import { useToast } from '@/hooks/use-toast';

import { ApiProvider } from '../providers/constants';

interface UseFetchModelsProps {
  mode?: 'create' | 'edit' | 'setup';
  initialData?: { id: string };
  onModelsUpdated?: (models: string[]) => void;
  autoSelectCommon?: boolean;
  provider?: ApiProvider | null;
}

interface FetchModelsData {
  apiKey?: string;
  baseUrl: string;
  id?: string;
  apiFormat: ApiFormat;
  extraHeaders?: unknown;
  extraBody?: unknown;
}

export function useFetchModels({
  mode = 'create',
  initialData,
  onModelsUpdated,
  autoSelectCommon = false,
  provider,
}: UseFetchModelsProps = {}) {
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [availableModels, setAvailableModels] = useState<string[]>([]);
  const fetchModelsMutation = useFetchApiModels();
  const { toast, dismiss } = useToast();

  const canFetch = (data: FetchModelsData) => {
    return Boolean(data.baseUrl);
  };

  const getFetchDisabledReason = (data: FetchModelsData) => {
    if (!data.baseUrl) {
      return 'You need to add base URL to fetch models';
    }
    return '';
  };

  const fetchModels = async (data: FetchModelsData) => {
    dismiss();
    setStatus('loading');

    // Build TestCreds discriminated union
    let creds: TestCreds | undefined;

    if (data.apiKey) {
      // Use provided API key directly
      creds = { type: 'api_key' as const, value: data.apiKey as ApiKey };
    } else if (mode === 'edit' && initialData?.id) {
      // Look up stored credentials by ID in edit mode
      creds = { type: 'id' as const, value: initialData.id };
    } else {
      // No authentication (public API)
      creds = { type: 'api_key' as const, value: null };
    }

    const fetchData: FetchModelsRequest = {
      creds,
      base_url: data.baseUrl,
      api_format: data.apiFormat,
      ...(data.extraHeaders !== null && data.extraHeaders !== undefined ? { extra_headers: data.extraHeaders } : {}),
      ...(data.extraBody !== null && data.extraBody !== undefined ? { extra_body: data.extraBody } : {}),
    };

    try {
      const response = await fetchModelsMutation.mutateAsync(fetchData);
      const modelIds = response.data.models;
      setAvailableModels(modelIds);
      setStatus('success');

      // Auto-select common models if enabled and provider available
      if (autoSelectCommon && provider?.commonModels.length && onModelsUpdated) {
        const commonModelsAvailable = provider.commonModels.filter((model) => modelIds.includes(model));
        if (commonModelsAvailable.length > 0) {
          onModelsUpdated(commonModelsAvailable.slice(0, 3)); // Select up to 3 common models
        }
      }

      toast({
        title: 'Models Fetched Successfully',
        description: `Found ${modelIds.length} available models`,
      });
    } catch (error: unknown) {
      setStatus('error');
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

  const resetStatus = () => {
    setStatus('idle');
    setAvailableModels([]);
  };

  const clearModels = () => {
    setAvailableModels([]);
    setStatus('idle');
  };

  return {
    status,
    isLoading: fetchModelsMutation.isPending,
    availableModels,
    canFetch,
    getFetchDisabledReason,
    fetchModels,
    resetStatus,
    clearModels,
  };
}
