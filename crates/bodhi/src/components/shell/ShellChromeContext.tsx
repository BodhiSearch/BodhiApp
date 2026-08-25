import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from 'react';

import type { ShellBreadcrumbProps } from './ShellChrome';

/**
 * The chrome seam for the persistent `<AppShell>`.
 *
 * One `<AppShell>` mounted by the `_app` layout route persists across navigations (it owns
 * collapse/resize state and must not remount). A screen's rich chrome (breadcrumb, header actions,
 * sidebar, detail rail) must render in the shell's columns, outside the screen's subtree — so it
 * publishes chrome up through this context via `useShellChrome(...)`, which `_app` consumes.
 * TanStack Router has no named outlets, and `staticData`/`useMatches()` only carries static chrome.
 *
 * Re-render discipline (vercel-react-best-practices): setter and value live in separate contexts —
 * publishers subscribe only to the stable setter, never re-rendering on value change; only `_app`
 * subscribes to the value. Screens must pass stable slot nodes (module-scope or memoized), never
 * inline component definitions.
 */
export interface ShellSlots {
  breadcrumb?: ShellBreadcrumbProps['items'];
  headerActions?: ReactNode;
  /** page-body sidebar (below the nav) — e.g. App Settings' settings-group scroll-spy nav. */
  sidebar?: ReactNode;
  rail?: ReactNode;
  railHeader?: ReactNode;
  railDefaultOpen?: boolean;
  // Layout overrides for screens needing a non-default shell (e.g. Chat's own scroll + wider rail);
  // omitted → AppShell default. Spread onto `<AppShell>` by the root shell (see `__root.tsx`).
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

// Screen-side: publish chrome slots to the persistent shell for the lifetime of the screen,
// clearing them on unmount (so navigating away resets the chrome).
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
