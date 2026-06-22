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
 *   localStorage.setItem('bodhi.ui-v2.mcp-discover', 'true')
 *
 * (The API-Keys screens — Batch 1 — and the Settings screens — App Settings, Manage Users,
 * Batch 2 — have shipped V2-only; their flags were retired, so they're no longer listed. The
 * My Models list — Batch 3-1 — and the New/Edit API Model form — Batch 3-2 — and the New/Edit Model
 * Router form — Batch 3-3 — also shipped V2-only: the list replaced the legacy table at the same
 * route, the forms' V2 chrome/rail is additive over the same routes, so none have a flag.)
 */

export type UiV2Screen = 'chat' | 'new-local-model' | 'mcp-discover' | 'new-mcp' | 'mcp-playground';

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
