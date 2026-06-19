import { AliasResponse } from '@bodhiapp/ts-client';

import { ShellIcon } from '@/components/shell';
import { Switch } from '@/components/ui/switch';
import { isApiAlias, isLocalAlias } from '@/lib/utils';
import { AliasCombobox } from './AliasCombobox';
import { AliasTypeBadge, ProviderBadge } from './AliasTypeBadge';
import { RouteToModelField } from './RouteToModelField';

export interface TargetRow {
  alias: string;
  model: string;
  enabled: boolean;
}

interface StepCardProps {
  target: TargetRow;
  index: number;
  total: number;
  selected?: AliasResponse;
  options: AliasResponse[];
  byIdentity: Map<string, AliasResponse>;
  onSelectAlias: (idx: number, identity: string) => void;
  onChangeModel: (idx: number, model: string) => void;
  onToggleEnabled: (idx: number, enabled: boolean) => void;
  onMove: (idx: number, dir: -1 | 1) => void;
  onRemove: (idx: number) => void;
  /** Controlled combobox open state (only one step's combobox is open at a time). */
  comboboxOpen: boolean;
  onComboboxOpenChange: (open: boolean) => void;
}

function IcoBtn({
  icon,
  testId,
  title,
  disabled,
  danger,
  onClick,
}: {
  icon: string;
  testId: string;
  title: string;
  disabled?: boolean;
  danger?: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      data-testid={testId}
      title={title}
      disabled={disabled}
      onClick={onClick}
      className={`rf-step-ico-btn${danger ? ' rf-step-ico-del' : ''}`}
    >
      <ShellIcon name={icon} size={12} />
    </button>
  );
}

export function StepCard({
  target,
  index,
  total,
  selected,
  options,
  byIdentity,
  onSelectAlias,
  onChangeModel,
  onToggleEnabled,
  onMove,
  onRemove,
  comboboxOpen,
  onComboboxOpenChange,
}: StepCardProps) {
  const isFirst = index === 0;
  const isLast = index === total - 1;
  const enabled = target.enabled;
  // Local repo backing for the meta line (no synthetic "backing" — use the real repo).
  const backing = selected && isLocalAlias(selected) ? selected.repo : undefined;

  return (
    <div data-testid={`target-row-${index}`} className={`rf-step-card${enabled ? '' : ' rf-step-disabled'}`}>
      <div className="rf-step-header">
        <span className="rf-step-num" data-testid={`step-num-${index}`}>
          {index + 1}
        </span>
        <span className="rf-step-label">
          Step {index + 1}
          {enabled && isFirst && total > 1 && <span className="rf-step-note">primary</span>}
          {enabled && isLast && !isFirst && <span className="rf-step-note">final fallback</span>}
          {!enabled && <span className="rf-step-note rf-step-note-warn">disabled</span>}
        </span>
        <span className="rf-step-spacer" />
        <div className="rf-enable-toggle">
          <Switch
            data-testid={`target-enabled-${index}`}
            checked={enabled}
            onCheckedChange={(checked) => onToggleEnabled(index, checked)}
            aria-label={enabled ? 'Disable this step' : 'Enable this step'}
          />
        </div>
        <IcoBtn
          icon="chevron-up"
          testId={`target-up-${index}`}
          title="Move up"
          disabled={isFirst}
          onClick={() => onMove(index, -1)}
        />
        <IcoBtn
          icon="chevron-down"
          testId={`target-down-${index}`}
          title="Move down"
          disabled={isLast}
          onClick={() => onMove(index, 1)}
        />
        <IcoBtn icon="x" testId={`target-remove-${index}`} title="Remove step" danger onClick={() => onRemove(index)} />
      </div>

      <div className="rf-step-body">
        <div className="rf-sub-label">Model alias</div>
        <AliasCombobox
          value={target.alias}
          options={options}
          byIdentity={byIdentity}
          onSelect={(id) => onSelectAlias(index, id)}
          testId={`target-alias-${index}`}
          open={comboboxOpen}
          onOpenChange={onComboboxOpenChange}
        />
        {selected && (
          <div className="rf-alias-meta" data-testid={`target-alias-meta-${index}`}>
            <AliasTypeBadge alias={selected} small />
            <ProviderBadge alias={selected} />
            {backing && <span className="rf-alias-meta-text mono">→ {backing}</span>}
            {isApiAlias(selected) && selected.forward_all_with_prefix && (
              <span className="rf-alias-meta-text rf-alias-meta-fwd">· forwards any model</span>
            )}
          </div>
        )}

        {selected && (
          <div className="rf-step-model">
            <RouteToModelField
              alias={selected}
              value={target.model}
              onChange={(m) => onChangeModel(index, m)}
              testId={`target-model-${index}`}
            />
          </div>
        )}
      </div>
    </div>
  );
}
