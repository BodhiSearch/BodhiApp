import { useRouter } from 'next/navigation';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Button } from '@/components/ui/button';
import { ChevronDown } from 'lucide-react';
import { useLogout } from '@/hooks/useQuery';
import { useToast } from "@/hooks/use-toast";
import { ApiError } from '@/types/models';

interface UserMenuProps {
}

export default function UserMenu({ }: UserMenuProps) {
  const router = useRouter();
  const { toast } = useToast();
  const email = 'user@example.com';

  const { mutate: logout, isLoading } = useLogout({
    onSuccess: (response) => {
      const redirectUrl = response.headers['location'] || '/ui/home';
      router.push(redirectUrl);
    },
    onError: (error) => {
      console.error('Logout failed:', error);
      let errorMessage = "An unexpected error occurred. Please try again.";

      if (error.response && error.response.data && (error.response.data as ApiError).message) {
        errorMessage = (error.response.data as ApiError).message;
      } else if (typeof error === 'string') {
        errorMessage = error;
      } else if (error instanceof Error) {
        errorMessage = error.message;
      }

      toast({
        variant: "destructive",
        title: "Logout failed",
        description: `Message: ${errorMessage}. Try again later.`,
      });
    }
  });

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="outline" className="w-full justify-between">
          {email} <ChevronDown className="ml-2 h-4 w-4" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        align="end"
        className="w-[calc(100vw-2rem)] sm:w-[200px]"
      >
        <DropdownMenuItem
          onClick={() => router.push('/account-settings')}
          className="justify-center sm:justify-start"
        >
          Account Settings
        </DropdownMenuItem>
        <DropdownMenuItem
          onClick={() => logout()}
          disabled={isLoading}
          className="justify-center sm:justify-start"
        >
          {isLoading ? 'Logging out...' : 'Logout'}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
