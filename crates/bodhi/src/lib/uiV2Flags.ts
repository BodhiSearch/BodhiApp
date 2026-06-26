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
 *   localStorage.setItem('bodhi.ui-v2.chat', 'true')
 *
 * (API-Keys — Batch 1 — Settings — Batch 2 — Models — Batch 3 — and MCP — Batch 4 — all shipped
 * V2-only at the same routes, so their flags were retired. Only Chat remains.)
 */

export type UiV2Screen = 'chat';

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
