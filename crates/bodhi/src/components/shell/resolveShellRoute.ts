import { BASE_PATH } from '@/lib/constants';

import { SHELL_NAV } from './shell-nav-config';

export interface ResolvedShellRoute {
  section: string;
  subPage: string | null;
}

/**
 * Route prefixes that render OUTSIDE the AppShell (bare): setup wizard, login, auth/oauth
 * callbacks, request-access, the root redirectors, and the standalone OAuth access-request
 * review (an in-app full-page consent flow — in scope, rendered via BareLayout). Everything else
 * is a shell (app) route.
 *
 * INTERIM: this central prefix switch is migration scaffolding — a route-declared layout seam
 * (route `staticData.layout` / pathless `_bare` routes) is the deferred follow-up; see
 * screen-v2/techdebt.md.
 */
const BARE_PREFIXES = ['/setup', '/login', '/auth', '/request-access', '/mcps/oauth', '/apps/access-requests/review'];

/** Strips the `/ui` basepath and any trailing slash so we can match against SHELL_NAV hrefs. */
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
 * topbar). The setup wizard owns its entire chrome (centered column, logo, stepper, theme toggle),
 * so it renders the Outlet directly. Subset of bare routes.
 */
const FULLSCREEN_PREFIXES = ['/setup'];

export function isFullscreenRoute(pathname: string): boolean {
  const p = normalize(pathname);
  return FULLSCREEN_PREFIXES.some((prefix) => p === prefix || p.startsWith(`${prefix}/`));
}

/**
 * Resolves the active shell section + sub-page from the current pathname by longest-prefix match
 * against SHELL_NAV. The shell nav highlight is normally prop-driven per route; this derives it
 * centrally for the coexistence root shell, where unmigrated routes don't yet pass props.
 * Returns null for bare routes.
 */
export function resolveShellRoute(pathname: string): ResolvedShellRoute | null {
  if (isBareRoute(pathname)) return null;
  const p = normalize(pathname);

  interface Match {
    section: string;
    subPage: string | null;
    len: number;
  }
  let best: Match | null = null;
  const consider = (section: string, subPage: string | null, href: string) => {
    const h = href.length > 1 && href.endsWith('/') ? href.slice(0, -1) : href;
    if (p !== h && !p.startsWith(`${h}/`)) return;
    // A sub-page wins over a same-length section landing (when they share an href, the sub-page
    // highlight is the more specific/useful one); otherwise longest prefix wins.
    const wins = best === null || h.length > best.len || (h.length === best.len && subPage !== null);
    if (wins) best = { section, subPage, len: h.length };
  };

  for (const item of SHELL_NAV) {
    consider(item.id, null, item.href);
    for (const sub of item.subPages) consider(item.id, sub.id, sub.href);
  }

  if (best !== null) {
    const match: Match = best;
    return { section: match.section, subPage: match.subPage };
  }
  // Authenticated app route not in the nav (e.g. edit pages, model files) — default to no highlight.
  return { section: '', subPage: null };
}
