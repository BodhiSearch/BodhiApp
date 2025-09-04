import { useLogout } from '@/hooks/useQuery';
import { AxiosResponse, AxiosError } from 'axios';
import { OpenAiApiError, RedirectResponse } from '@bodhiapp/ts-client';

// Type alias for compatibility
type ErrorResponse = OpenAiApiError;

interface UseLogoutHandlerOptions {
  onSuccess?: (response: AxiosResponse<RedirectResponse>) => void;
  onError?: (message: string) => void;
}

export function useLogoutHandler(options?: UseLogoutHandlerOptions) {
  const { mutate: logout, isLoading } = useLogout({
    onSuccess: (response: AxiosResponse<RedirectResponse>) => {
      options?.onSuccess?.(response);
    },
    onError: (error: AxiosError<ErrorResponse>) => {
      console.error('Logout failed:', error);
      const errorMessage =
        error.response?.data?.error?.message || error.message || 'An unexpected error occurred. Please try again.';
      options?.onError?.(errorMessage);
    },
  });

  return { logout, isLoading };
}
