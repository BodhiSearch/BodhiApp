import { BASE_PATH } from '@/lib/constants';

/**
 * Layout classification for the root shell: which routes render OUTSIDE the AppShell.
 *
 * - Bare routes (setup wizard, login, auth/oauth callbacks, request-access, root redirectors, and
 *   the standalone OAuth access-request review) render through `BareLayout` (slim topbar).
 * - Fullscreen routes (the setup wizard) render the Outlet directly — no chrome at all.
 *
 * Everything else is an app (shell) route. Section/sub-page highlight for app routes comes from
 * route `staticData` via `useShellSection()`, not from here.
 *
 * INTERIM: this central prefix switch could eventually be replaced by route-declared layout
 * (`staticData.layout` / pathless `_bare` routes); see screen-v2/techdebt.md.
 */
const BARE_PREFIXES = ['/setup', '/login', '/auth', '/request-access', '/mcps/oauth', '/apps/access-requests/review'];

/** Strips the `/ui` basepath and any trailing slash. */
function normalize(pathname: string): string {
  let p = pathname;
  if (p.startsWith(BASE_PATH)) p = p.slice(BASE_PATH.length);
  if (p.length > 1 && p.endsWith('/')) p = p.slice(0, -1);
  return p || '/';
}

export function isBareRoute(pathname: string): boolean {
  const p = normalize(pathname);
  if (p === '/' || p === '/home') return true;
  return BARE_PREFIXES.some((prefix) => p === prefix || p.startsWith(`${prefix}/`));
}

/**
 * Route prefixes that render fullscreen — no chrome at all (no AppShell, no BareLayout slim
 * topbar). The setup wizard owns its entire chrome, so it renders the Outlet directly. Subset of
 * bare routes.
 */
const FULLSCREEN_PREFIXES = ['/setup'];

export function isFullscreenRoute(pathname: string): boolean {
  const p = normalize(pathname);
  return FULLSCREEN_PREFIXES.some((prefix) => p === prefix || p.startsWith(`${prefix}/`));
}
