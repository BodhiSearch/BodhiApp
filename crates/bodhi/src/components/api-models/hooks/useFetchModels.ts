import { useState } from 'react';
import { useFetchApiModels } from '@/hooks/useApiModels';
import { useToast } from '@/hooks/use-toast';
import { FetchModelsRequest } from '@bodhiapp/ts-client';
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
    return Boolean(data.baseUrl && (data.apiKey || (mode === 'edit' && initialData?.id)));
  };

  const getFetchDisabledReason = (data: FetchModelsData) => {
    const missing = [];
    if (!data.baseUrl) missing.push('base URL');
    if (!data.apiKey && !(mode === 'edit' && initialData?.id)) {
      missing.push(mode === 'edit' ? 'API key (or use stored credentials)' : 'API key');
    }
    return missing.length > 0 ? `You need to add ${missing.join(', ')} to fetch models` : '';
  };

  const fetchModels = async (data: FetchModelsData) => {
    if (!canFetch(data)) {
      return;
    }

    dismiss();
    setStatus('loading');

    const fetchData: FetchModelsRequest = {
      // In edit mode, use stored model ID if no API key provided
      ...(data.apiKey
        ? { api_key: data.apiKey }
        : mode === 'edit' && initialData?.id
          ? { id: initialData.id }
          : { api_key: data.apiKey || '' }),
      base_url: data.baseUrl,
    };

    try {
      const response = await fetchModelsMutation.mutateAsync(fetchData);
      const models = response.data.models;
      setAvailableModels(models);
      setStatus('success');

      // Auto-select common models if enabled and provider available
      if (autoSelectCommon && provider?.commonModels.length && onModelsUpdated) {
        const commonModelsAvailable = provider.commonModels.filter((model) => models.includes(model));
        if (commonModelsAvailable.length > 0) {
          onModelsUpdated(commonModelsAvailable.slice(0, 3)); // Select up to 3 common models
        }
      }

      toast({
        title: 'Models Fetched Successfully',
        description: `Found ${models.length} available models`,
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
    isLoading: fetchModelsMutation.isLoading,
    availableModels,
    canFetch,
    getFetchDisabledReason,
    fetchModels,
    resetStatus,
    clearModels,
  };
}
