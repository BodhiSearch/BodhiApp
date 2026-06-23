import type { McpAuthConfigResponse } from '@/hooks/mcps';

export const safeOrigin = (urlStr: string): string => {
  try {
    return new URL(urlStr).origin;
  } catch {
    return urlStr;
  }
};

export type AuthConfigOption = {
  id: string;
  name: string;
  type: 'header' | 'oauth';
  config: McpAuthConfigResponse;
};
