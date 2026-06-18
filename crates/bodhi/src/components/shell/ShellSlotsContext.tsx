import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from 'react';

import type { ShellBreadcrumbProps } from './ShellChrome';

/**
 * TEMPORARY migration scaffolding (see screen-v2/techdebt.md).
 *
 * During the in-place flag-gated migration, `__root` renders ONE `<AppShell>` and only knows
 * the active section/subPage. A migrated screen needs to contribute rich chrome (breadcrumb,
 * header actions, an optional page sidebar, an optional detail rail) up to that single shell.
 * This context is the seam: a screen publishes its slots via `useShellChrome(...)`; the root
 * shell consumes them.
 *
 * Once every screen is migrated, screens pass props to a per-route `<AppShell>` directly (or we
 * adopt pathless layout routes) and this whole file is deleted.
 *
 * Re-render discipline (vercel-react-best-practices): the SETTER and the VALUE live in two
 * separate contexts. Publishers subscribe only to the stable setter (never re-render on value
 * change); only the root shell subscribes to the value. Screens must pass STABLE slot nodes
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
}

const EMPTY_SLOTS: ShellSlots = {};

type SetSlots = (slots: ShellSlots | null) => void;

const ShellSlotsValueContext = createContext<ShellSlots>(EMPTY_SLOTS);
const ShellSlotsSetContext = createContext<SetSlots>(() => {});

export function ShellSlotsProvider({ children }: { children: ReactNode }) {
  const [slots, setSlots] = useState<ShellSlots>(EMPTY_SLOTS);

  // Stable setter: identity never changes, so publishers never re-render from this context.
  const set = useCallback<SetSlots>((next) => setSlots(next ?? EMPTY_SLOTS), []);

  return (
    <ShellSlotsSetContext.Provider value={set}>
      <ShellSlotsValueContext.Provider value={slots}>{children}</ShellSlotsValueContext.Provider>
    </ShellSlotsSetContext.Provider>
  );
}

/** Root-shell read of the currently published slots. */
export function useShellSlots(): ShellSlots {
  return useContext(ShellSlotsValueContext);
}

/**
 * Screen-side: publish chrome slots to the root shell for the lifetime of the screen, clearing
 * them on unmount (so navigating away resets the chrome). Pass stable nodes; the publish effect
 * keys on the individual slot fields.
 */
export function useShellChrome(slots: ShellSlots): void {
  const setSlots = useContext(ShellSlotsSetContext);
  const { breadcrumb, headerActions, sidebar, rail, railHeader, railDefaultOpen } = slots;

  // Re-publish whenever any individual slot changes; stable nodes keep this from thrashing.
  const next = useMemo<ShellSlots>(
    () => ({ breadcrumb, headerActions, sidebar, rail, railHeader, railDefaultOpen }),
    [breadcrumb, headerActions, sidebar, rail, railHeader, railDefaultOpen]
  );

  useEffect(() => {
    setSlots(next);
    return () => setSlots(null);
  }, [setSlots, next]);
}
