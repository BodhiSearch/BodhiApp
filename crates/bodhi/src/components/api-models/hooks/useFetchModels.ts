import { useState } from 'react';

import { ApiFormat, ApiKey, FetchModelsRequest, LlmLibertyEnvelope, TestCreds } from '@bodhiapp/ts-client';

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
  /** For api_format === 'llm_liberty_oauth' in create mode, the parsed envelope. */
  llmLibertyEnvelope?: LlmLibertyEnvelope;
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
    if (data.apiFormat === 'llm_liberty_oauth') {
      return Boolean(data.id || data.llmLibertyEnvelope);
    }
    return Boolean(data.baseUrl);
  };

  const getFetchDisabledReason = (data: FetchModelsData) => {
    if (data.apiFormat === 'llm_liberty_oauth') {
      if (!data.id && !data.llmLibertyEnvelope) {
        return 'Paste the LLM Liberty envelope to fetch models';
      }
      return '';
    }
    if (!data.baseUrl) {
      return 'You need to add base URL to fetch models';
    }
    return '';
  };

  const buildRequest = (data: FetchModelsData): FetchModelsRequest => {
    if (data.apiFormat === 'llm_liberty_oauth') {
      const aliasId = data.id ?? (mode === 'edit' ? initialData?.id : undefined);
      if (aliasId) {
        return { api_format: 'llm_liberty_oauth', id: aliasId };
      }
      if (!data.llmLibertyEnvelope) {
        throw new Error('LLM Liberty envelope is required to fetch models in create mode');
      }
      return { api_format: 'llm_liberty_oauth', envelope: data.llmLibertyEnvelope };
    }

    let creds: TestCreds;
    if (data.apiKey) {
      creds = { type: 'api_key', value: data.apiKey as ApiKey };
    } else if (mode === 'edit' && initialData?.id) {
      creds = { type: 'id', value: initialData.id };
    } else {
      creds = { type: 'api_key', value: null };
    }

    return {
      api_format: data.apiFormat,
      creds,
      base_url: data.baseUrl,
      ...(data.extraHeaders !== null && data.extraHeaders !== undefined ? { extra_headers: data.extraHeaders } : {}),
      ...(data.extraBody !== null && data.extraBody !== undefined ? { extra_body: data.extraBody } : {}),
    } as FetchModelsRequest;
  };

  const fetchModels = async (data: FetchModelsData) => {
    dismiss();
    setStatus('loading');

    const fetchData = buildRequest(data);

    try {
      const response = await fetchModelsMutation.mutateAsync(fetchData);
      const modelIds = response.data.models;
      setAvailableModels(modelIds);
      setStatus('success');

      if (autoSelectCommon && provider?.commonModels.length && onModelsUpdated) {
        const commonModelsAvailable = provider.commonModels.filter((model) => modelIds.includes(model));
        if (commonModelsAvailable.length > 0) {
          onModelsUpdated(commonModelsAvailable.slice(0, 3));
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
