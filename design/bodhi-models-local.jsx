/* ═══════════════════════════════════════════════════════════════
   Bodhi Models — Explore · Local Models enrichments
   bodhi-models-local.jsx
   Load AFTER bodhi-app-shell.jsx and BEFORE bodhi-models-app.jsx.

   Pure helpers + presentational components shared with the page app
   (exposed on window at the bottom). Covers the LM-Studio-comparison
   brief: download-options picker, host-fit pills, metadata chips,
   README renderer, "More from <org>", publisher autocomplete, the
   split bit-width / quant-method filters, and the context slider.
═══════════════════════════════════════════════════════════════ */
const LIc = window.ShellIcon;
const { HOST: LM_HOST, ORG_SUGGESTIONS } = window.MODELS_DATA;

/* ── Derivations (Ⓗ catalog + Ⓛ host-local) ───────────────────── */
const parseGB = (s) => parseFloat(String(s).replace(/[^0-9.]/g, '')) || 0;

/* Bit-width = the digit right after "Q" in the quant name (Q4_K_M → 4).
   This is the PRECISION axis — distinct from the quant METHOD (_0 / K_S /
   K_M / K_L / IQ). The filename is a combination of both. */
const quantBits = (name) => {
  const m = String(name).match(/Q(\d+)/i);
  return m ? parseInt(m[1], 10) : null;
};
/* Quant method = the scheme suffix after the bit-width. */
const quantMethod = (name) => {
  const m = String(name).match(/^IQ\d+/i) ? 'IQ' :
  String(name).match(/_K_M$/i) ? 'K_M' :
  String(name).match(/_K_S$/i) ? 'K_S' :
  String(name).match(/_K_L$/i) ? 'K_L' :
  String(name).match(/_K$/i) ? 'K' :
  String(name).match(/_1$/) ? '_1' :
  String(name).match(/_0$/) ? '_0' : '—';
  return m;
};

/* Ⓛ Per-quant host fit, computed from quant size vs the machine profile.
   gpu = fits VRAM (full offload) · ram = fits system RAM (partial offload)
   · too-large = exceeds RAM. ~1 GB GPU headroom, ~4 GB RAM headroom. */
const quantFit = (sizeGB) => {
  if (sizeGB <= LM_HOST.vramGB - 1) return 'gpu';
  if (sizeGB <= LM_HOST.ramGB - 4) return 'ram';
  return 'too-large';
};
const FIT_CFG = {
  'gpu': { cls: 'fit-gpu', icon: 'zap', short: 'Fit', tip: 'Fits entirely in VRAM' },
  'ram': { cls: 'fit-ram', icon: 'cpu', short: 'Partial Fit', tip: 'Runs with partial CPU offload' },
  'too-large': { cls: 'fit-big', icon: 'alert-triangle', short: 'Too large', tip: 'Exceeds available memory' }
};
/* Model-level run status = best fit across its quants. */
const modelRunStatus = (quants) => {
  const fits = quants.map((q) => quantFit(parseGB(q.size)));
  if (fits.includes('gpu')) return { key: 'gpu', label: 'Runs fully on GPU' };
  if (fits.includes('ram')) return { key: 'ram', label: 'Runs in system RAM' };
  return { key: 'too-large', label: 'Too large for this machine' };
};

/* ── Relative time (with absolute on hover) ───────────────────── */
const NOW = new Date('2026-06-21T12:00:00');
const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
const fmtAbs = (iso) => {const d = new Date(iso);return d.getDate() + ' ' + MONTHS[d.getMonth()] + ' ' + d.getFullYear();};
const relTime = (iso) => {
  const d = new Date(iso);const days = Math.round((NOW - d) / 86400000);
  if (days < 1) return 'today';
  if (days < 30) return days + 'd ago';
  const months = Math.round(days / 30.4);
  if (months < 12) return months + 'mo ago';
  const years = (days / 365).toFixed(1).replace(/\.0$/, '');
  return years + 'y ago';
};
const ctxLabel = (n) => n >= 1024 ? Math.round(n / 1024) + 'k' : String(n);

/* ── Tiny markdown renderer (headings / lists / **bold** / `code`) ── */
function MarkdownView({ src }) {
  const blocks = [];let list = null,para = [];
  const flushP = () => {if (para.length) {blocks.push({ t: 'p', text: para.join(' ') });para = [];}};
  const flushL = () => {if (list) {blocks.push({ t: 'ul', items: list });list = null;}};
  String(src).split('\n').forEach((raw) => {
    const line = raw.trim();let m;
    if (!line) {flushP();flushL();return;}
    if (m = line.match(/^### (.*)/)) {flushP();flushL();blocks.push({ t: 'h3', text: m[1] });} else
    if (m = line.match(/^## (.*)/)) {flushP();flushL();blocks.push({ t: 'h2', text: m[1] });} else
    if (m = line.match(/^# (.*)/)) {flushP();flushL();blocks.push({ t: 'h1', text: m[1] });} else
    if (m = line.match(/^- (.*)/)) {flushP();(list = list || []).push(m[1]);} else
    {flushL();para.push(line);}
  });
  flushP();flushL();
  const inline = (text) => {
    const parts = [];const re = /(\*\*([^*]+)\*\*|`([^`]+)`)/g;let mm,last = 0;
    while (mm = re.exec(text)) {
      if (mm.index > last) parts.push(text.slice(last, mm.index));
      if (mm[2] != null) parts.push(<strong key={parts.length}>{mm[2]}</strong>);else
      parts.push(<code key={parts.length} className="md-code">{mm[3]}</code>);
      last = re.lastIndex;
    }
    if (last < text.length) parts.push(text.slice(last));
    return parts;
  };
  return (
    <div className="md">{blocks.map((b, i) => {
        if (b.t === 'h1') return <h1 key={i} className="md-h1">{inline(b.text)}</h1>;
        if (b.t === 'h2') return <h2 key={i} className="md-h2">{inline(b.text)}</h2>;
        if (b.t === 'h3') return <h3 key={i} className="md-h3">{inline(b.text)}</h3>;
        if (b.t === 'ul') return <ul key={i} className="md-ul">{b.items.map((it, j) => <li key={j}>{inline(it)}</li>)}</ul>;
        return <p key={i} className="md-p">{inline(b.text)}</p>;
      })}</div>);

}

/* ── Small badges / pills ─────────────────────────────────────── */
function FitPill({ fit, full }) {
  const c = FIT_CFG[fit];if (!c) return null;
  return <span className={'fit-pill ' + c.cls} title={c.tip}><LIc name={c.icon} size={11} /><span className="fit-txt">{full ? c.short : fit === 'gpu' ? 'GPU' : fit === 'ram' ? 'RAM' : 'Big'}</span></span>;
}
function RunBadge({ quants }) {
  const s = modelRunStatus(quants);const c = FIT_CFG[s.key];
  return <span className={'run-badge ' + c.cls} title={s.label + ' · ' + LM_HOST.label}><LIc name={c.icon} size={11} />{c.short}</span>;
}
function VerifiedBadge() {
  return <span className="ver-badge" title="Verified publisher"><LIc name="badge-check" size={13} /></span>;
}
function StaffBadge({ small }) {
  return <span className={'staff-badge' + (small ? ' sm' : '')} title="Recommended"><LIc name="thumbs-up" size={small ? 9 : 11} />{small ? '' : 'Recommended'}</span>;
}

/* ── Metadata chip row (detail header) — value-first, labels on hover ── */
function MetaChips({ m }) {
  return (
    <div className="meta-chips">
      <span className="mc" title={'Parameters: ' + m.params}><LIc name="box" size={11} /><span className="mc-v">{m.params}</span></span>
      <span className="mc" title={'Architecture: ' + m.arch}><LIc name="blocks" size={11} /><span className="mc-v mono">{m.arch}</span></span>
      <span className="mc" title={'Format: ' + m.format}><LIc name="file-box" size={11} /><span className="mc-v mono">{m.format}</span></span>
    </div>);

}

/* ── "More from <org>" cross-discovery strip ──────────────────── */
function MoreFrom({ org, items, onPickOrg }) {
  if (!items || !items.length) return null;
  return (
    <div className="p-section morefrom">
      <div className="p-sec-lbl mf-head">
        <span>More from <button className="mf-org" onClick={() => onPickOrg && onPickOrg(org)}>{org}</button></span>
        <button className="mf-all" onClick={() => onPickOrg && onPickOrg(org)}>View all <LIc name="arrow-right" size={11} /></button>
      </div>
      <div className="mf-list">
        {items.map((it) =>
        <button className="mf-row" key={it.repo} onClick={() => onPickOrg && onPickOrg(org)}>
            <span className="mf-name">{it.repo}</span>
            <span className="mf-stats">
              <span className="mf-stat"><LIc name="download" size={10} />{it.dl}</span>
              <span className="mf-stat"><LIc name="heart" size={10} />{it.likes}</span>
            </span>
          </button>
        )}
      </div>
    </div>);

}

/* ── Column show/hide menu (toolbar) ──────────────────────────── */
function ColumnsMenu({ cols, onToggle, compact }) {
  const [open, setOpen] = React.useState(false);
  const ref = React.useRef(null);
  React.useEffect(() => {
    if (!open) return;
    const h = (e) => {if (ref.current && !ref.current.contains(e.target)) setOpen(false);};
    document.addEventListener('mousedown', h);return () => document.removeEventListener('mousedown', h);
  }, [open]);
  const ITEMS = [{ k: 'human', label: 'Human evals' }, { k: 'downloads', label: 'Downloads' }, { k: 'likes', label: 'Likes' }];
  return (
    <div className="col-menu-wrap" ref={ref}>
      <button className={(compact ? 'col-head-btn' : 'l-iconbtn') + (open ? ' on' : '')} title="Show / hide columns" onClick={() => setOpen((v) => !v)} style={{ lineHeight: "1.4", fontSize: "13px", width: "24px" }}>
        <LIc name={compact ? 'columns-3' : 'sliders-horizontal'} size={compact ? 14 : 15} />
      </button>
      {open &&
      <div className={'col-menu' + (compact ? ' col-menu-head' : '')}>
          <div className="col-menu-title">Columns</div>
          {ITEMS.map((it) =>
        <button key={it.k} className="col-menu-row" onClick={() => onToggle(it.k)}>
              <span className={'col-check' + (cols[it.k] ? ' on' : '')}>{cols[it.k] && <LIc name="check" size={11} />}</span>
              {it.label}
            </button>
        )}
        </div>
      }
    </div>);

}

/* ── Sortable stat header cell ────────────────────────────────── */
function SortHeaderCell({ label, k, sortKey, sortOrder, onSort }) {
  const active = sortKey === k;
  return (
    <button className={'sort-h' + (active ? ' on' : '')} onClick={() => onSort(k)} title={'Sort by ' + label}>
      {label}
      <LIc name={active ? sortOrder === 'asc' ? 'arrow-up' : 'arrow-down' : 'chevrons-up-down'} size={10} />
    </button>);

}

/* ── Custom sidebar groups ────────────────────────────────────── */
function CollapsedRailBtn({ icon, label, on, badge, onClick }) {
  return (
    <button className={'shell-railbtn shell-tip' + (on ? ' on' : '')} data-tip={label}
    onClick={(e) => {e.stopPropagation();onClick && onClick();}}>
      <LIc name={icon} size={17} />
      {badge != null && <span className="rb-badge">{badge}</span>}
    </button>);

}

/* Browse → Trending / New (visual) + a real "Recommended" preset. */
function BrowseGroup({ staffOnly, onToggleStaff }) {
  const { collapsed } = window.useShell();
  const [browse, setBrowse] = React.useState('trending');
  if (collapsed) return <CollapsedRailBtn icon="compass" label="Browse" on={staffOnly} onClick={onToggleStaff} />;
  return (
    <div className="shell-filtergroup">
      <div className="shell-fg-label"><span className="fg-ico"><LIc name="compass" size={13} /></span><span className="fg-name">Browse</span></div>
      <div className="shell-fc-row">
        <button className={'shell-fc fc-neutral' + (browse === 'trending' ? ' on' : '')} onClick={() => setBrowse('trending')}>↗ Trending</button>
        <button className={'shell-fc fc-neutral' + (browse === 'new' ? ' on' : '')} onClick={() => setBrowse('new')}>✦ New</button>
        <button className={'shell-fc fc-saffron rec-fc' + (staffOnly ? ' on' : '')} onClick={onToggleStaff}><LIc name="thumbs-up" size={11} />Recommended</button>
      </div>
    </div>);

}

/* Publisher → multi-select autocomplete that also accepts free text.
   Selected publishers sit as cancellable tags above the lookup input. */
function PublisherGroup({ orgFilters, onPick, onRemove }) {
  const { collapsed } = window.useShell();
  const [q, setQ] = React.useState('');
  const [open, setOpen] = React.useState(false);
  const ref = React.useRef(null);
  React.useEffect(() => {
    if (!open) return;
    const h = (e) => {if (ref.current && !ref.current.contains(e.target)) setOpen(false);};
    document.addEventListener('mousedown', h);return () => document.removeEventListener('mousedown', h);
  }, [open]);
  if (collapsed) return <CollapsedRailBtn icon="building-2" label="Publisher" on={orgFilters.length > 0} badge={orgFilters.length || null} />;
  const sel = orgFilters.map((o) => o.toLowerCase());
  const matches = ORG_SUGGESTIONS.filter((o) => o.toLowerCase().includes(q.toLowerCase()) && !sel.includes(o.toLowerCase())).slice(0, 7);
  const commit = (val) => {const v = (val || '').trim();if (v && !sel.includes(v.toLowerCase())) onPick(v);setQ('');setOpen(false);};
  return (
    <div className="shell-filtergroup" ref={ref}>
      <div className="shell-fg-label"><span className="fg-ico"><LIc name="building-2" size={13} /></span><span className="fg-name">Publisher</span>{orgFilters.length > 0 && <button className="fg-clear" onClick={() => orgFilters.forEach(onRemove)}>Clear</button>}</div>
      {orgFilters.length > 0 &&
      <div className="pub-tags">
          {orgFilters.map((o) =>
        <span className="pub-chip" key={o}><LIc name="building-2" size={11} />{o}<button className="pub-x" onClick={() => onRemove(o)}><LIc name="x" size={10} /></button></span>
        )}
        </div>}
      <div className="pub-combo">
        <span className="pub-ico"><LIc name="search" size={12} /></span>
        <input className="pub-input" placeholder={orgFilters.length ? 'Add another…' : 'org or author…'} value={q}
        onChange={(e) => {setQ(e.target.value);setOpen(true);}} onFocus={() => setOpen(true)}
        onKeyDown={(e) => {if (e.key === 'Enter') commit(q);}} />
        {open && (matches.length > 0 || q) &&
        <div className="pub-pop">
            {matches.map((o) =>
          <button key={o} className="pub-opt" onClick={() => commit(o)}><LIc name="building-2" size={12} />{o}</button>
          )}
            {q && !matches.some((o) => o.toLowerCase() === q.toLowerCase()) && !sel.includes(q.toLowerCase()) &&
          <button className="pub-opt pub-free" onClick={() => commit(q)}><LIc name="plus" size={12} />Use "{q}"</button>
          }
          </div>
        }
      </div>
    </div>);

}

/* Format & Source — both axes future-proofed (GGUF/HF active, MLX soon). */
function FormatSourceGroup() {
  const { collapsed } = window.useShell();
  if (collapsed) return <CollapsedRailBtn icon="file-box" label="Format & Source" on={true} />;
  return (
    <div className="shell-filtergroup">
      <div className="shell-fg-label"><span className="fg-ico"><LIc name="file-box" size={13} /></span><span className="fg-name">Format & Source</span></div>
      <div className="fs-row">
        <span className="fs-lbl">Format</span>
        <div className="fs-toggle">
          <button className="fs-opt on">GGUF</button>
          <button className="fs-opt soon" title="Coming soon">MLX</button>
        </div>
      </div>
      <div className="fs-row">
        <span className="fs-lbl">Source</span>
        <div className="fs-toggle">
          <button className="fs-opt on">HuggingFace</button>
        </div>
      </div>
    </div>);

}

/* Context window — single-thumb MAX slider snapping to the context
   lengths actually present in the results. */
function ContextGroup({ stops }) {
  const { collapsed } = window.useShell();
  const max = stops.length - 1;
  const [i, setI] = React.useState(max);
  if (collapsed) return <CollapsedRailBtn icon="ruler" label="Context window" on={i < max} />;
  const pct = i / max * 100;
  const atMax = i >= max;
  return (
    <div className="shell-filtergroup">
      <div className="shell-fg-label"><span className="fg-ico"><LIc name="ruler" size={13} /></span><span className="fg-name">Context window</span>{!atMax && <button className="fg-clear" onClick={() => setI(max)}>Clear</button>}</div>
      <div className="shell-range ctx-range">
        <div className="shell-range-vals"><span>up to</span><span>{ctxLabel(stops[i])}{atMax ? '' : ''} tokens</span></div>
        <div className="shell-range-track">
          <div className="shell-range-rail" />
          <div className="shell-range-fill" style={{ left: 0, right: 100 - pct + '%' }} />
          <input type="range" min={0} max={max} step={1} value={i} onChange={(e) => setI(Number(e.target.value))} className="shell-range-input" style={{ zIndex: 4 }} aria-label="Maximum context" />
        </div>
        <div className="ctx-stops">{stops.map((s, idx) => <span key={s} className={'ctx-tick' + (idx === i ? ' on' : '')}>{ctxLabel(s)}</span>)}</div>
      </div>
    </div>);

}

Object.assign(window, {
  parseGB, quantBits, quantMethod, quantFit, FIT_CFG, modelRunStatus,
  relTime, fmtAbs, ctxLabel, MarkdownView, FitPill, RunBadge, VerifiedBadge,
  StaffBadge, MetaChips, MoreFrom, ColumnsMenu, SortHeaderCell,
  BrowseGroup, PublisherGroup, FormatSourceGroup, ContextGroup, LM_HOST
});