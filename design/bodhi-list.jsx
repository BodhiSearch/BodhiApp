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
    <div className={cls} style={mergedStyle} onClick={() => onSelect && onSelect()} {...rest}>
      <RowLink onActivate={onSelect} label={label} />
      {children}
    </div>
  );
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

Object.assign(window, { ListToolbar, ListView, ListPage, RowLink, ListRow });
