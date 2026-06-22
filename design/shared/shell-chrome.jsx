/* ═══════════════════════════════════════════════════════════════
   Bodhi App Shell — CHROME (sidebar nav + filter controls)
   shared/shell-chrome.jsx   (load after shell-core.jsx)

   The primary section nav (expanded dropdown / collapsed icon rail),
   the page-level controls that live in the `sidebar` slot
   (mode switch, range slider, filter group), plus the brand mark and
   breadcrumb. Reads collapse state from ShellContext (shell-core).
═══════════════════════════════════════════════════════════════ */

/* ── Primary section nav (expanded dropdown OR collapsed icon rail) ── */
function ShellNav({ section = 'chat', subPage = null }) {
  const { collapsed, openPop, setOpenPop } = useShell();
  const open = openPop === 'nav';
  const anchorRef = React.useRef(null);
  const cur = SHELL_NAV.find(n => n.id === section) || SHELL_NAV[0];

  React.useEffect(() => {
    if (!open) return;
    const h = () => setOpenPop(null);
    document.addEventListener('click', h);
    return () => document.removeEventListener('click', h);
  }, [open]);

  const menuItems = SHELL_NAV.map(item => (
    <a key={item.id} href={item.href || '#'} className={'shell-nav-item' + (item.id === section ? ' on' : '')}>
      <ShellIcon name={item.icon} color={item.id === section ? '#DB456C' : 'currentColor'} />
      {item.label}
      {item.badge && <span className="shell-nav-badge">{item.badge}</span>}
    </a>
  ));

  if (collapsed) {
    return (
      <>
        <button ref={anchorRef} className={'shell-railbtn shell-tip on'} data-tip={cur.label + ' · switch section'}
                onClick={e => { e.stopPropagation(); setOpenPop(open ? null : 'nav'); }}>
          <ShellIcon name={cur.icon} size={18} />
        </button>
        <AnchoredPopover open={open} anchorRef={anchorRef} onClose={() => setOpenPop(null)}>
          <div className="shell-pop-title">Go to section</div>
          {menuItems}
        </AnchoredPopover>
        {cur.subPages && cur.subPages.length > 0 && <div className="shell-iconrail-div" />}
        {cur.subPages && cur.subPages.map(sp => (
          <a key={sp.id} href={sp.href || '#'}
             className={'shell-railbtn shell-tip' + (sp.id === subPage ? ' on' : '')} data-tip={sp.label}>
            <ShellIcon name={sp.icon || 'circle'} size={17} />
            {sp.badge && <span className="rb-badge">{sp.badge}</span>}
          </a>
        ))}
      </>
    );
  }

  return (
    <div className="shell-nav-block">
      <div className={'shell-nav' + (open ? ' open' : '')} onClick={e => e.stopPropagation()}>
        <button className="shell-nav-trigger" data-tip="Switch section" onClick={e => { e.stopPropagation(); setOpenPop(open ? null : 'nav'); }}>
          <span className="lead"><ShellIcon name={cur.icon} size={15} color="#DB456C" /></span>
          <span className="lbl">{cur.label}</span>
          <span className="chev"><ShellIcon name="chevron-down" /></span>
        </button>
        {open && <div className="shell-nav-menu">{menuItems}</div>}
      </div>
      {cur.subPages && cur.subPages.length > 0 && (
        <div className="shell-sub">
          {cur.subPages.map(sp => (
            <a key={sp.id} href={sp.href || '#'} data-tip={sp.label} className={'shell-sub-item' + (sp.id === subPage ? ' on' : '')}>
              <ShellIcon name={sp.icon || 'circle'} size={13} color={sp.id === subPage ? '#DB456C' : 'currentColor'} />
              {sp.label}
              {sp.badge && <span className="shell-sub-badge">{sp.badge}</span>}
            </a>
          ))}
        </div>
      )}
    </div>
  );
}

/* ── Mode switch (page control) ─────────────────────────────── */
function ShellModeSwitch({ value, onChange, options = [], label }) {
  const { collapsed } = useShell();
  if (collapsed) {
    return (
      <>
        {options.map(o => (
          <button key={o.id} className={'shell-railbtn shell-tip' + (o.id === value ? ' on' : '')}
                  data-tip={o.label} onClick={() => onChange(o.id)}>
            <ShellIcon name={o.icon || 'circle'} size={18} />
          </button>
        ))}
      </>
    );
  }
  return (
    <div>
      {label && <div className="shell-fg-label" style={{ paddingTop: 10 }}><span className="fg-name">{label}</span></div>}
      <div className="shell-modeswitch">
        {options.map(o => (
          <button key={o.id} data-tip={o.label + (o.sub ? ' — ' + o.sub : '')} className={'shell-mode-card' + (o.id === value ? ' on' : '')} onClick={() => onChange(o.id)}>
            <span className="shell-mode-ico"><ShellIcon name={o.icon || 'circle'} size={16} /></span>
            <span>
              <span className="shell-mode-label" style={{ display: 'block' }}>{o.label}</span>
              {o.sub && <span className="shell-mode-sub" style={{ display: 'block' }}>{o.sub}</span>}
            </span>
          </button>
        ))}
      </div>
    </div>
  );
}

/* ── Dual-handle range slider ───────────────────────────────── */
function ShellRangeSlider({ min = 0, max = 100, step = 1, unit = '', prefix = '', defaultMin, defaultMax }) {
  const [lo, setLo] = React.useState(defaultMin != null ? defaultMin : min);
  const [hi, setHi] = React.useState(defaultMax != null ? defaultMax : max);
  const pct = v => ((v - min) / (max - min)) * 100;
  const fmt = v => prefix + v + (v >= max ? '+' : '') + unit;

  const onLo = e => setLo(Math.min(Number(e.target.value), hi - step));
  const onHi = e => setHi(Math.max(Number(e.target.value), lo + step));

  return (
    <div className="shell-range">
      <div className="shell-range-vals">
        <span>{fmt(lo)}</span>
        <span>{fmt(hi)}</span>
      </div>
      <div className="shell-range-track">
        <div className="shell-range-rail" />
        <div className="shell-range-fill" style={{ left: pct(lo) + '%', right: (100 - pct(hi)) + '%' }} />
        <input type="range" min={min} max={max} step={step} value={lo} onChange={onLo}
               className="shell-range-input" style={{ zIndex: lo > max - step ? 5 : 3 }} aria-label="Minimum" />
        <input type="range" min={min} max={max} step={step} value={hi} onChange={onHi}
               className="shell-range-input" style={{ zIndex: 4 }} aria-label="Maximum" />
      </div>
    </div>
  );
}

/* ── Filter group (page control) ────────────────────────────── */
function ShellFilterGroup({ icon = 'filter', label, chips = [], note, clearable, range, value, onSelect, single }) {
  const { collapsed, openPop, setOpenPop } = useShell();
  const popId = 'fg:' + label;
  const open = openPop === popId;
  const [sel, setSel] = React.useState(() => new Set(chips.filter(c => c.defaultOn).map(c => c.label)));
  const anchorRef = React.useRef(null);

  const toggle = lbl => setSel(prev => {
    const next = new Set(prev);
    next.has(lbl) ? next.delete(lbl) : next.add(lbl);
    return next;
  });
  const clear = () => setSel(new Set());

  const chipId = c => c.id != null ? c.id : c.label;
  const isOn = c => single ? value === chipId(c) : sel.has(c.label);
  const handle = c => single ? (onSelect && onSelect(chipId(c))) : toggle(c.label);
  const activeCount = single ? (value && value !== 'all' ? 1 : 0) : sel.size;

  const chipEls = chips.map(c => (
    <button key={chipId(c)} data-tip={c.label}
            className={'shell-fc fc-' + (c.color || 'neutral') + (isOn(c) ? ' on' : '')}
            onClick={() => handle(c)}>{c.label}{c.badge != null && <span className="shell-fc-badge">{c.badge}</span>}</button>
  ));

  if (collapsed) {
    return (
      <>
        <button ref={anchorRef} className={'shell-railbtn shell-tip' + (activeCount ? ' on' : '')} data-tip={label}
                onClick={e => { e.stopPropagation(); setOpenPop(open ? null : popId); }}>
          <ShellIcon name={icon} size={17} />
          {activeCount > 0 && <span className="rb-badge">{activeCount}</span>}
        </button>
        <AnchoredPopover open={open} anchorRef={anchorRef} onClose={() => setOpenPop(null)}>
          <div className="shell-pop-title">
            <span>{label}</span>
            {clearable && (single ? value && value !== 'all' : sel.size > 0) && <button className="fg-clear" onClick={() => single ? onSelect && onSelect('all') : clear()}>Clear</button>}
          </div>
          {range ? <div className="shell-pop-chips"><ShellRangeSlider {...range} /></div>
                 : <div className="shell-pop-chips">{chipEls}</div>}
        </AnchoredPopover>
      </>
    );
  }

  return (
    <div className="shell-filtergroup">
      <div className="shell-fg-label">
        <span className="fg-ico"><ShellIcon name={icon} size={13} /></span>
        <span className="fg-name">{label}{note && <span className="fg-note"> {note}</span>}</span>
        {clearable && sel.size > 0 && <button className="fg-clear" onClick={clear}>Clear</button>}
      </div>
      {range ? <ShellRangeSlider {...range} /> : <div className="shell-fc-row">{chipEls}</div>}
    </div>
  );
}

/* ── Default brand mark ─────────────────────────────────────── */
function ShellBrand({ collapsed }) {
  return (
    <a href="Bodhi Chat.html" style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
      <img src="assets/bodhi-logo-60.svg" alt="Bodhi" onError={e => { e.target.style.display = 'none'; }} />
      {!collapsed && (
        <span>
          <span className="shell-brand-t" style={{ display: 'block' }}>Bodhi</span>
          <span className="shell-brand-s" style={{ display: 'block' }}>AI Gateway</span>
        </span>
      )}
    </a>
  );
}

/* ── Breadcrumb ─────────────────────────────────────────────── */
function ShellBreadcrumb({ items }) {
  if (!items) return null;
  if (!Array.isArray(items)) return <div className="shell-bc">{items}</div>;
  return (
    <div className="shell-bc">
      {items.map((it, i) => (
        <React.Fragment key={i}>
          {i > 0 && <ShellIcon name="chevron-right" size={11} />}
          {it.current
            ? <span className="shell-bc-current">{it.label}</span>
            : <a className="shell-bc-seg" href={it.href || '#'}>{it.label}</a>}
        </React.Fragment>
      ))}
    </div>
  );
}

Object.assign(window, {
  ShellNav, ShellModeSwitch, ShellRangeSlider, ShellFilterGroup,
  ShellBrand, ShellBreadcrumb,
});
