import type { McpAuthConfigResponse } from '@/hooks/useMcps';

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
      return 'Header';
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
  if (config.type === 'header') return `Key: ${config.header_key}`;
  return `${config.scopes || 'no scopes'}`;
}
