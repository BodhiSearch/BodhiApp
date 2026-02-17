import React from 'react';
import { useLocation, useNavigate, NavLink } from 'react-router-dom';
import { Button } from '@/components/ui';
import { useAuth } from '@/context/AuthContext';
import { clearAll, loadToken } from '@/lib/storage';

function decodeEmailFromJwt(token: string): string | null {
  try {
    const parts = token.split('.');
    if (parts.length < 2) return null;
    const padded = parts[1].replace(/-/g, '+').replace(/_/g, '/');
    const decoded = atob(padded);
    const payload = JSON.parse(decoded);
    return payload.email || payload.preferred_username || null;
  } catch {
    return null;
  }
}

const authenticatedPaths = ['/dashboard', '/chat', '/rest'];

export function AppLayout({ children }: { children: React.ReactNode }) {
  const location = useLocation();
  const navigate = useNavigate();
  const { token, setToken } = useAuth();

  const effectiveToken = token || loadToken();
  const isAuthenticated = !!effectiveToken;
  const showNav = authenticatedPaths.includes(location.pathname) && isAuthenticated;

  const handleLogout = () => {
    clearAll();
    setToken(null);
    navigate('/', { replace: true });
  };

  const userEmail = effectiveToken ? decodeEmailFromJwt(effectiveToken) : null;

  return (
    <div className="flex flex-col min-h-screen">
      <header className="h-12 border-b bg-card flex items-center px-4 shrink-0">
        {/* Left: App branding */}
        <div className="font-semibold text-sm">OAuth2 Test App</div>

        {/* Center: Navigation (if authenticated and on nav pages) */}
        {showNav && (
          <nav className="flex-1 flex justify-center gap-4">
            <NavLink
              to="/dashboard"
              data-testid="nav-dashboard"
              className={({ isActive }) =>
                `text-sm px-2 py-1 ${isActive ? 'font-semibold text-foreground' : 'text-muted-foreground hover:text-foreground'}`
              }
            >
              Dashboard
            </NavLink>
            <NavLink
              to="/chat"
              data-testid="nav-chat"
              className={({ isActive }) =>
                `text-sm px-2 py-1 ${isActive ? 'font-semibold text-foreground' : 'text-muted-foreground hover:text-foreground'}`
              }
            >
              Chat
            </NavLink>
            <NavLink
              to="/rest"
              data-testid="nav-rest"
              className={({ isActive }) =>
                `text-sm px-2 py-1 ${isActive ? 'font-semibold text-foreground' : 'text-muted-foreground hover:text-foreground'}`
              }
            >
              REST
            </NavLink>
          </nav>
        )}

        {!showNav && <div className="flex-1" />}

        {/* Right: User email and logout (if authenticated) */}
        {isAuthenticated && (
          <div className="flex items-center gap-3">
            <span data-testid="header-user-email" className="text-sm text-muted-foreground">
              {userEmail}
            </span>
            <Button
              data-testid="btn-header-logout"
              variant="destructive"
              size="sm"
              onClick={handleLogout}
            >
              Logout
            </Button>
          </div>
        )}
      </header>
      <main className="flex-1 flex justify-center">
        {children}
      </main>
    </div>
  );
}
