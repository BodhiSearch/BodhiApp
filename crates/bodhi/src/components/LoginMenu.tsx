'use client';

import { Button } from '@/components/ui/button';
import { useLogoutHandler } from '@/hooks/useLogoutHandler';
import { ENDPOINT_APP_LOGIN, useAppInfo, useUser } from '@/hooks/useQuery';
import Link from 'next/link';

export function LoginMenu() {
  const { data: userInfo, isLoading: userLoading } = useUser();
  const { data: appInfo, isLoading: appLoading } = useAppInfo();
  const { logout, isLoading: isLoggingOut } = useLogoutHandler();

  if (userLoading || appLoading) {
    return null;
  }

  const isNonAuthz = appInfo && !appInfo.authz;

  if (userInfo?.logged_in) {
    return (
      <div className="px-2 py-1.5 flex flex-col items-center gap-1">
        <Button
          variant="destructive"
          className="px-8 border border-destructive hover:border-destructive"
          onClick={() => logout()}
          disabled={isLoggingOut}
        >
          {isLoggingOut ? 'Logging out...' : 'Log Out'}
        </Button>
        <span className="text-xs text-muted-foreground">
          Logged in as {userInfo.email}
        </span>
      </div>
    );
  }

  if (isNonAuthz) {
    return (
      <div className="px-2 py-1.5">
        <Button
          variant="ghost"
          className="w-full justify-start flex flex-col items-start gap-1"
          disabled
        >
          <span>Login</span>
          <span className="text-xs text-muted-foreground">
            Non-Authenticated Mode Setup
          </span>
        </Button>
      </div>
    );
  }

  return (
    <div className="px-2 py-1.5 flex flex-col items-center">
      <Link href={ENDPOINT_APP_LOGIN} passHref>
        <Button
          variant="default"
          className="px-8 border border-primary hover:border-primary"
        >
          Login
        </Button>
      </Link>
    </div>
  );
}
