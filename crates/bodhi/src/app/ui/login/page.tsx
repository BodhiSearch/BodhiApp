'use client';

import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import Link from 'next/link';
import { useLogoutHandler } from '@/hooks/useLogoutHandler';
import AppInitializer from '@/components/AppInitializer';
import { useUser } from '@/hooks/useQuery';
import { PATH_UI_HOME } from '@/lib/utils';
import { ENDPOINT_APP_LOGIN } from '@/hooks/useQuery';
import { useAppInfo } from '@/hooks/useQuery';

export function LoginContent() {
  const { data: userInfo, isLoading: userLoading } = useUser();
  const { data: appInfo, isLoading: appLoading } = useAppInfo();
  const { logout, isLoading: isLoggingOut } = useLogoutHandler();

  if (userLoading || appLoading) {
    return <div className="text-center">Loading...</div>;
  }

  const isNonAuthz = appInfo && !appInfo.authz;
  const loginTitle = userInfo?.logged_in ? 'Welcome' : 'Login';
  const loginMessage = userInfo?.logged_in ? (
    `You are logged in as ${userInfo?.email}`
  ) : isNonAuthz ? (
    <>
      This app is setup in non-authenticated mode.
      <br />
      User login is not available.
    </>
  ) : (
    'You need to login to use the Bodhi App'
  );

  return (
    <div className="w-full max-w-md mx-auto mt-8 h-fit text-center">
      <Card>
        <CardHeader className="text-center">
          <CardTitle>{loginTitle}</CardTitle>
          <CardDescription>{loginMessage}</CardDescription>
        </CardHeader>
        <CardContent>
          {userInfo?.logged_in ? (
            <>
              <Link href={PATH_UI_HOME} passHref>
                <Button className="w-full mb-2" variant="secondary">
                  Go to Home
                </Button>
              </Link>
              <Button
                className="w-full"
                onClick={() => logout()}
                disabled={isLoggingOut}
              >
                {isLoggingOut ? 'Logging out...' : 'Log Out'}
              </Button>
            </>
          ) : (
            <div
              className={`${isNonAuthz ? 'opacity-50 pointer-events-none' : ''}`}
            >
              <Link href={ENDPOINT_APP_LOGIN} passHref>
                <Button
                  className="w-full"
                  variant="default"
                  disabled={isNonAuthz}
                >
                  Login
                </Button>
              </Link>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

export default function LoginPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={false}>
      <LoginContent />
    </AppInitializer>
  );
}
