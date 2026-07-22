import { LinkRow, ShellIcon } from '@/components/shell';
import './catalog.css';

export type ColumnAlign = 'left' | 'right';

/**
 * One column of a {@link CatalogTable}. Headers, row cells, and the `<colgroup>` all derive from this
 * array so the column picker (show/hide) and any sortable header stay in sync.
 *
 * - `width` is a `<col>` width (px); an empty string means "no explicit width" — that column absorbs
 *   the table's slack under `table-layout: fixed`.
 * - `sort` (when set) makes the header a sortable {@link ColSort}; its value is the page's sort key.
 * - `align` keeps the header label justified like the cell content (text left, numeric right).
 * - `optional` marks the column as toggleable via the column picker.
 * - The column with `key: 'num'` is special-cased: its cell renders the `#` index plus the row's
 *   stretched {@link LinkRow} (the keyboard-nav / link-hint target). Every numbered table needs one.
 *
 * `S` is the page's sort-key union; pass `string` when the page has no sortable columns.
 */
export interface CatalogColumn<T, S extends string = string> {
  key: string;
  label: string;
  width: string;
  align?: ColumnAlign;
  sort?: S;
  optional?: boolean;
  cell: (row: T, idx: number) => React.ReactNode;
}

/**
 * A sortable column header. Icon: `chevrons-up-down` when inactive, `arrow-up`/`arrow-down` when
 * active. With `descendingOnly` the active icon is always `arrow-down` and `order` is ignored — for
 * catalogs whose upstream rejects ascending order (e.g. the HuggingFace-backed local catalog).
 */
export function ColSort<S extends string>({
  col,
  label,
  sort,
  order,
  align,
  descendingOnly,
  testIdPrefix,
  onSort,
}: {
  col: S;
  label: string;
  sort: S | undefined;
  order?: 'asc' | 'desc';
  align: ColumnAlign;
  descendingOnly?: boolean;
  testIdPrefix: string;
  onSort: (c: S) => void;
}) {
  const active = sort === col;
  const icon = !active ? 'chevrons-up-down' : descendingOnly || order === 'desc' ? 'arrow-down' : 'arrow-up';
  return (
    <button
      type="button"
      className={`cat-colsort${align === 'left' ? ' cat-colsort--left' : ''}${active ? ' on' : ''}`}
      onClick={() => onSort(col)}
      data-testid={`${testIdPrefix}-sort-${col}`}
      data-test-state={active ? 'active' : 'idle'}
    >
      <span className="cat-colsort-label">{label}</span>
      <ShellIcon name={icon} size={10} />
    </button>
  );
}

function CatalogRow<T, S extends string>({
  row,
  idx,
  active,
  columns,
  testId,
  label,
  rowAttrs,
  onSelect,
}: {
  row: T;
  idx: number;
  active: boolean;
  columns: CatalogColumn<T, S>[];
  testId: string;
  label: string;
  rowAttrs?: Record<string, string>;
  onSelect: () => void;
}) {
  return (
    <tr
      className={`l-listrow cat-row${active ? ' active' : ''}`}
      onClick={onSelect}
      role="option"
      aria-selected={active}
      data-testid={testId}
      {...rowAttrs}
    >
      {columns.map((col) =>
        col.key === 'num' ? (
          // The row link lives IN the `#` cell (compact): a leftmost, always-visible, uncovered
          // target that link-hint tools (Vimium) detect reliably even under horizontal overflow —
          // a full-row stretched anchor is not. Keyboard nav still activates this `.l-rowlink`.
          <td className="cat-num-td" key="num">
            <LinkRow onActivate={onSelect} label={label}>
              <span className="cat-num">#{idx}</span>
            </LinkRow>
          </td>
        ) : (
          <td
            key={col.key}
            className={col.key === 'logo' ? 'cat-logo-td' : col.align === 'right' ? 'cat-td--right' : undefined}
          >
            {col.cell(row, idx)}
          </td>
        )
      )}
    </tr>
  );
}

/**
 * A semantic `<table>` for the catalog/list screens, generic over the row type `T` and sort-key union
 * `S`. Renders the `<colgroup>`/`<thead>` (sortable columns become {@link ColSort}) and a `<tbody>` of
 * {@link CatalogRow}s. The `<tr className="l-listrow">` + first-cell {@link LinkRow} are the contract
 * `useListKeyNav` relies on for arrow-key navigation — keep them.
 *
 * Per-page code is just the `columns` adapter plus the `rowKey`/`rowTestId`/`rowLabel` extractors.
 */
export function CatalogTable<T, S extends string>({
  columns,
  rows,
  rowKey,
  rowTestId,
  rowLabel,
  rowAttrs,
  activeKey,
  onSelect,
  sort,
  order,
  onSort,
  startIndex = 0,
  descendingOnly,
  testIdPrefix,
}: {
  columns: CatalogColumn<T, S>[];
  rows: T[];
  rowKey: (row: T) => string;
  rowTestId: (row: T) => string;
  rowLabel: (row: T) => string;
  rowAttrs?: (row: T) => Record<string, string>;
  activeKey: string | null;
  onSelect: (row: T) => void;
  sort?: S;
  order?: 'asc' | 'desc';
  onSort: (c: S) => void;
  /** 0-based index of the first row across pages, so `#` numbering continues past page 1. */
  startIndex?: number;
  descendingOnly?: boolean;
  testIdPrefix: string;
}) {
  return (
    <table className="cat-table">
      <colgroup>
        {columns.map((col) => (
          <col key={col.key} style={col.width ? { width: col.width } : undefined} />
        ))}
      </colgroup>
      <thead className="cat-listhead" data-testid="cat-listhead">
        <tr>
          {columns.map((col) =>
            col.sort ? (
              <th key={col.key} scope="col" className={col.align === 'right' ? 'cat-th--right' : undefined}>
                <ColSort
                  col={col.sort}
                  label={col.label}
                  sort={sort}
                  order={order}
                  align={col.align ?? 'left'}
                  descendingOnly={descendingOnly}
                  testIdPrefix={testIdPrefix}
                  onSort={onSort}
                />
              </th>
            ) : (
              <th
                key={col.key}
                scope="col"
                className={`cat-colhead${col.align === 'right' ? ' cat-colhead--right' : ''}`}
                aria-hidden={col.label === '' ? true : undefined}
              >
                {col.label}
              </th>
            )
          )}
        </tr>
      </thead>
      <tbody className="l-listview">
        {rows.map((row, i) => {
          const key = rowKey(row);
          return (
            <CatalogRow
              key={key}
              row={row}
              idx={startIndex + i + 1}
              active={key === activeKey}
              columns={columns}
              testId={rowTestId(row)}
              label={rowLabel(row)}
              rowAttrs={rowAttrs?.(row)}
              onSelect={() => onSelect(row)}
            />
          );
        })}
      </tbody>
    </table>
  );
}
