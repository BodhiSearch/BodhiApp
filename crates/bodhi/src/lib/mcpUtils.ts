import type { McpAuthConfigResponse } from '@/hooks/mcps';

export function authConfigTypeBadge(config: McpAuthConfigResponse): string {
  switch (config.type) {
    case 'header':
      return 'Header';
    case 'oauth':
      return 'OAuth';
  }
}

export function authConfigTypeLabel(type: string): string {
  switch (type) {
    case 'header':
      return 'Header / Query Params';
    case 'oauth':
      return 'OAuth';
    default:
      return type;
  }
}

export function authConfigBadgeVariant(config: McpAuthConfigResponse): 'default' | 'secondary' | 'outline' {
  switch (config.type) {
    case 'header':
      return 'secondary';
    case 'oauth':
      return 'default';
  }
}

export function authConfigDetail(config: McpAuthConfigResponse): string {
  if (config.type === 'header') {
    const keys = config.entries.map((e) => `${e.param_type}:${e.param_key}`);
    return keys.length > 0 ? `Keys: ${keys.join(', ')}` : 'No keys defined';
  }
  return `${config.scopes || 'no scopes'}`;
}
