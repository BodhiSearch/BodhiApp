import type { Model, SortKey } from '@bodhiapp/reference-api-types';

import { LinkRow, ShellIcon } from '@/components/shell';

import { fmtDate } from './LocalDiscoveryRail';

/** Compact count: 1234 → 1.2k, 1_200_000 → 1.2M. */
export function compact(n: number | null | undefined): string {
  if (n == null) return '—';
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1).replace(/\.0$/, '')}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1).replace(/\.0$/, '')}k`;
  return String(n);
}

interface SortHeaderProps {
  label: string;
  col: SortKey;
  sort: SortKey;
  onSort: (col: SortKey) => void;
}

// Descending-only: the catalog API (and HuggingFace upstream) reject ascending order,
// so headers pick the sort key but never flip direction.
export function SortHeader({ label, col, sort, onSort }: SortHeaderProps) {
  const active = sort === col;
  return (
    <button
      type="button"
      className={`ld-sort-h${active ? ' on' : ''}`}
      onClick={() => onSort(col)}
      data-testid={`ld-sort-${col}`}
      data-test-state={active ? 'active' : 'idle'}
    >
      {label}
      <ShellIcon name={active ? 'arrow-down' : 'chevrons-up-down'} size={10} />
    </button>
  );
}

interface LocalRowProps {
  model: Model;
  idx: number;
  sort: SortKey;
  active: boolean;
  onSelect: () => void;
}

export function LocalRow({ model, idx, sort, active, onSelect }: LocalRowProps) {
  const tags = model.tags ?? model.specialisation ?? [];
  const isMultimodal = model.pipeline_tag === 'image-text-to-text';
  return (
    <div
      className={`l-listrow ld-row${active ? ' active' : ''}`}
      onClick={onSelect}
      role="option"
      aria-selected={active}
      data-testid={`ld-row-${model.namespace}-${model.repo}`}
    >
      <LinkRow onActivate={onSelect} label={`Open ${model.namespace}/${model.repo}`} />
      <div className="ld-num">#{idx}</div>
      <div className="ld-body">
        <div className="ld-name">
          <span className="ld-org">{model.namespace}</span>
          <span className="ld-sep">/</span>
          <span className="ld-repo">{model.repo}</span>
          {model.owner_verified && (
            <span className="ld-verified" title="Verified publisher">
              <ShellIcon name="badge-check" size={13} />
            </span>
          )}
          {isMultimodal && (
            <span className="ld-modality" title="Image-Text-to-Text (multimodal)">
              <ShellIcon name="image" size={10} />
              multimodal
            </span>
          )}
        </div>
        <div className="ld-tags">
          {tags.slice(0, 4).map((t) => (
            <span className="ld-tag" key={t}>
              {t}
            </span>
          ))}
          {model.quant_count != null && (
            <span className="ld-meta-chip" title={`${model.quant_count} quantizations`}>
              {model.quant_count} quants
            </span>
          )}
          {model.license && <span className="ld-meta-chip">{model.license}</span>}
        </div>
      </div>
      <div className="ld-stats">
        <div className={`ld-stat${sort === 'downloads' ? ' sorted' : ''}`}>
          <div className="ld-stat-num">{compact(model.downloads)}</div>
          <div className="ld-stat-lbl">DOWNLOADS</div>
        </div>
        <div className={`ld-stat${sort === 'likes' ? ' sorted' : ''}`}>
          <div className="ld-stat-num">{compact(model.likes)}</div>
          <div className="ld-stat-lbl">LIKES</div>
        </div>
        <div className={`ld-stat${sort === 'last_modified' ? ' sorted' : ''}`}>
          <div className="ld-stat-num ld-stat-date">{fmtDate(model.last_modified)}</div>
          <div className="ld-stat-lbl">UPDATED</div>
        </div>
      </div>
    </div>
  );
}
