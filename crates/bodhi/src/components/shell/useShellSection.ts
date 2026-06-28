import { useLocation, useMatches } from '@tanstack/react-router';

import { resolveShellRoute } from './resolveShellRoute';

/**
 * Route-declared shell chrome. A screen sets `staticData: { section, subPage }` on its
 * `createFileRoute(...)`; the persistent `<AppShell>` reads it via `useShellSection()` to drive the
 * primary-nav highlight. This is the idiomatic TanStack way to carry STATIC chrome (fixed at route
 * creation) — dynamic, screen-state-dependent chrome (rail/sidebar/header actions) goes through
 * `ShellChromeContext` instead.
 */
declare module '@tanstack/react-router' {
  interface StaticDataRouteOption {
    section?: string;
    subPage?: string | null;
  }
}

export interface ShellSection {
  section: string;
  subPage: string | null;
}

/**
 * Resolves the active section/subPage from the matched route chain — the deepest match that
 * declares a `section` wins (so a sub-page route can refine its parent's section).
 *
 * Migration fallback: routes that don't yet declare `staticData.section` fall back to the
 * pathname-derived `resolveShellRoute()`. This keeps the nav highlight correct section-by-section
 * as routes adopt `staticData`; remove the fallback once every section is migrated.
 */
export function useShellSection(): ShellSection {
  const matches = useMatches();
  const { pathname } = useLocation();
  for (let i = matches.length - 1; i >= 0; i--) {
    const data = matches[i].staticData;
    if (data?.section) return { section: data.section, subPage: data.subPage ?? null };
  }
  return resolveShellRoute(pathname) ?? { section: '', subPage: null };
}
