import { useState } from 'react';
import { useTestApiModel } from '@/hooks/useApiModels';
import { useToast } from '@/hooks/use-toast';
import { TestPromptRequest, TestCreds, ApiKey } from '@bodhiapp/ts-client';
import { DEFAULT_TEST_PROMPT } from '../providers/constants';

interface UseTestConnectionProps {
  mode?: 'create' | 'edit' | 'setup';
  initialData?: { id: string };
}

interface TestConnectionData {
  apiKey?: string;
  baseUrl: string;
  model: string;
  id?: string;
}

export function useTestConnection({ mode = 'create', initialData }: UseTestConnectionProps = {}) {
  const [status, setStatus] = useState<'idle' | 'testing' | 'success' | 'error'>('idle');
  const testMutation = useTestApiModel();
  const { toast, dismiss } = useToast();

  const canTest = (data: TestConnectionData) => {
    return Boolean(data.baseUrl && data.model);
  };

  const getMissingRequirements = (data: TestConnectionData) => {
    const missing = [];
    if (!data.baseUrl) missing.push('base URL');
    if (!data.model) missing.push('at least one model');
    return `You need to add ${missing.join(', ')} to test connection`;
  };

  const testConnection = async (data: TestConnectionData) => {
    if (!canTest(data)) {
      return;
    }

    dismiss();
    setStatus('testing');

    // Build TestCreds discriminated union based on what's available
    let creds: TestCreds | undefined;

    if (data.apiKey) {
      // Use provided API key directly
      creds = { type: 'api_key' as const, value: data.apiKey as ApiKey };
    } else if (data.id) {
      // Look up stored credentials by ID
      creds = { type: 'id' as const, value: data.id };
    } else {
      // No authentication (public API)
      creds = { type: 'api_key' as const, value: null };
    }

    const testData: TestPromptRequest = {
      creds,
      base_url: data.baseUrl,
      model: data.model,
      prompt: DEFAULT_TEST_PROMPT,
    };

    try {
      const response = await testMutation.mutateAsync(testData);
      if (response.data.success) {
        setStatus('success');
        toast({
          title: 'Connection Test Successful',
          description: response.data.response || 'Test completed',
        });
      } else {
        setStatus('error');
        toast({
          title: 'Connection Test Failed',
          description: response.data.error || 'Unable to connect to the API.',
          variant: 'destructive',
        });
      }
    } catch (error: unknown) {
      setStatus('error');
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

  const resetStatus = () => {
    setStatus('idle');
  };

  return {
    status,
    isLoading: testMutation.isLoading,
    canTest,
    getMissingRequirements,
    testConnection,
    resetStatus,
  };
}
