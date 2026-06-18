import { createFileRoute } from '@tanstack/react-router';

import AppInitializer from '@/components/AppInitializer';

import { SettingGroupConfig, SettingsConfigV2, SettingsPageV2 } from './-components/SettingsPageV2';

export const Route = createFileRoute('/settings/')({
  component: SettingsPage,
});

type SettingConfig = {
  key: string;
  description?: string;
};

type GroupConfig = {
  /** uppercase section header name */
  title: string;
  /** short sidebar label */
  label: string;
  /** kebab lucide icon name */
  icon: string;
  settings: SettingConfig[];
};

type SettingsConfig = {
  [key: string]: GroupConfig;
};

/**
 * Static group → keys mapping (titles, sidebar labels, kebab icons, per-key descriptions). A
 * setting's value/source/metadata comes from the API; this only supplies grouping + copy. Settings
 * not listed here fall into the data-driven "Variant Args" / "Miscellaneous" groups (see
 * SettingsPageV2). Editability is enforced by the backend EDIT_SETTINGS_ALLOWED list, mirrored as
 * EDITABLE_KEYS in SettingsPageV2.
 */
const SETTINGS_CONFIG: SettingsConfig = {
  app: {
    title: 'App Configuration',
    label: 'App Config',
    icon: 'settings',
    settings: [{ key: 'BODHI_HOME', description: 'The home directory for Bodhi application.' }],
  },
  model: {
    title: 'Model Files Configuration',
    label: 'Model Files',
    icon: 'database',
    settings: [{ key: 'HF_HOME', description: 'Home directory for Hugging Face model files.' }],
  },
  execution: {
    title: 'Llama.cpp Executable Configuration',
    label: 'Llama.cpp Exec',
    icon: 'terminal',
    settings: [
      { key: 'BODHI_EXEC_LOOKUP_PATH', description: 'Path to look for Llama.cpp executables.' },
      { key: 'BODHI_EXEC_TARGET', description: 'Target platform for llama.cpp executable.' },
      { key: 'BODHI_EXEC_VARIANT', description: 'Optimized hardware specific variant of llama.cpp to use.' },
      { key: 'BODHI_EXEC_NAME', description: 'Name of the llama.cpp executable.' },
      { key: 'BODHI_EXEC_VARIANTS', description: 'Available llama.cpp variants for this platform.' },
      { key: 'BODHI_LLAMACPP_ARGS', description: 'Common arguments passed to all llama.cpp server instances.' },
      {
        key: 'BODHI_KEEP_ALIVE_SECS',
        description: 'Keep alive timeout for llama-server (in seconds). range 300 (5 mins)..=86400 (1 day)',
      },
    ],
  },
  server: {
    title: 'Server Configuration',
    label: 'Server Config',
    icon: 'server',
    settings: [
      { key: 'BODHI_SCHEME', description: 'Scheme used for server connection (http/https).' },
      { key: 'BODHI_HOST', description: 'Host address for the server.' },
      { key: 'BODHI_PORT', description: 'Port number for the server.' },
    ],
  },
  publicServer: {
    title: 'Public Server Configuration',
    label: 'Public Server',
    icon: 'globe',
    settings: [
      { key: 'BODHI_PUBLIC_SCHEME', description: 'Public scheme used for external connections (http/https).' },
      { key: 'BODHI_PUBLIC_HOST', description: 'Public host address for external access.' },
      { key: 'BODHI_PUBLIC_PORT', description: 'Public port number for external access.' },
      { key: 'BODHI_CANONICAL_REDIRECT', description: 'Enable canonical URL redirects.' },
    ],
  },
  logging: {
    title: 'Logging Configuration',
    label: 'Logging',
    icon: 'file-text',
    settings: [
      { key: 'BODHI_LOGS', description: 'Directory for log files.' },
      { key: 'BODHI_LOG_LEVEL', description: 'Level of logging (e.g., info, debug).' },
      { key: 'BODHI_LOG_STDOUT', description: 'Whether to log to standard output.' },
    ],
  },
  dev: {
    title: 'Development Settings',
    label: 'Development',
    icon: 'code',
    settings: [
      { key: 'BODHI_VERSION', description: 'Version of app' },
      { key: 'BODHI_ENV_TYPE', description: 'Environment type of app' },
      { key: 'BODHI_APP_TYPE', description: 'App flavour' },
      { key: 'BODHI_COMMIT_SHA', description: 'Git commit SHA of the current build' },
    ],
  },
  auth: {
    title: 'Authentication Configuration',
    label: 'Authentication',
    icon: 'shield',
    settings: [
      { key: 'BODHI_AUTH_URL', description: 'Authentication server URL.' },
      { key: 'BODHI_AUTH_REALM', description: 'Authentication realm identifier.' },
    ],
  },
};

/** Project the static SETTINGS_CONFIG into the V2 shape (dynamic groups are added at render). */
function buildSettingsConfigV2(config: SettingsConfig): SettingsConfigV2 {
  const groups: SettingGroupConfig[] = Object.entries(config).map(([id, group]) => ({
    id,
    name: group.title,
    label: group.label,
    icon: group.icon,
    keys: group.settings.map((s) => s.key),
    descriptions: Object.fromEntries(group.settings.map((s) => [s.key, s.description])),
  }));
  return { groups };
}

const SETTINGS_CONFIG_V2 = buildSettingsConfigV2(SETTINGS_CONFIG);

function SettingsPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <SettingsPageV2 config={SETTINGS_CONFIG_V2} />
    </AppInitializer>
  );
}
