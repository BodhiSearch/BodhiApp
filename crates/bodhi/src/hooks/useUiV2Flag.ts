import { useLocalStorage } from '@/hooks/useLocalStorage';
import { UI_V2_FLAG_PREFIX, type UiV2Screen } from '@/lib/uiV2Flags';

/**
 * Reactive per-screen UI V2 flag. Returns `[enabled, setEnabled]`, default false (old screen).
 * Backed by localStorage so a toggle persists across reloads and survives navigation.
 * See {@link UiV2Screen} and docs/claude-plans/202606/screen-v2/ for the migration playbook.
 */
export function useUiV2Flag(screen: UiV2Screen): [boolean, (enabled: boolean) => void] {
  return useLocalStorage<boolean>(`${UI_V2_FLAG_PREFIX}${screen}`, false);
}
