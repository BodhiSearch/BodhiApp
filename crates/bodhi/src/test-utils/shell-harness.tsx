import { useCallback, useMemo, useState, type ReactNode } from 'react';

import { ShellChromeProvider, useShellSlots } from '@/components/shell/ShellChromeContext';
import { ShellContext, type ShellContextValue } from '@/components/shell/ShellContext';

// Renders published chrome slots (breadcrumb, sidebar, rail) into stable harness-* testids for assertions.
// Provides a real ShellContext so rail-consuming screens get working openRail/closeRail/collapseRail.
export function ChromeProbe() {
  const { breadcrumb, headerActions, sidebar, rail, railHeader } = useShellSlots();
  const crumbs = Array.isArray(breadcrumb) ? breadcrumb.map((b) => b.label).join(' / ') : '';
  return (
    <>
      <div data-testid="harness-breadcrumb">{crumbs}</div>
      <div data-testid="harness-header-actions">{headerActions}</div>
      <div data-testid="harness-sidebar">{sidebar}</div>
      <div data-testid="harness-rail-header">{railHeader}</div>
      <div data-testid="harness-rail">{rail}</div>
    </>
  );
}

// ShellContext backed by local state so rail open/close behaves in tests.
function WiredShellContext({ children }: { children: ReactNode }) {
  const [openPop, setOpenPop] = useState<string | null>(null);
  const [, setRailOpen] = useState(false);
  const openRail = useCallback(() => setRailOpen(true), []);
  const closeRail = useCallback(() => setRailOpen(false), []);
  const collapseRail = useCallback(() => setRailOpen(false), []);
  const value: ShellContextValue = useMemo(
    () => ({ collapsed: false, isMobile: false, openPop, setOpenPop, openRail, closeRail, collapseRail }),
    [openPop, openRail, closeRail, collapseRail]
  );
  return <ShellContext.Provider value={value}>{children}</ShellContext.Provider>;
}

// renderProbe: false for router-based tests that render ChromeProbe INSIDE the router (shares screen context).
export function ShellHarness({ children, renderProbe = true }: { children: ReactNode; renderProbe?: boolean }) {
  return (
    <ShellChromeProvider>
      <WiredShellContext>
        {renderProbe && <ChromeProbe />}
        {children}
      </WiredShellContext>
    </ShellChromeProvider>
  );
}
