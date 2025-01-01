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
import Logo from '@/components/Logo';
import { useLogoutHandler } from '@/hooks/useLogoutHandler';
import AppInitializer from '@/components/AppInitializer';
import { useUser } from '@/hooks/useQuery';
import { path_app_login, path_home } from '@/lib/utils';

function LoginContent() {
  const { data: userInfo, isLoading } = useUser();
  const { logout, isLoading: isLoggingOut } = useLogoutHandler();

  if (isLoading) {
    return <div className="text-center">Loading...</div>;
  }

  return (
    <>
      <div className="my-6">
        <Logo />
      </div>
      <Card className="w-full max-w-md mx-auto mt-10">
        <CardHeader className="text-center">
          <CardTitle>{userInfo?.logged_in ? 'Welcome' : 'Login'}</CardTitle>
          <CardDescription>
            {userInfo?.logged_in
              ? `You are logged in as ${userInfo?.email}`
              : 'You need to login to use the Bodhi App'}
          </CardDescription>
        </CardHeader>
        <CardContent>
          {userInfo?.logged_in ? (
            <>
              <Link href={path_home} passHref>
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
            <Link href={path_app_login} passHref>
              <Button className="w-full">Log In</Button>
            </Link>
          )}
        </CardContent>
      </Card>
    </>
  );
}

export default function LoginPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={false}>
      <LoginContent />
    </AppInitializer>
  );
}
