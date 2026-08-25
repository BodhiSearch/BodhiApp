/** icon is a kebab-case lucide icon name. */
export interface GroupMeta {
  id: string;
  name: string;
  icon: string;
}

export interface SettingGroupConfig {
  /** stable group id (matches the SETTINGS_CONFIG key) */
  id: string;
  /** uppercase section header name */
  name: string;
  label: string;
  icon: string;
  /** keys belonging to this group, in display order */
  keys: string[];
  /** per-key description from the static config (optional) */
  descriptions: Record<string, string | undefined>;
}

export interface SettingsConfigV2 {
  groups: SettingGroupConfig[];
}
