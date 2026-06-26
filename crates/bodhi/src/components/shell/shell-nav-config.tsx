import {
  ROUTE_CHAT,
  ROUTE_MCPS,
  ROUTE_MCPS_EXPLORE,
  ROUTE_MODELS_EXPLORE_API,
  ROUTE_MODELS_EXPLORE_LOCAL,
  ROUTE_MODELS_EXPLORE_PROVIDERS,
  ROUTE_USERS,
} from '@/lib/constants';

export interface ShellNavSubPage {
  id: string;
  label: string;
  /** kebab-case lucide icon name. */
  icon?: string;
  href: string;
  badge?: string;
  /** Hidden in multi-tenant deployments (e.g. local-model features that can't download there). */
  hideInMultiTenant?: boolean;
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
      { id: 'my-models', label: 'My Models', icon: 'globe-2', href: '/models/' },
      {
        id: 'explore-local',
        label: 'Explore · Local Models',
        icon: 'compass',
        href: ROUTE_MODELS_EXPLORE_LOCAL,
        // Catalog browse-and-pull relies on local downloads, which HubService rejects in MultiTenant.
        hideInMultiTenant: true,
      },
      {
        id: 'explore-api',
        label: 'Explore · API Models',
        icon: 'sparkles',
        href: ROUTE_MODELS_EXPLORE_API,
        hideInMultiTenant: true,
      },
      {
        id: 'explore-api-providers',
        label: 'Explore · API Providers',
        icon: 'at-sign',
        href: ROUTE_MODELS_EXPLORE_PROVIDERS,
        hideInMultiTenant: true,
      },
      { id: 'new-local-model', label: 'New Local Model', icon: 'plus-circle', href: '/models/alias/new/' },
      { id: 'new-api-model', label: 'New API Model', icon: 'plug-zap', href: '/models/api/new/' },
      { id: 'new-fallback-model', label: 'New Model Router', icon: 'route', href: '/models/router/new/' },
    ],
  },
  {
    id: 'mcp',
    label: 'MCP',
    icon: 'plug',
    href: ROUTE_MCPS,
    subPages: [
      { id: 'my-mcps', label: 'My MCPs', icon: 'globe-2', href: ROUTE_MCPS },
      { id: 'explore-mcp', label: 'Explore · MCP Servers', icon: 'compass', href: ROUTE_MCPS_EXPLORE },
      { id: 'new-mcp', label: 'New Instance', icon: 'plus-circle', href: '/mcps/new/' },
    ],
  },
  {
    id: 'api-keys',
    label: 'Access Tokens',
    icon: 'key-round',
    href: '/tokens/',
    subPages: [
      { id: 'api-tokens', label: 'API Tokens', icon: 'key-round', href: '/tokens/' },
      { id: 'new-token', label: 'New API Token', icon: 'plus-circle', href: '/tokens/new/' },
    ],
  },
  {
    id: 'users',
    label: 'Users',
    icon: 'users',
    href: '/users/access-requests/',
    subPages: [
      { id: 'access-requests', label: 'User Access Requests', icon: 'user-check', href: '/users/access-requests/' },
      { id: 'manage-users', label: 'Manage Users', icon: 'users', href: ROUTE_USERS },
    ],
  },
  {
    id: 'settings',
    label: 'Settings',
    icon: 'settings',
    href: '/settings/',
    subPages: [{ id: 'app-settings', label: 'App Settings', icon: 'settings', href: '/settings/' }],
  },
];
