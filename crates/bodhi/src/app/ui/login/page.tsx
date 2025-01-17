'use client';

import AppInitializer from '@/components/AppInitializer';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { useLogoutHandler } from '@/hooks/useLogoutHandler';
import { ENDPOINT_APP_LOGIN, useAppInfo, useUser } from '@/hooks/useQuery';
import { PATH_UI_HOME } from '@/lib/utils';
import Link from 'next/link';

export function LoginContent() {
  const { data: userInfo, isLoading: userLoading } = useUser();
  const { data: appInfo, isLoading: appLoading } = useAppInfo();
  const { logout, isLoading: isLoggingOut } = useLogoutHandler();

  if (userLoading || appLoading) {
    return (
      <div className="w-full max-w-md mx-auto mt-8 h-fit text-center">
        <div className="text-center">Loading...</div>
      </div>
    );
  }

  const isNonAuthz = appInfo && !appInfo.authz;

  return (
    <div className="w-full max-w-md mx-auto mt-8 h-fit text-center">
      {userInfo?.logged_in ? (
        <Card>
          <CardHeader className="text-center">
            <CardTitle>Welcome</CardTitle>
            <CardDescription>
              You are logged in as {userInfo.email}
            </CardDescription>
          </CardHeader>
          <CardContent>
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
          </CardContent>
        </Card>
      ) : (
        <Card>
          <CardHeader className="text-center">
            <CardTitle>Login</CardTitle>
            {!isNonAuthz && (
              <CardDescription>Login to use the Bodhi App</CardDescription>
            )}
          </CardHeader>
          <CardContent>
            <div
              className={`${isNonAuthz ? 'opacity-50 pointer-events-none' : ''}`}
            >
              {isNonAuthz && (
                <div className="mb-4 text-sm text-muted-foreground">
                  This app is setup in non-authenticated mode.
                  <br />
                  User login is not available.
                </div>
              )}
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
          </CardContent>
        </Card>
      )}
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
