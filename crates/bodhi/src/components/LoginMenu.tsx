import { Button } from '@/components/ui/button';
import { useLogoutHandler } from '@/hooks/useLogoutHandler';
import { useOAuthInitiate } from '@/hooks/useOAuth';
import { useAppInfo, useUser } from '@/hooks/useQuery';

export function LoginMenu() {
  const { data: userInfo, isLoading: userLoading } = useUser();
  const { data: appInfo, isLoading: appLoading } = useAppInfo();
  const { logout, isLoading: isLoggingOut } = useLogoutHandler();
  const oauthInitiate = useOAuthInitiate();

  if (userLoading || appLoading) {
    return null;
  }

  const isNonAuthz = appInfo && !appInfo.authz;

  const handleLogin = () => {
    oauthInitiate.mutate();
  };

  if (userInfo?.logged_in) {
    return (
      <div
        className="p-2 space-y-1.5 text-center"
        data-testid="login-menu-logged-in"
      >
        <p className="text-xs text-muted-foreground">
          Logged in as {userInfo.email}
        </p>
        <Button
          variant="destructive"
          className="w-full border border-destructive"
          onClick={() => logout()}
          disabled={isLoggingOut}
        >
          {isLoggingOut ? 'Logging out...' : 'Log Out'}
        </Button>
      </div>
    );
  }

  if (isNonAuthz) {
    return (
      <div className="p-2" data-testid="login-menu-non-authz">
        <Button
          variant="ghost"
          className="w-full space-y-1 items-start"
          disabled
        >
          <p className="text-xs text-muted-foreground">
            Non-Authenticated Mode Setup
          </p>
        </Button>
      </div>
    );
  }

  return (
    <div className="p-2" data-testid="login-menu-default">
      <Button
        variant="default"
        className="w-full border border-primary"
        onClick={handleLogin}
        disabled={oauthInitiate.isLoading}
      >
        {oauthInitiate.isLoading ? 'Redirecting...' : 'Login'}
      </Button>
    </div>
  );
}
