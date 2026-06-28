import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from 'react';

import type { ShellBreadcrumbProps } from './ShellChrome';

/**
 * The chrome seam for the persistent `<AppShell>`.
 *
 * One `<AppShell>` is mounted by the `_app` layout route and persists across app-route
 * navigations (it owns collapse/resize state and the localStorage width-restore effect, so it
 * must not remount). A screen renders inside that shell's `<Outlet/>`, but its rich chrome
 * (breadcrumb, header actions, page sidebar, detail rail) must appear in the shell's columns —
 * OUTSIDE the screen's subtree. Since a child can't prop-drill into an ancestor, the screen
 * publishes its chrome up through this context via `useShellChrome(...)` and `_app` consumes it.
 *
 * This is the idiomatic React mechanism for state-dependent chrome into a persistent ancestor
 * layout — TanStack Router has no named outlets, and `staticData`/`useMatches()` carries only
 * STATIC chrome (section/subPage), not live nodes like a detail rail.
 *
 * Re-render discipline (vercel-react-best-practices): the SETTER and the VALUE live in two
 * separate contexts. Publishers subscribe only to the stable setter (never re-render on value
 * change); only `_app` subscribes to the value. Screens must pass STABLE slot nodes
 * (module-scope or memoized) — never inline component definitions (`rerender-no-inline-components`).
 */
export interface ShellSlots {
  breadcrumb?: ShellBreadcrumbProps['items'];
  headerActions?: ReactNode;
  /** page-body sidebar (below the nav) — e.g. App Settings' settings-group scroll-spy nav. */
  sidebar?: ReactNode;
  rail?: ReactNode;
  railHeader?: ReactNode;
  railDefaultOpen?: boolean;
  /**
   * Layout overrides for screens that need a non-default shell (e.g. Chat owns its own
   * conversation/composer scroll and a wider rail). Each is optional; omitted → AppShell default.
   * The root shell spreads these onto `<AppShell>` (see `__root.tsx`).
   */
  mainScroll?: boolean;
  railScroll?: boolean;
  contentClass?: string;
  railWidth?: number;
  sidebarWidth?: number;
  resizeKey?: string;
}

const EMPTY_SLOTS: ShellSlots = {};

type SetSlots = (slots: ShellSlots | null) => void;

const ShellSlotsValueContext = createContext<ShellSlots>(EMPTY_SLOTS);
const ShellSlotsSetContext = createContext<SetSlots>(() => {});

export function ShellChromeProvider({ children }: { children: ReactNode }) {
  const [slots, setSlots] = useState<ShellSlots>(EMPTY_SLOTS);

  // Stable setter: identity never changes, so publishers never re-render from this context.
  const set = useCallback<SetSlots>((next) => setSlots(next ?? EMPTY_SLOTS), []);

  return (
    <ShellSlotsSetContext.Provider value={set}>
      <ShellSlotsValueContext.Provider value={slots}>{children}</ShellSlotsValueContext.Provider>
    </ShellSlotsSetContext.Provider>
  );
}

/** `_app` layout read of the currently published slots. */
export function useShellSlots(): ShellSlots {
  return useContext(ShellSlotsValueContext);
}

/**
 * Screen-side: publish chrome slots to the persistent shell for the lifetime of the screen, clearing
 * them on unmount (so navigating away resets the chrome). Pass stable nodes; the publish effect
 * keys on the individual slot fields.
 */
export function useShellChrome(slots: ShellSlots): void {
  const setSlots = useContext(ShellSlotsSetContext);
  const {
    breadcrumb,
    headerActions,
    sidebar,
    rail,
    railHeader,
    railDefaultOpen,
    mainScroll,
    railScroll,
    contentClass,
    railWidth,
    sidebarWidth,
    resizeKey,
  } = slots;

  // Re-publish whenever any individual slot changes; stable nodes keep this from thrashing.
  const next = useMemo<ShellSlots>(
    () => ({
      breadcrumb,
      headerActions,
      sidebar,
      rail,
      railHeader,
      railDefaultOpen,
      mainScroll,
      railScroll,
      contentClass,
      railWidth,
      sidebarWidth,
      resizeKey,
    }),
    [
      breadcrumb,
      headerActions,
      sidebar,
      rail,
      railHeader,
      railDefaultOpen,
      mainScroll,
      railScroll,
      contentClass,
      railWidth,
      sidebarWidth,
      resizeKey,
    ]
  );

  useEffect(() => {
    setSlots(next);
    return () => setSlots(null);
  }, [setSlots, next]);
}
