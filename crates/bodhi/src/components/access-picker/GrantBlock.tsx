import { AccessPicker } from './AccessPicker';
import { ListingToggle } from './ListingToggle';
import type { AccessItem, AccessMode } from './types';

interface GrantBlockProps {
  /** Singular resource noun, e.g. `model` or `MCP`. */
  noun: string;

  /** Render the listing toggle (default true). */
  showListing?: boolean;
  listChecked: boolean;
  onListToggle: () => void;
  listLabel: string;
  listCode?: string;
  listDescription: string;
  listTestId: string;

  /** Render the All/Specific access picker (default true). */
  showAccess?: boolean;
  mode: AccessMode;
  onModeChange: (mode: AccessMode) => void;
  items: AccessItem[];
  selectedIds: string[];
  onToggle: (id: string) => void;
  panelTitle: string;
  panelSubtitle: string;
  allLabel?: string;
  allDesc?: string;
  specificLabel?: string;
  specificDesc?: string;
  testIdPrefix: string;

  disabled?: boolean;
}

/** A single resource grant editor — the listing toggle plus the All/Specific
 *  access picker. Shared by the API-token form and the app-access consent screen
 *  so both express model/MCP grants identically. Either half can be hidden (the
 *  consent screen shows each only when the app requested it). */
export function GrantBlock({
  noun,
  showListing = true,
  listChecked,
  onListToggle,
  listLabel,
  listCode,
  listDescription,
  listTestId,
  showAccess = true,
  mode,
  onModeChange,
  items,
  selectedIds,
  onToggle,
  panelTitle,
  panelSubtitle,
  allLabel,
  allDesc,
  specificLabel,
  specificDesc,
  testIdPrefix,
  disabled = false,
}: GrantBlockProps) {
  return (
    <div className="grant-block" data-testid={`${testIdPrefix}-block`}>
      {showListing && (
        <ListingToggle
          checked={listChecked}
          onToggle={onListToggle}
          label={listLabel}
          code={listCode}
          description={listDescription}
          // "All" already lists everything — only a meaningful hint when the picker is shown.
          redundant={showAccess && mode === 'all'}
          disabled={disabled}
          testId={listTestId}
        />
      )}
      {showAccess && (
        <AccessPicker
          mode={mode}
          onModeChange={onModeChange}
          items={items}
          selectedIds={selectedIds}
          onToggle={onToggle}
          noun={noun}
          panelTitle={panelTitle}
          panelSubtitle={panelSubtitle}
          allLabel={allLabel}
          allDesc={allDesc}
          specificLabel={specificLabel}
          specificDesc={specificDesc}
          disabled={disabled}
          testIdPrefix={testIdPrefix}
        />
      )}
    </div>
  );
}
