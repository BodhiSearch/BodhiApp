import { useCallback, useMemo, useState, type ReactNode } from 'react';

import { ShellChromeProvider, useShellSlots } from '@/components/shell/ShellChromeContext';
import { ShellContext, type ShellContextValue } from '@/components/shell/ShellContext';

/**
 * Shared test harness for screens that publish chrome via `useShellChrome`. Replaces the per-file
 * `SlotsConsumer` copies: it renders the published slots into stable `harness-*` testids (so a test
 * can assert the breadcrumb / sidebar facets / detail rail) AND provides a real `ShellContext` so
 * rail-consuming screens (ModelsScreenV2, LocalDiscoveryScreen, RouterInfoRail, ChatHistorySidebar)
 * get working `openRail`/`closeRail`/`collapseRail` instead of the no-op default.
 *
 * Usage mirrors the old wrapper:
 *   render(<ShellHarness><MyScreen /></ShellHarness>, { wrapper: createWrapper() });
 *   within(screen.getByTestId('harness-sidebar')).getByTestId('...');
 */

/**
 * Renders the published slots into stable `harness-*` testids. Exported so router-based tests
 * (makeRouteRouter/RouteHarness) can render it INSIDE the in-memory router alongside the screen —
 * the published rail/sidebar nodes are created in the screen's subtree and may depend on context
 * (e.g. QueryClient) that lives there, so they must render in the same tree, not above the router.
 */
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

/** A working ShellContext backed by local state so rail open/close behaves in tests. */
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

/**
 * @param renderProbe Render the `harness-*` probe here (default). Set false for router-based tests
 *   that render `<ChromeProbe/>` INSIDE their in-memory router instead (so published nodes share the
 *   screen's context), avoiding a duplicate render of the rail/sidebar.
 */
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
