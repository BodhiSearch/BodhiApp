import { useLogout } from '@/hooks/useQuery';

interface UseLogoutHandlerOptions {
  onSuccess?: (response: any) => void;
  onError?: (message: string) => void;
}

export function useLogoutHandler(options?: UseLogoutHandlerOptions) {
  const { mutate: logout, isLoading } = useLogout({
    onSuccess: (response) => {
      options?.onSuccess?.(response);
    },
    onError: (error) => {
      console.error('Logout failed:', error);
      const errorMessage =
        error.response?.data?.error?.message || error.message || 'An unexpected error occurred. Please try again.';
      options?.onError?.(errorMessage);
    },
  });

  return { logout, isLoading };
}
