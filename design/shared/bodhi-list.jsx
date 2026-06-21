/* ═══════════════════════════════════════════════════
   BODHI — shared LIST-PAGE components
   bodhi-list.jsx   (load AFTER bodhi-app-shell.jsx, BEFORE the page app)

   Exports (to window): ListToolbar, ListView, ListPage
   Uses globals from the shell: ShellIcon, ShellSearch.

   ── ListToolbar ─────────────────────────────────────
   The unified control band for every list page.

     <ListToolbar
       categories={[{ id, label, cls, badge }]}   // sub-category pills (left)
       category={id} onCategory={fn}
       loading={bool}                               // badges show a shimmer until data resolves
       search={q} onSearch={fn} searchPlaceholder="…"
       searchMode="collapse" | "inline" | "none"   // collapse = icon → row above
       searchKbd="⌘K"                              // inline mode only
       actions={<…/>}                              // extra right-side buttons/icons
     />

   • collapse (default): search is an icon on the right. Click → the full
     input drops into its OWN ROW ABOVE the pills. Esc or ✕ closes; ✕ also
     clears. A dot marks the icon when a query is active but the row is shut.
   • inline: a permanent full-width search bar (Models keeps this forever).
   • none: no search affordance.

   ── ListView ────────────────────────────────────────
   Edge-to-edge list frame with a sticky column header. Cells are page-local.
     <ListView head={<>…label cells…</>}>{rows}</ListView>
     <ListView columns={[{ label, cls, style }]}>{rows}</ListView>
═══════════════════════════════════════════════════ */

function ListToolbar({
  categories = [], category, onCategory,
  search = '', onSearch, searchPlaceholder = 'Search…',
  searchMode = 'collapse', searchKbd,
  loading = false,
  actions, children,
}) {
  const [open, setOpen] = React.useState(false);
  const hasVal = Boolean(search && String(search).length);

  const closeSearch = () => { if (onSearch) onSearch(''); setOpen(false); };

  const pills = categories.length > 0 ? (
    <div className="l-cats">
      {categories.map(c => (
        <button key={c.id}
                className={'l-cat ' + (c.cls || '') + (category === c.id ? ' on' : '')}
                onClick={() => onCategory && onCategory(c.id)}>
          {c.label}
          {(loading || c.badge != null) && (
            loading
              ? <span className="l-cat-badge l-cat-badge--loading" aria-label="Loading count" />
              : <span className="l-cat-badge">{c.badge}</span>
          )}
        </button>
      ))}
    </div>
  ) : null;

  return (
    <div className="l-controls">
      {searchMode === 'collapse' && open && (
        <div className="l-searchrow">
          <ShellSearch value={search} onChange={onSearch} placeholder={searchPlaceholder} autoFocus
                       onKeyDown={e => { if (e.key === 'Escape') setOpen(false); }} />
          <button className="l-iconbtn" title="Close search" onClick={closeSearch}>
            <ShellIcon name="x" size={15} />
          </button>
        </div>
      )}
      <div className="l-toolbar">
        {searchMode === 'inline' && (
          <ShellSearch value={search} onChange={onSearch} placeholder={searchPlaceholder} kbd={searchKbd} />
        )}
        {pills}
        <div className="l-tb-actions">
          {actions}
          {searchMode === 'collapse' && (
            <button className={'l-iconbtn' + (open ? ' on' : '')} title="Search"
                    onClick={() => setOpen(o => !o)}>
              <ShellIcon name="search" size={15} />
              {hasVal && !open && <span className="l-dot" />}
            </button>
          )}
          {children}
        </div>
      </div>
    </div>
  );
}

/* ── RowLink ─────────────────────────────────────────
   An empty, stretched <a href="#"> that turns a selectable row into a real
   LINK target — so keyboard/accessibility tools (e.g. the Vimium extension)
   surface the whole row when listing links, and screen readers announce it.
   It fills the row but sits BEHIND the row's own controls (see .l-rowlink in
   bodhi-list.css), so buttons / selects keep working and a normal mouse click
   still lands on the row. Activating the link runs the same select handler. */
function RowLink({ onActivate, label }) {
  return (
    <a className="l-rowlink" href="#" aria-label={label || 'Open details'}
       onMouseDown={e => e.preventDefault()}
       onClick={e => { e.preventDefault(); if (onActivate) onActivate(); }}>
    </a>
  );
}

/* ── ListRow ─────────────────────────────────────────
   The shared selectable list row. Renders the .l-listrow frame + the stretched
   RowLink + your cell children. Selecting (mouse OR link/keyboard) calls
   onSelect; the selected row gets an app-wide left accent border (.active).
     active    → selected state (wash + --c-lotus-text left border)
     accent    → status stripe color (adds .accent, sets --row-accent)
     className → page-local classes for cell layout
     onSelect  → fired on row click AND on RowLink activation
     label     → aria-label for the row link
     style     → merged through (pages can still pass CSS vars) */
function ListRow({ active, accent, className = '', style, onSelect, label, children, ...rest }) {
  const cls = ['l-listrow', accent ? 'accent' : '', active ? 'active' : '', className]
    .filter(Boolean).join(' ');
  const mergedStyle = accent ? { ...style, '--row-accent': accent } : style;
  return (
    <div className={cls} style={mergedStyle} onClick={() => onSelect && onSelect()}
         role={onSelect ? 'option' : undefined} aria-selected={onSelect ? !!active : undefined} {...rest}>
      <RowLink onActivate={onSelect} label={label} />
      {children}
    </div>
  );
}

/* ── useListKeyNav ───────────────────────────────────────
   Arrow-key navigation for a selectable master-detail list. ↑/↓ move the
   selection by one row — EAGER: selecting also opens the detail panel, exactly
   like a click — Home/End jump to first/last, and movement STOPS at the ends
   (no wrap). It works by activating the target row's stretched <a.l-rowlink>,
   reusing each page's existing select handler, so no per-page state wiring is
   needed. One call per page from the component that renders the list.
   Scoped: ignores keys while focus is in a text field, the left sidebar, or the
   right detail rail, so typing and panel use are never hijacked.
     rowSelector  → rows to navigate     (default '.l-listrow')
     rootSelector → the scroll container (default '.l-scroll')        */
function useListKeyNav(opts = {}) {
  const rowSelector  = opts.rowSelector  || '.l-listrow';
  const rootSelector = opts.rootSelector || '.l-scroll';
  React.useEffect(() => {
    function scrollIntoView(root, row) {
      const cr = root.getBoundingClientRect();
      const rr = row.getBoundingClientRect();
      const head = root.querySelector('.l-listhead');
      const top = cr.top + (head ? head.offsetHeight : 0) + 6;
      if (rr.top < top) root.scrollTop -= (top - rr.top);
      else if (rr.bottom > cr.bottom - 6) root.scrollTop += (rr.bottom - cr.bottom + 6);
    }
    function onKey(e) {
      if (e.defaultPrevented || e.ctrlKey || e.metaKey || e.altKey) return;
      if (!['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(e.key)) return;
      const ae = document.activeElement;
      if (ae && ae.closest('input, textarea, select, [contenteditable=""], [contenteditable="true"], .shell-sidebar, .shell-rail')) return;
      const root = document.querySelector(rootSelector);
      if (!root) return;
      const rows = Array.from(root.querySelectorAll(rowSelector)).filter(el => el.offsetParent !== null);
      if (!rows.length) return;
      let cur = rows.findIndex(r => r.classList.contains('active'));
      if (cur < 0 && ae) cur = rows.findIndex(r => r.contains(ae));
      let next;
      if (e.key === 'Home') next = 0;
      else if (e.key === 'End') next = rows.length - 1;
      else if (e.key === 'ArrowDown') next = cur < 0 ? 0 : Math.min(rows.length - 1, cur + 1);
      else next = cur < 0 ? rows.length - 1 : Math.max(0, cur - 1);
      e.preventDefault();
      if (next === cur) return;            // stop at ends — no wrap
      const row = rows[next];
      const link = row.querySelector('.l-rowlink') || row;
      link.focus({ preventScroll: true }); // ring follows keyboard selection
      link.click();                        // reuse the row's existing select handler
      scrollIntoView(root, row);
    }
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  }, [rowSelector, rootSelector]);
}

function ListView({ head, columns, children, className = '' }) {
  const headNode = head || (columns && columns.map((c, i) => (
    <div key={i} className={'l-lh ' + (c.cls || '')} style={c.style}>{c.label}</div>
  )));
  return (
    <div className={'l-listview ' + className}>
      {headNode && <div className="l-listhead">{headNode}</div>}
      {children}
    </div>
  );
}

/* Convenience skeleton: <ListPage toolbar={<ListToolbar/>}>{scroll content}</ListPage> */
function ListPage({ toolbar, children, scrollClass = '' }) {
  return (
    <div className="l-page">
      {toolbar}
      <div className={'l-scroll ' + scrollClass}>{children}</div>
    </div>
  );
}

Object.assign(window, { ListToolbar, ListView, ListPage, RowLink, ListRow, useListKeyNav });
