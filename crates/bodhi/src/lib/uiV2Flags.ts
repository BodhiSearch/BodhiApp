/**
 * Per-screen UI V2 migration flags.
 *
 * During the screen-by-screen migration to the new AppShell design, each screen can render
 * its OLD or NEW (v2) version at the SAME route, gated by a per-screen flag. Flags are
 * localStorage-backed (a dev/dogfooding toggle), default OFF (old). A screen's flag is
 * removed once its v2 version is accepted and the old code deleted (see the migration
 * playbook in docs/claude-plans/202606/screen-v2/).
 *
 * Toggle from the browser console, e.g.:
 *   localStorage.setItem('bodhi.ui-v2.app-tokens', 'true')
 */

export type UiV2Screen =
  | 'chat'
  | 'models'
  | 'new-local-model'
  | 'new-api-model'
  | 'new-fallback-model'
  | 'mcp-discover'
  | 'new-mcp'
  | 'mcp-playground'
  | 'app-tokens'
  | 'new-token'
  | 'access-requests'
  | 'access-request-review'
  | 'app-settings'
  | 'manage-users';

export const UI_V2_FLAG_PREFIX = 'bodhi.ui-v2.';

const flagKey = (screen: UiV2Screen): string => `${UI_V2_FLAG_PREFIX}${screen}`;

/** Reads a screen's UI V2 flag from localStorage. Default false (render the old screen). */
export function isUiV2Enabled(screen: UiV2Screen): boolean {
  if (typeof window === 'undefined') return false;
  try {
    return localStorage.getItem(flagKey(screen)) === 'true';
  } catch {
    return false;
  }
}
