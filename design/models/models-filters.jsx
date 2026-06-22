/* ═══════════════════════════════════════════════════════════════
   Bodhi Models — sidebar filters
   models/models-filters.jsx   (load after models-base.jsx)

   Per-mode filter definitions and the ModelsSidebar that renders them
   into the shell's `sidebar` slot. The local-explore sidebar also
   stitches in the publisher / browse / context groups that live in
   bodhi-models-local.jsx (only loaded on the Local page).

   Exports: MODE_CFG, FILTERS, PLACEHOLDER, ModelsSidebar
═══════════════════════════════════════════════════════════════ */
const { LOCAL_MODELS, API_CATALOG_MODELS } = window.MODELS_DATA;

/* Provider options for the catalog filter — the full set of served-by
   providers across the catalog (Ⓗ would be GET /api/v1/providers). Many
   of them, so the filter is an autocomplete with cancellable tags. */
const PROVIDER_OPTIONS = (() => {
  const map = new Map();
  (API_CATALOG_MODELS || []).forEach((m) => (m.providers || []).forEach((p) => {
    if (!map.has(p.slug)) map.set(p.slug, p.name || p.slug);
  }));
  return [...map].map(([slug, name]) => ({ slug, name })).sort((a, b) => a.name.localeCompare(b.name));
})();

/* ── Mode config (one fixed mode per page) ────────────────────── */
const MODE_CFG = {
  'my-models':   { subPage: 'my-models',          label: 'My Models' },
  'local':       { subPage: 'explore-local',      label: 'Explore · Local Models' },
  'api':         { subPage: 'explore-api',        label: 'Explore · API Providers' },
  'api-catalog': { subPage: 'explore-api-catalog', label: 'Explore · API Models' }
};

/* ── Search placeholders (per mode) ───────────────────────────── */
const PLACEHOLDER = {
  'my-models': 'Search by alias, repo, filename, base URL — ⌘K',
  'local': 'Search HuggingFace repos — ⌘K',
  'api': 'Search API providers — ⌘K',
  'api-catalog': 'Search models by name, family, or provider… — ⌘K'
};

const FILTERS = {
  'my-models': [
  { icon: 'shapes', label: 'Type', chips: [
    { label: 'Local', color: 'saffron' }, { label: 'Alias', color: 'lotus' },
    { label: 'API', color: 'indigo' }, { label: 'Router', color: 'teal' }] },
  { icon: 'sparkles', label: 'Capability', chips: ['chat', 'tool-use', 'vision', 'embeddings', 'reasoning'].map((l) => ({ label: l })) },
  { icon: 'ruler', label: 'Size', note: '(local files)', range: { min: 0, max: 16, step: 1, unit: ' GB', defaultMin: 0, defaultMax: 16 } },
  { icon: 'plug', label: 'API Format', note: '(API only)', chips: ['OpenAI', 'Responses', 'Anthropic', 'Gemini'].map((l) => ({ label: l })) }],

  'local': [
  { icon: 'compass', label: 'Browse', chips: [{ label: '↗ Trending' }, { label: '✦ New' }] },
  { icon: 'list-checks', label: 'Task', chips: [
    { label: 'Text Generation', color: 'indigo', defaultOn: true }, { label: 'Image-Text-to-Text', color: 'lotus' }] },
  { icon: 'target', label: 'Specialisation', clearable: true, chips: [
    { label: 'All', defaultOn: true }, { label: 'Coding', color: 'saffron' },
    { label: 'Reasoning' }, { label: 'Vision' }] },
  { icon: 'hash', label: 'Tag', note: '(advanced)', clearable: true, chips: [
    { label: 'tool-use' }, { label: 'conversational' }, { label: 'thinking' }, { label: 'moe' }, { label: 'embedding' }] },
  { icon: 'languages', label: 'Language', clearable: true, chips: [
    { label: 'en', color: 'leaf' }, { label: 'zh', color: 'leaf' }, { label: 'es', color: 'leaf' }, { label: 'fr', color: 'leaf' }, { label: 'de', color: 'leaf' }, { label: 'ja', color: 'leaf' }, { label: 'ko', color: 'leaf' }] },
  { icon: 'scale', label: 'License', clearable: true, chips: [
    { label: 'Apache-2', color: 'indigo', defaultOn: true }, { label: 'MIT' }, { label: 'Llama' }, { label: 'Gemma' }, { label: 'DeepSeek' }] }],

  'api': [
  { icon: 'activity', label: 'Status', single: true, chips: [
    { label: 'Connected', color: 'leaf' }] },
  { icon: 'sparkles', label: 'Capability', chips: [
    { label: 'Reasoning', color: 'indigo' }, { label: 'Tool use', color: 'indigo' }, { label: 'Vision', color: 'indigo' }, { label: 'Structured' }] },
  { icon: 'dollar-sign', label: 'Pricing', note: '($/Mtok)', range: { min: 0, max: 75, step: 0.5, unit: '/M', prefix: '$', defaultMin: 0, defaultMax: 75 } },
  { icon: 'plug', label: 'API Format', chips: [
    { label: 'OpenAI', color: 'indigo' }, { label: 'Responses' }, { label: 'Anthropic' }, { label: 'Gemini' }, { label: 'Other' }] }],

  'api-catalog': [
  { icon: 'at-sign', label: 'Provider', chips: [
    { label: 'Anthropic', color: 'lotus' }, { label: 'OpenAI', color: 'leaf' }, { label: 'Google', color: 'indigo' },
    { label: 'Groq', color: 'saffron' }, { label: 'DeepSeek', color: 'teal' }, { label: 'Meta', color: 'indigo' }] },
  { icon: 'sparkles', label: 'Capability', chips: [
    { label: 'Reasoning', color: 'indigo' }, { label: 'Tool use', color: 'indigo' }, { label: 'Vision', color: 'indigo' }, { label: 'Structured output' }] },
  { icon: 'shapes', label: 'Modality', note: '(input & output)', chips: [
    { label: 'Text', color: 'indigo' }, { label: 'Image' }, { label: 'PDF' }, { label: 'Audio' }] },
  { icon: 'dollar-sign', label: 'Pricing', note: '(input $/Mtok)', range: { min: 0, max: 75, step: 0.5, unit: '/M', prefix: '$', defaultMin: 0, defaultMax: 75 } },
  { icon: 'ruler', label: 'Context size', note: '(context window)', range: { min: 0, max: 1000, step: 8, unit: 'K', defaultMin: 0, defaultMax: 1000 } },
  { icon: 'activity', label: 'Status', clearable: true, chips: [
    { label: 'Stable', color: 'leaf' }, { label: 'Beta', color: 'saffron' }, { label: 'Deprecated' }] },
  { icon: 'unlock', label: 'Open weights', single: true, clearable: true, chips: [
    { label: 'Open weights', color: 'leaf' }] }]

};

/* Provider → multi-select autocomplete with cancellable tags.
   Mirrors the local Publisher combo; values are provider slugs. */
function ProviderComboGroup({ options, values, onToggle, onClear }) {
  const { collapsed, openPop, setOpenPop } = useShell();
  const [q, setQ] = React.useState('');
  const [open, setOpen] = React.useState(false);
  const ref = React.useRef(null);
  const anchorRef = React.useRef(null);
  React.useEffect(() => {
    if (!open) return;
    const h = (e) => { if (ref.current && !ref.current.contains(e.target)) setOpen(false); };
    document.addEventListener('mousedown', h);
    return () => document.removeEventListener('mousedown', h);
  }, [open]);

  const sel = values || [];
  const nameOf = (slug) => { const o = options.find((x) => x.slug === slug); return o ? o.name : slug; };
  const matches = options.filter((o) => !sel.includes(o.slug) &&
    (o.name.toLowerCase().includes(q.toLowerCase()) || o.slug.toLowerCase().includes(q.toLowerCase()))).slice(0, 8);

  if (collapsed) {
    const popId = 'fg:Provider';
    const isOpen = openPop === popId;
    return (
      <>
        <button ref={anchorRef} className={'shell-railbtn shell-tip' + (sel.length ? ' on' : '')} data-tip="Provider"
                onClick={(e) => { e.stopPropagation(); setOpenPop(isOpen ? null : popId); }}>
          <Ic name="at-sign" size={17} />
          {sel.length > 0 && <span className="rb-badge">{sel.length}</span>}
        </button>
        <AnchoredPopover open={isOpen} anchorRef={anchorRef} onClose={() => setOpenPop(null)}>
          <div className="shell-pop-title"><span>Provider</span>{sel.length > 0 && <button className="fg-clear" onClick={onClear}>Clear</button>}</div>
          <div className="shell-pop-chips">
            {options.map((o) =>
              <button key={o.slug} className={'shell-fc fc-neutral' + (sel.includes(o.slug) ? ' on' : '')} onClick={() => onToggle(o.slug)}>{o.name}</button>
            )}
          </div>
        </AnchoredPopover>
      </>
    );
  }

  return (
    <div className="shell-filtergroup" ref={ref}>
      <div className="shell-fg-label">
        <span className="fg-ico"><Ic name="at-sign" size={13} /></span>
        <span className="fg-name">Provider</span>
        {sel.length > 0 && <button className="fg-clear" onClick={onClear}>Clear</button>}
      </div>
      {sel.length > 0 &&
        <div className="pub-tags">
          {sel.map((slug) =>
            <span className="pub-chip" key={slug}><Ic name="at-sign" size={11} />{nameOf(slug)}<button className="pub-x" onClick={() => onToggle(slug)}><Ic name="x" size={10} /></button></span>
          )}
        </div>}
      <div className="pub-combo">
        <span className="pub-ico"><Ic name="search" size={12} /></span>
        <input className="pub-input" placeholder={sel.length ? 'Add another…' : 'Search providers…'} value={q}
          onChange={(e) => { setQ(e.target.value); setOpen(true); }} onFocus={() => setOpen(true)}
          onKeyDown={(e) => { if (e.key === 'Enter' && matches[0]) { onToggle(matches[0].slug); setQ(''); } }} />
        {open && matches.length > 0 &&
          <div className="pub-pop">
            {matches.map((o) =>
              <button key={o.slug} className="pub-opt" onClick={() => { onToggle(o.slug); setQ(''); }}><Ic name="at-sign" size={12} />{o.name}</button>
            )}
          </div>}
      </div>
    </div>
  );
}

function ModelsSidebar({ mode, orgFilters, onPickOrg, onRemoveOrg, onClearOrgs, sort, onBrowse, apiConnectedOnly, onToggleApiConnected,
  catProv, onToggleCatProv, onClearCatProv, catCap, onToggleCatCap, onClearCatCap }) {
  const { collapsed } = useShell();
  if (mode === 'api-catalog') {
    return (
      <>
        {FILTERS['api-catalog'].map((g) => {
          if (g.label === 'Provider')
            return <ProviderComboGroup key={'cat-' + g.label} options={PROVIDER_OPTIONS}
              values={catProv} onToggle={onToggleCatProv} onClear={onClearCatProv} />;
          if (g.label === 'Capability')
            return <ShellFilterGroup key={'cat-' + g.label} icon={g.icon} label={g.label} clearable chips={g.chips}
              values={catCap} onToggle={onToggleCatCap} onClear={onClearCatCap} />;
          return <ShellFilterGroup key={'cat-' + g.label} icon={g.icon} label={g.label}
            note={g.note} clearable={g.clearable} chips={g.chips} range={g.range} single={g.single} />;
        })}
      </>);
  }
  if (mode !== 'local') {
    return (
      <>
        {FILTERS[mode].map((g) =>
        g.label === 'Status' && g.single ?
        <ShellFilterGroup key={mode + '-' + g.label} icon={g.icon} label={g.label} chips={g.chips} single clearable
          value={apiConnectedOnly ? 'Connected' : null} onSelect={() => onToggleApiConnected && onToggleApiConnected()} /> :
        <ShellFilterGroup key={mode + '-' + g.label} icon={g.icon} label={g.label}
        note={g.note} clearable={g.clearable} chips={g.chips} range={g.range} />
        )}
      </>);
  }
  return (
    <>
      {FILTERS.local.map((g) =>
      <React.Fragment key={'local-' + g.label}>
        {g.label === 'Browse'
          ? <BrowseGroup sort={sort} onBrowse={onBrowse} />
          : <ShellFilterGroup icon={g.icon} label={g.label} note={g.note} clearable={g.clearable} chips={g.chips} range={g.range} />}
        {g.label === 'Tag' &&
          <PublisherGroup orgFilters={orgFilters} onPick={onPickOrg} onRemove={onRemoveOrg} />}
      </React.Fragment>
      )}
    </>);
}

Object.assign(window, { MODE_CFG, FILTERS, PLACEHOLDER, ModelsSidebar });
