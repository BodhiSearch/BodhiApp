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
import { useUserContext } from '@/hooks/useUserContext';
import { useLogoutHandler } from '@/hooks/useLogoutHandler';

export default function LoginContent() {
  const { userInfo, isLoading } = useUserContext();
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
              <Link href="/app/home" passHref>
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
            <Link href="/app/login" passHref>
              <Button className="w-full">Log In</Button>
            </Link>
          )}
        </CardContent>
      </Card>
    </>
  );
}
