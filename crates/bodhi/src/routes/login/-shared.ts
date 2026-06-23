import { useNavigate } from '@tanstack/react-router';

import { useLogoutHandler } from '@/hooks/auth';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { ROUTE_DEFAULT, ROUTE_LOGIN } from '@/lib/constants';
import { handleSmartRedirect } from '@/lib/utils';

// Logout wiring shared by both login variants: redirect to the server-provided
// location on success; on failure, hard-clear local/session storage + cookies,
// surface the error, and fall back to the login page.
export function useLogoutWithCleanup() {
  const navigate = useNavigate();
  const { showError } = useToastMessages();

  return useLogoutHandler({
    onSuccess: (response) => {
      const redirectUrl = response.data?.location || ROUTE_DEFAULT;
      handleSmartRedirect(redirectUrl, navigate);
    },
    onError: (message) => {
      localStorage.clear();
      sessionStorage.clear();
      document.cookie.split(';').forEach((c) => {
        const eqPos = c.indexOf('=');
        const name = eqPos > -1 ? c.slice(0, eqPos) : c;
        document.cookie = name + '=;expires=Thu, 01 Jan 1970 00:00:00 GMT;path=/';
      });
      showError('Logout failed', `Message: ${message}. Redirecting to login page.`);
      handleSmartRedirect(ROUTE_LOGIN, navigate);
    },
  });
}
