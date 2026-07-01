import type { AliasResponse, Mcp } from '@bodhiapp/ts-client';

import type { AccessItem } from '@/components/access-picker';
import { modelId } from '@/lib/modelAlias';

/** Resolve each alias to grantable model items (the inference grant id space), tagged
 *  local/api so the picker can group + filter. Shared by the API-token form and the
 *  app-access consent screen. */
export function grantableModelItems(aliases: AliasResponse[]): AccessItem[] {
  const items = new Map<string, AccessItem>();
  for (const alias of aliases) {
    // The generated `source` is a plain string, so narrow structurally instead.
    if ('models' in alias && 'prefix' in alias) {
      const prefix = alias.prefix ?? '';
      for (const model of alias.models) {
        const id = `${prefix}${modelId(model)}`;
        if (!items.has(id)) items.set(id, { id, label: id, type: 'api' });
      }
    } else if ('targets' in alias && 'strategy' in alias) {
      // ModelRouterResponse — a composite routing alias (also carries `alias`), so it
      // must be matched BEFORE the local branch. Emit untyped (neither local nor api).
      if (!items.has(alias.alias)) items.set(alias.alias, { id: alias.alias, label: alias.alias });
    } else if ('alias' in alias) {
      if (!items.has(alias.alias)) items.set(alias.alias, { id: alias.alias, label: alias.alias, type: 'local' });
    }
  }
  return Array.from(items.values());
}

/** MCP instances as pickable items (instance id is the grant currency). */
export function grantableMcpItems(mcps: Mcp[]): AccessItem[] {
  return mcps.map((m) => ({ id: m.id, label: m.name }));
}
