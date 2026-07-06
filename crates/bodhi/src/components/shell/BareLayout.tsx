import { type ReactNode } from 'react';

import { BASE_PATH, ROUTE_CHAT } from '@/lib/constants';
import { ThemeToggle } from '@/components/ThemeToggle';

import './bare-layout.css';

export interface BareLayoutProps {
  children: ReactNode;
}

/**
 * Standalone (bare) chrome for in-app full-page flows rendered OUTSIDE the AppShell:
 * a slim branded top bar + a centered, scrollable content region. The OAuth
 * access-request review uses it; future request-access / status pages reuse it by
 * rendering bare (see resolveShellRoute `BARE_PREFIXES`). The eventual route-declared
 * layout seam (techdebt.md) is a drop-in — it only changes how a route is chosen to
 * render bare, not this component.
 */
export function BareLayout({ children }: BareLayoutProps) {
  return (
    <div className="bare-page" data-testid="bare-layout">
      <header className="bare-topbar">
        <a className="bare-brand" href={ROUTE_CHAT}>
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
        </a>
        <ThemeToggle />
      </header>
      <main className="bare-main">{children}</main>
    </div>
  );
}
