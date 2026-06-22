import { ShellIcon } from './ShellIcon';

export interface ShellPaginationProps {
  /** Total rows across all pages. */
  total: number;
  /** 1-based current page. */
  page: number;
  onPage: (page: number) => void;
  /** Rows per page (used to derive page count + the count label). */
  pageSize: number;
  /** Provide to render the rows-per-page selector; omit to hide it. */
  onPageSize?: (size: number) => void;
  pageSizeOptions?: number[];
  /** Show the "from–to of total" count label (full mode only). */
  showCount?: boolean;
  /** How many page numbers to show either side of the current page. */
  siblingCount?: number;
  /** Noun used in the count label, e.g. "models". */
  unit?: string;
  className?: string;
  /** Centered page nav only — no count, no rows-per-page selector. */
  minimal?: boolean;
}

/** Page-number range with ellipsis gaps: [1, 'gap-l', 4, 5, 6, 'gap-r', 12]. */
function paginationRange(page: number, pageCount: number, sibling: number): Array<number | 'gap-l' | 'gap-r'> {
  if (pageCount <= 1) return [1];
  const range: Array<number | 'gap-l' | 'gap-r'> = [1];
  const left = Math.max(2, page - sibling);
  const right = Math.min(pageCount - 1, page + sibling);
  if (left > 2) range.push('gap-l');
  for (let i = left; i <= right; i++) range.push(i);
  if (right < pageCount - 1) range.push('gap-r');
  range.push(pageCount);
  return range;
}

/**
 * Theme-aligned footer pager for any list page: numbered pages with ellipses +
 * prev/next, an optional rows-per-page selector, and a result count. Controlled —
 * the parent owns `page` (1-based) and `pageSize`. Pass `minimal` for a centered
 * page-nav-only band (the default for the server-paginated V2 list pages).
 */
export function ShellPagination({
  total,
  page,
  onPage,
  pageSize,
  onPageSize,
  pageSizeOptions = [10, 25, 50, 100],
  showCount = true,
  siblingCount = 1,
  unit = 'items',
  className = '',
  minimal = false,
}: ShellPaginationProps) {
  const pageCount = Math.max(1, Math.ceil(total / pageSize));
  const cur = Math.min(Math.max(1, page), pageCount);
  const go = (p: number) => {
    const next = Math.min(Math.max(1, p), pageCount);
    if (next !== cur) onPage(next);
  };
  const from = total === 0 ? 0 : (cur - 1) * pageSize + 1;
  const to = Math.min(total, cur * pageSize);
  const pages = paginationRange(cur, pageCount, siblingCount);
  const fmt = (n: number) => n.toLocaleString();

  const nav = (
    <div className="l-pag-nav" role="navigation" aria-label="Pagination">
      <button
        type="button"
        className="l-pag-btn l-pag-arrow"
        disabled={cur <= 1}
        onClick={() => go(cur - 1)}
        aria-label="Previous page"
        title="Previous page"
        data-testid="pagination-prev"
      >
        <ShellIcon name="chevron-left" size={15} />
      </button>
      {pages.map((p, i) =>
        typeof p === 'number' ? (
          <button
            key={`p${p}`}
            type="button"
            className={`l-pag-btn${p === cur ? ' on' : ''}`}
            aria-current={p === cur ? 'page' : undefined}
            onClick={() => go(p)}
            data-testid={`pagination-page-${p}`}
          >
            {p}
          </button>
        ) : (
          <span key={`${p}${i}`} className="l-pag-gap" aria-hidden="true">
            …
          </span>
        )
      )}
      <button
        type="button"
        className="l-pag-btn l-pag-arrow"
        disabled={cur >= pageCount}
        onClick={() => go(cur + 1)}
        aria-label="Next page"
        title="Next page"
        data-testid="pagination-next"
      >
        <ShellIcon name="chevron-right" size={15} />
      </button>
    </div>
  );

  if (minimal) {
    return (
      <div className={`l-pagination minimal${className ? ` ${className}` : ''}`} data-testid="pagination">
        {nav}
      </div>
    );
  }

  return (
    <div className={`l-pagination${className ? ` ${className}` : ''}`} data-testid="pagination">
      <div className="l-pag-side l-pag-left">
        {showCount &&
          (total === 0 ? (
            <span className="l-pag-count">No {unit}</span>
          ) : (
            <span className="l-pag-count">
              <strong>
                {fmt(from)}–{fmt(to)}
              </strong>{' '}
              of <strong>{fmt(total)}</strong> {unit}
            </span>
          ))}
      </div>
      {nav}
      <div className="l-pag-side l-pag-right">
        {onPageSize && (
          <label className="l-pag-size">
            <span>Rows</span>
            <span className="l-pag-select">
              <select value={pageSize} aria-label="Rows per page" onChange={(e) => onPageSize(Number(e.target.value))}>
                {pageSizeOptions.map((n) => (
                  <option key={n} value={n}>
                    {n}
                  </option>
                ))}
              </select>
              <ShellIcon name="chevron-down" size={13} />
            </span>
          </label>
        )}
      </div>
    </div>
  );
}
