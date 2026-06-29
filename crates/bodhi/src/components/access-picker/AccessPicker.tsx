import { useMemo, useState } from 'react';

import { Plus, X } from 'lucide-react';

import { cn } from '@/lib/utils';

import { AccessPickerPanel } from './AccessPickerPanel';
import type { AccessItem, AccessMode } from './types';

interface AccessPickerProps {
  mode: AccessMode;
  onModeChange: (mode: AccessMode) => void;
  items: AccessItem[];
  selectedIds: string[];
  onToggle: (id: string) => void;
  /** Singular resource noun, e.g. `model` or `MCP`. */
  noun: string;
  panelTitle: string;
  panelSubtitle: string;
  allLabel?: string;
  allDesc?: string;
  specificLabel?: string;
  specificDesc?: string;
  disabled?: boolean;
  /** Prefix for the radio / add / panel / selected-row testids. */
  testIdPrefix: string;
}

/** All/Specific radio + (when specific) selected-rows list with remove + an
 *  Add button that opens the slide-in panel. Shared by models and MCPs. */
export function AccessPicker({
  mode,
  onModeChange,
  items,
  selectedIds,
  onToggle,
  noun,
  panelTitle,
  panelSubtitle,
  allLabel,
  allDesc,
  specificLabel,
  specificDesc,
  disabled = false,
  testIdPrefix,
}: AccessPickerProps) {
  const [panelOpen, setPanelOpen] = useState(false);

  const selectedItems = useMemo(
    () => selectedIds.map((id) => items.find((i) => i.id === id) ?? { id, label: id }).filter(Boolean),
    [selectedIds, items]
  );

  const pickMode = (next: AccessMode) => {
    if (disabled) return;
    onModeChange(next);
    if (next === 'specific' && mode !== 'specific') setPanelOpen(true);
  };

  const Radio = ({
    value,
    label,
    desc,
    futureBadge,
  }: {
    value: AccessMode;
    label: string;
    desc: string;
    futureBadge?: boolean;
  }) => (
    <button
      type="button"
      className={cn('ap-radio', mode === value && 'selected', disabled && 'is-disabled')}
      onClick={() => pickMode(value)}
      aria-pressed={mode === value}
      data-testid={`${testIdPrefix}-mode-${value}`}
    >
      <span className="ap-radio-dot">
        <span className="ap-radio-dot-inner" />
      </span>
      <span className="ap-radio-body">
        <span className="ap-radio-text">
          {label}
          {futureBadge && <span className="ap-future-badge">+ future</span>}
        </span>
        <span className="ap-radio-desc">{desc}</span>
      </span>
    </button>
  );

  return (
    <div className="access-picker" data-testid={testIdPrefix}>
      <div className="ap-radio-group">
        <Radio
          value="all"
          label={allLabel ?? `All ${noun}s`}
          desc={allDesc ?? `Grant access to all current and future ${noun}s.`}
          futureBadge
        />
        <Radio
          value="specific"
          label={specificLabel ?? `Specific ${noun}s`}
          desc={specificDesc ?? `Choose exactly which ${noun}s are accessible.`}
        />
      </div>

      {mode === 'specific' && (
        <div className="ap-specific">
          {selectedItems.length > 0 ? (
            <div className="ap-selected-list" data-testid={`${testIdPrefix}-selected-list`}>
              {selectedItems.map((item) => (
                <div className="ap-selected-row" key={item.id} data-testid={`${testIdPrefix}-selected-${item.id}`}>
                  <span className="ap-selected-name">{item.label}</span>
                  {'meta' in item && item.meta && <span className="ap-selected-meta">{item.meta}</span>}
                  {!disabled && (
                    <button
                      type="button"
                      className="ap-remove"
                      onClick={() => onToggle(item.id)}
                      title="Remove"
                      aria-label={`Remove ${item.label}`}
                      data-testid={`${testIdPrefix}-remove-${item.id}`}
                    >
                      <X />
                    </button>
                  )}
                </div>
              ))}
            </div>
          ) : (
            <div className="ap-empty-hint" data-testid={`${testIdPrefix}-empty`}>
              No {noun}s selected — no access will be granted.
            </div>
          )}

          {!disabled && (
            <button
              type="button"
              className="ap-add"
              onClick={() => setPanelOpen(true)}
              data-testid={`${testIdPrefix}-add`}
            >
              <Plus />
              {selectedItems.length > 0 ? `Add more ${noun}s` : `Select ${noun}s`}
            </button>
          )}
        </div>
      )}

      <AccessPickerPanel
        open={panelOpen}
        onClose={() => setPanelOpen(false)}
        title={panelTitle}
        subtitle={panelSubtitle}
        items={items}
        selectedIds={selectedIds}
        onToggle={onToggle}
        noun={noun}
        testIdPrefix={testIdPrefix}
      />
    </div>
  );
}
