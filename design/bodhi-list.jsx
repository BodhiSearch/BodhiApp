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
          {c.badge != null && <span className="l-cat-badge">{c.badge}</span>}
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

Object.assign(window, { ListToolbar, ListView, ListPage });
