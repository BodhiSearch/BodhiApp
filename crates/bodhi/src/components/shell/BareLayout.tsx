import { Link } from '@tanstack/react-router';
import { type ReactNode } from 'react';

import { BASE_PATH, ROUTE_CHAT } from '@/lib/constants';
import { ThemeToggle } from '@/components/ThemeToggle';

import './bare-layout.css';

export interface BareLayoutProps {
  children: ReactNode;
}

/**
 * Standalone chrome for full-page flows rendered outside AppShell (OAuth access-request review,
 * future request-access/status pages — see resolveShellRoute `BARE_PREFIXES`). The eventual
 * route-declared layout seam (techdebt.md) only changes how a route picks this, not the component.
 */
export function BareLayout({ children }: BareLayoutProps) {
  return (
    <div className="bare-page" data-testid="bare-layout">
      <header className="bare-topbar">
        <Link className="bare-brand" to={ROUTE_CHAT}>
          <img
            className="bare-brand-mark"
            src={`${BASE_PATH}/bodhi-logo/bodhi-logo-60.svg`}
            alt="Bodhi"
            onError={(e) => {
              (e.currentTarget as HTMLImageElement).style.display = 'none';
            }}
          />
          <span className="bare-brand-text">
            <span className="bare-brand-word">Bodhi</span>
            <span className="bare-brand-sub">AI Operating System</span>
          </span>
        </Link>
        <ThemeToggle />
      </header>
      <main className="bare-main">{children}</main>
    </div>
  );
}
