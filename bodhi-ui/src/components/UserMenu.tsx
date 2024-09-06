import { useRouter } from 'next/navigation';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { Button } from "@/components/ui/button";
import { ChevronDown } from "lucide-react";

interface UserMenuProps {
  email: string;
  onLogout: () => void;
}

export default function UserMenu({ email, onLogout }: UserMenuProps) {
  const router = useRouter();

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="outline" className="w-full justify-between">
          {email} <ChevronDown className="ml-2 h-4 w-4" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align='end' className="w-[calc(100vw-2rem)] sm:w-[200px]">
        <DropdownMenuItem
          onClick={() => router.push('/account-settings')}
          className="justify-center sm:justify-start"
        >
          Account Settings
        </DropdownMenuItem>
        <DropdownMenuItem
          onClick={onLogout}
          className="justify-center sm:justify-start"
        >
          Logout
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
