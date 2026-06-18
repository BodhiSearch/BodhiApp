import { ROUTE_CHAT, ROUTE_MCPS, ROUTE_USERS } from '@/lib/constants';

export interface ShellNavSubPage {
  id: string;
  label: string;
  /** kebab-case lucide icon name. */
  icon?: string;
  href: string;
  badge?: string;
}

export interface ShellNavItem {
  id: string;
  label: string;
  /** kebab-case lucide icon name. */
  icon: string;
  href: string;
  badge?: string;
  subPages: ShellNavSubPage[];
}

export const SHELL_NAV: ShellNavItem[] = [
  { id: 'chat', label: 'Chat', icon: 'message-circle', href: ROUTE_CHAT, subPages: [] },
  {
    id: 'models',
    label: 'Models',
    icon: 'cpu',
    href: '/models/',
    subPages: [
      { id: 'all-models', label: 'All Models', icon: 'globe-2', href: '/models/' },
      { id: 'new-local-model', label: 'New Local Model', icon: 'plus-circle', href: '/models/alias/new/' },
      { id: 'new-api-model', label: 'New API Model', icon: 'plug-zap', href: '/models/api/new/' },
      { id: 'new-fallback-model', label: 'New Fallback Alias', icon: 'route', href: '/models/router/new/' },
    ],
  },
  {
    id: 'mcp',
    label: 'MCP',
    icon: 'plug',
    href: ROUTE_MCPS,
    subPages: [
      { id: 'discover', label: 'All MCPs', icon: 'compass', href: ROUTE_MCPS },
      { id: 'new-mcp', label: 'New Instance', icon: 'plus-circle', href: '/mcps/new/' },
    ],
  },
  {
    id: 'api-keys',
    label: 'API Keys',
    icon: 'key-round',
    href: '/tokens/',
    subPages: [
      { id: 'app-tokens', label: 'App Tokens', icon: 'key-round', href: '/tokens/' },
      { id: 'new-token', label: 'New Token', icon: 'plus-circle', href: '/tokens/new/' },
      { id: 'access-requests', label: 'Access Requests', icon: 'shield-check', href: '/users/access-requests/' },
    ],
  },
  {
    id: 'settings',
    label: 'Settings',
    icon: 'settings',
    href: '/settings/',
    subPages: [
      { id: 'app-settings', label: 'App Settings', icon: 'settings', href: '/settings/' },
      { id: 'manage-users', label: 'Manage Users', icon: 'users', href: ROUTE_USERS },
    ],
  },
];
