import { useRouter } from 'next/navigation';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Button } from '@/components/ui/button';
import { ChevronDown } from 'lucide-react';
import { useLogoutHandler } from '@/hooks/useLogoutHandler';

interface UserMenuProps {}

export default function UserMenu({}: UserMenuProps) {
  const router = useRouter();
  const email = 'user@example.com';
  const { logout, isLoading } = useLogoutHandler();

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
