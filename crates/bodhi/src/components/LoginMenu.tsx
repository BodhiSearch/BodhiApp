import { Button } from '@/components/ui/button';
import { useLogoutHandler } from '@/hooks/useLogoutHandler';
import { useOAuthInitiate } from '@/hooks/useOAuth';
import { useUser } from '@/hooks/useQuery';

export function LoginMenu() {
  const { data: userInfo, isLoading: userLoading } = useUser();
  const { logout, isLoading: isLoggingOut } = useLogoutHandler();
  const oauthInitiate = useOAuthInitiate();

  if (userLoading) {
    return null;
  }

  const handleLogin = () => {
    oauthInitiate.mutate();
  };

  if (userInfo?.logged_in) {
    return (
      <div className="p-2 space-y-1.5 text-center" data-testid="login-menu-logged-in">
        <p className="text-xs text-muted-foreground">Logged in as {userInfo.email}</p>
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
