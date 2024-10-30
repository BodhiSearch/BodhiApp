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
import { useUser } from '@/hooks/useQuery';
import Link from 'next/link';
import { path_app_login } from '@/lib/utils';

interface UserMenuProps {}

export default function UserMenu({}: UserMenuProps) {
  const router = useRouter();
  const { data: userInfo, isLoading, error } = useUser();
  const { logout, isLoading: isLoadingLogout } = useLogoutHandler();

  if (isLoading || error || !userInfo) return <div></div>;
  if (!userInfo?.logged_in) {
    return (
      <Link href={path_app_login} passHref>
        <Button variant="outline" className="w-full justify-between">
          Log In
        </Button>
      </Link>
    );
  }
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="outline" className="w-full justify-between">
          {userInfo?.email} <ChevronDown className="ml-2 h-4 w-4" />
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
          disabled={isLoadingLogout}
          className="justify-center sm:justify-start"
        >
          {isLoadingLogout ? 'Logging out...' : 'Logout'}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
