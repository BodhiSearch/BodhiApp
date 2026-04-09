import { useState } from 'react';

import { TestPromptRequest } from '@bodhiapp/ts-client';

import { useToast } from '@/hooks/use-toast';
import { useTestApiModel } from '@/hooks/models';

interface UseTestConnectionProps {
  mode?: 'create' | 'edit' | 'setup';
  initialData?: { id: string };
}

export function useTestConnection({ mode: _mode = 'create', initialData: _initialData }: UseTestConnectionProps = {}) {
  const [status, setStatus] = useState<'idle' | 'testing' | 'success' | 'error'>('idle');
  const testMutation = useTestApiModel();
  const { toast, dismiss } = useToast();

  const canTest = (data: Pick<TestPromptRequest, 'base_url' | 'model'>) => {
    return Boolean(data.base_url && data.model);
  };

  const getMissingRequirements = (data: Pick<TestPromptRequest, 'base_url' | 'model'>) => {
    const missing = [];
    if (!data.base_url) missing.push('base URL');
    if (!data.model) missing.push('at least one model');
    return `You need to add ${missing.join(', ')} to test connection`;
  };

  const testConnection = async (data: TestPromptRequest) => {
    if (!canTest(data)) {
      return;
    }

    dismiss();
    setStatus('testing');

    try {
      const response = await testMutation.mutateAsync(data);
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
    isLoading: testMutation.isPending,
    canTest,
    getMissingRequirements,
    testConnection,
    resetStatus,
  };
}
