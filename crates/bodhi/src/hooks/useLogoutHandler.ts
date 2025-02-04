import { useToastMessages } from '@/hooks/use-toast-messages';
import { useLogout } from '@/hooks/useQuery';
import { ROUTE_DEFAULT } from '@/lib/constants';
import { useRouter } from 'next/navigation';

export function useLogoutHandler() {
  const router = useRouter();
  const { showError } = useToastMessages();

  const { mutate: logout, isLoading } = useLogout({
    onSuccess: (response) => {
      const redirectUrl = response.headers['location'] || ROUTE_DEFAULT;
      router.push(redirectUrl);
    },
    onError: (error) => {
      console.error('Logout failed:', error);
      const errorMessage =
        error.response?.data?.error?.message ||
        error.message ||
        'An unexpected error occurred. Please try again.';
      showError('Logout failed', `Message: ${errorMessage}. Try again later.`);
    },
  });

  return { logout, isLoading };
}
