import { useRouter } from 'next/navigation';
import { useLogout } from '@/hooks/useQuery';
import { useToast } from '@/hooks/use-toast';
import { ApiError } from '@/types/models';

export function useLogoutHandler() {
  const router = useRouter();
  const { toast } = useToast();

  const { mutate: logout, isLoading } = useLogout({
    onSuccess: (response) => {
      const redirectUrl = response.headers['location'] || '/ui/home';
      router.push(redirectUrl);
    },
    onError: (error) => {
      console.error('Logout failed:', error);
      let errorMessage = 'An unexpected error occurred. Please try again.';

      if (
        error.response &&
        error.response.data &&
        (error.response.data as ApiError).message
      ) {
        errorMessage = (error.response.data as ApiError).message;
      } else if (typeof error === 'string') {
        errorMessage = error;
      } else if (error instanceof Error) {
        errorMessage = error.message;
      }

      toast({
        variant: 'destructive',
        title: 'Logout failed',
        description: `Message: ${errorMessage}. Try again later.`,
      });
    },
  });

  return { logout, isLoading };
}
