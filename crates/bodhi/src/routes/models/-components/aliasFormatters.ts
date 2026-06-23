import { AliasResponse } from '@bodhiapp/ts-client';

import { ModelTypeFacet } from '@/hooks/models';
import { isApiAlias, isModelRouterAlias } from '@/lib/utils';

/** A stable identity for any alias (api/router use id, local aliases use the alias name). */
export function getAliasId(alias: AliasResponse): string {
  if (isApiAlias(alias) || isModelRouterAlias(alias)) return alias.id;
  return alias.alias;
}

/** Primary display name — api uses name (falling back to id), others use the alias. */
export function getAliasTitle(alias: AliasResponse): string {
  if (isApiAlias(alias)) return alias.name || alias.id;
  return alias.alias;
}

export interface AliasTypeMeta {
  token: ModelTypeFacet;
  label: string;
  badgeCls: string;
  iconCls: string;
  icon: string;
}

/** TYPE-facet token + display label + badge + icon-tile classes (color-coded per type). */
export function getAliasTypeMeta(alias: AliasResponse): AliasTypeMeta {
  switch (alias.source) {
    case 'model':
      return {
        token: 'local_file',
        label: 'Local File',
        badgeCls: 'm-badge-local',
        iconCls: 'm-icon-local',
        icon: 'hard-drive',
      };
    case 'user':
      return {
        token: 'model_alias',
        label: 'Model Alias',
        badgeCls: 'm-badge-alias',
        iconCls: 'm-icon-alias',
        icon: 'tag',
      };
    case 'api':
      return {
        token: 'api_model',
        label: 'API Model',
        badgeCls: 'm-badge-api',
        iconCls: 'm-icon-api',
        icon: 'at-sign',
      };
    default:
      return {
        token: 'fallback',
        label: 'Router',
        badgeCls: 'm-badge-fallback',
        iconCls: 'm-icon-fallback',
        icon: 'route',
      };
  }
}
