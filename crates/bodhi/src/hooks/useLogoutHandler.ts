import { useRouter } from 'next/navigation';
import { useLogout } from '@/hooks/useQuery';
import { useToast } from '@/hooks/use-toast';

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
      const errorMessage =
        error.response?.data?.error?.message ||
        error.message ||
        'An unexpected error occurred. Please try again.';
      toast({
        variant: 'destructive',
        title: 'Logout failed',
        description: `Message: ${errorMessage}. Try again later.`,
      });
    },
  });

  return { logout, isLoading };
}
