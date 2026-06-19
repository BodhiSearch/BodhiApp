import { AliasResponse } from '@bodhiapp/ts-client';

import { ShellIcon } from '@/components/shell';
import { isApiAlias, isModelAlias } from '@/lib/utils';

/** Discriminated alias type → the prototype's three badge looks, driven by the real `source`. */
type AliasKind = 'local-file' | 'model-alias' | 'api-model';

export function aliasKind(alias: AliasResponse): AliasKind {
  if (isApiAlias(alias)) return 'api-model';
  if (isModelAlias(alias)) return 'local-file';
  return 'model-alias'; // user alias (model-router is filtered out upstream)
}

const KIND_CFG: Record<AliasKind, { cls: string; icon: string; label: string }> = {
  'local-file': { cls: 'rf-badge-local', icon: 'hard-drive', label: 'Local File' },
  'model-alias': { cls: 'rf-badge-alias', icon: 'tag', label: 'Model Alias' },
  'api-model': { cls: 'rf-badge-api', icon: 'at-sign', label: 'API Model' },
};

export function AliasTypeBadge({ alias, small = false }: { alias: AliasResponse; small?: boolean }) {
  const cfg = KIND_CFG[aliasKind(alias)];
  return (
    <span className={`rf-type-badge ${cfg.cls}${small ? ' rf-type-badge-sm' : ''}`}>
      <ShellIcon name={cfg.icon} size={small ? 8 : 9} />
      {cfg.label}
    </span>
  );
}

/** Provider = the real `api_format` (uppercased); only API aliases have one. */
export function ProviderBadge({ alias }: { alias: AliasResponse }) {
  if (!isApiAlias(alias)) return null;
  return <span className="rf-provider-badge">{alias.api_format.toUpperCase()}</span>;
}
