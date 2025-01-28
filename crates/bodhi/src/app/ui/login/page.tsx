'use client';

import { AuthCard } from '@/components/auth/AuthCard';
import AppInitializer from '@/components/AppInitializer';
import { ENDPOINT_APP_LOGIN, useAppInfo, useUser } from '@/hooks/useQuery';
import { useLogoutHandler } from '@/hooks/useLogoutHandler';
import { PATH_UI_HOME } from '@/lib/utils';

export function LoginContent() {
  const { data: userInfo, isLoading: userLoading } = useUser();
  const { data: appInfo, isLoading: appLoading } = useAppInfo();
  const { logout, isLoading: isLoggingOut } = useLogoutHandler();

  const isNonAuthz = appInfo && !appInfo.authz;

  if (userLoading || appLoading) {
    return <AuthCard title="Loading" isLoading={true} />;
  }

  if (userInfo?.logged_in) {
    return (
      <AuthCard
        title="Welcome"
        description={`You are logged in as ${userInfo.email}`}
        actions={[
          {
            label: 'Go to Home',
            href: PATH_UI_HOME,
            variant: 'secondary',
          },
          {
            label: isLoggingOut ? 'Logging out...' : 'Log Out',
            onClick: () => logout(),
            disabled: isLoggingOut,
            loading: isLoggingOut,
          },
        ]}
      />
    );
  }

  return (
    <AuthCard
      title="Login"
      description={
        isNonAuthz ? (
          <>
            This app is setup in non-authenticated mode.
            <br />
            User login is not available.
          </>
        ) : (
          'Login to use the Bodhi App'
        )
      }
      actions={[
        {
          label: 'Login',
          href: ENDPOINT_APP_LOGIN,
          disabled: isNonAuthz,
        },
      ]}
      disabled={isNonAuthz}
    />
  );
}

export default function LoginPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={false}>
      <div
        className="flex-1 pt-12 sm:pt-16"
        data-testid="login-page"
      >
        <LoginContent />
      </div>
    </AppInitializer>
  );
}
