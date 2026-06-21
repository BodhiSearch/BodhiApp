/* ═══════════════════════════════════════════════════════════════
   Bodhi Models — page app
   bodhi-models-app.jsx   (load after bodhi-app-shell.jsx + bodhi-models-data.js)
═══════════════════════════════════════════════════════════════ */
const { useState, useEffect } = React;
const { MY_MODELS, LOCAL_MODELS, API_PROVIDERS, TAG_MAP, STATUS_CFG, PROV_COLORS } = window.MODELS_DATA;

const Ic = ShellIcon;
const tagClass = (t) => 'tag ' + (TAG_MAP[t] || 'tag-muted');
const Tag = ({ t, big }) => <span className={tagClass(t)} style={big ? { fontSize: 12, padding: '3px 9px' } : null}>{t}</span>;

/* ═══ Downloads ════════════════════════════════════════════════
   Right-rail panel listing in-flight + finished model pulls.
   The progress bar (DownloadProgress) is the SAME visual primitive
   used on the setup wizard's Local Models step — lotus-pink fill on
   a light track, recoloured per-theme. Reused here verbatim.
═══════════════════════════════════════════════════════════════ */
const fmtSize = (mb) => mb >= 1024 ? (mb / 1024).toFixed(mb / 1024 >= 10 ? 1 : 2) + ' GB' : mb.toFixed(1) + ' MB';
const gbToMb = (s) => parseFloat(s) * (s.includes('GB') ? 1024 : 1);

const DOWNLOADS_INIT = {
  active: [
    { id: 'd1', org: 'Qwen', repo: 'Qwen3-Coder-32B', file: 'Qwen3-Coder-32B-Q4_K_M.gguf', pct: 42.3, totalMB: 18944, rate: 0.55, speed: '24.6 MB/s' },
    { id: 'd2', org: 'BAAI', repo: 'bge-m3', file: 'bge-m3-Q4_K_M.gguf', pct: 30.3, totalMB: 417.5, rate: 4.1, speed: '17.8 MB/s' }
  ],
  queued: [
    { id: 'q1', org: 'meta-llama', repo: 'Llama-3.3-70B', file: 'Llama-3.3-70B-Instruct.Q4_K_M.gguf', total: '35.0 GB', waitOn: 'Qwen3-Coder-32B' }
  ],
  done: [
    { id: 'c1', org: 'Mungert', repo: 'SmolLM2-135M-Instruct-GGUF', file: 'SmolLM2-135M-Instruct-bf16_q8_0.gguf', size: '138 MB', when: 'Today, 16:49' },
    { id: 'c2', org: 'microsoft', repo: 'Phi-4', file: 'Phi-4-Q4_K_M.gguf', size: '5.1 GB', when: 'Yesterday' },
    { id: 'c3', org: 'google', repo: 'gemma-2-9b-it', file: 'gemma-2-9b-it-Q4_K_M.gguf', size: '5.8 GB', when: '2 days ago' }
  ],
  failed: [
    { id: 'f1', org: 'deepseek-ai', repo: 'DeepSeek-V3', file: 'DeepSeek-V3-Q2_K.gguf', size: '35.0 GB', reason: 'Not enough disk space' }
  ]
};

function DownloadProgress({ pct, loadedMB, totalMB }) {
  const p = Math.min(100, pct);
  return (
    <div className="dl-prog">
      <div className="dl-prog-head">
        <span className="dl-prog-pct">{p.toFixed(0)}%</span>
        <span className="dl-prog-bytes">{fmtSize(loadedMB)} <span className="dl-prog-sep">/</span> {fmtSize(totalMB)}</span>
      </div>
      <div className="dl-prog-track"><div className="dl-prog-fill" style={{ width: p + '%' }} /></div>
    </div>);
}

function DlRow({ d, kind, onCancel, onRetry, onClear }) {
  return (
    <div className={'dl-card dl-' + kind}>
      <div className="dl-card-top">
        <div className={'dl-icon dl-icon-' + kind}>
          <Ic name={kind === 'done' ? 'check' : kind === 'failed' ? 'alert-triangle' : kind === 'queued' ? 'clock' : 'hard-drive'} size={14} />
        </div>
        <div className="dl-meta">
          <div className="dl-name"><span className="dl-org">{d.org}/</span>{d.repo}</div>
          <div className="dl-file">{d.file}</div>
        </div>
        {kind === 'active' && <button className="dl-act" title="Cancel download" onClick={onCancel}><Ic name="x" size={13} /></button>}
        {kind === 'queued' && <button className="dl-act" title="Remove from queue" onClick={onCancel}><Ic name="x" size={13} /></button>}
        {kind === 'failed' && <button className="dl-act" title="Retry" onClick={onRetry}><Ic name="rotate-cw" size={13} /></button>}
        {kind === 'done' && <button className="dl-act" title="Remove from list" onClick={onClear}><Ic name="x" size={13} /></button>}
      </div>

      {kind === 'active' &&
        <>
          <DownloadProgress pct={d.pct} loadedMB={d.totalMB * d.pct / 100} totalMB={d.totalMB} />
          <div className="dl-substat"><Ic name="arrow-down" size={10} />{d.speed}<span className="dl-substat-dot">·</span>{d.eta || 'calculating…'}</div>
        </>}
      {kind === 'queued' &&
        <div className="dl-line"><span className="dl-line-l"><Ic name="clock" size={10} /> Waiting for {d.waitOn}</span><span className="dl-line-r">{d.total}</span></div>}
      {kind === 'done' &&
        <div className="dl-line"><span className="dl-line-l dl-ok"><Ic name="check-circle" size={10} /> Completed · {d.when}</span><span className="dl-line-r">{d.size}</span></div>}
      {kind === 'failed' &&
        <div className="dl-line"><span className="dl-line-l dl-err"><Ic name="alert-triangle" size={10} /> {d.reason}</span><span className="dl-line-r">{d.size}</span></div>}
    </div>);
}

function DownloadsHead({ onClose }) {
  return (
    <div className="panel-head-rail">
      <div className="dl-head-icon"><Ic name="arrow-down-to-line" size={14} /></div>
      <div className="panel-head-title">Downloads</div>
      <button className="panel-close" onClick={onClose}><Ic name="x" size={15} /></button>
    </div>);
}

function DownloadsRail({ dl, dispatch }) {
  const empty = !dl.active.length && !dl.queued.length && !dl.done.length && !dl.failed.length;
  if (empty) return (
    <div className="dl-empty">
      <div className="dl-empty-icon"><Ic name="inbox" size={22} /></div>
      <div className="dl-empty-title">No downloads</div>
      <div className="dl-empty-sub">Download a model from the list and it will show up here with live progress.</div>
    </div>);

  const sect = (key, label, items, kind) => items.length > 0 &&
    <div className="dl-sect" key={key}>
      <div className="dl-sect-head"><span className="dl-sect-lbl">{label}</span><span className="dl-sect-count">{items.length}</span></div>
      <div className="dl-list">{items.map((d) =>
        <DlRow key={d.id} d={d} kind={kind}
          onCancel={() => dispatch({ t: 'remove', kind, id: d.id })}
          onClear={() => dispatch({ t: 'remove', kind, id: d.id })}
          onRetry={() => dispatch({ t: 'retry', id: d.id })} />)}</div>
    </div>;

  return (
    <div className="dl-rail">
      {sect('active', 'Downloading', dl.active, 'active')}
      {sect('queued', 'Queued', dl.queued, 'queued')}
      {sect('failed', 'Failed', dl.failed, 'failed')}
      {sect('done', 'Completed', dl.done, 'done')}
    </div>);
}

/* ── Mode config (one fixed mode per page) ────────────────────── */
const MODE_CFG = {
  'my-models': { subPage: 'my-models',     label: 'My Models' },
  'local':     { subPage: 'explore-local', label: 'Explore · Local Models' },
  'api':       { subPage: 'explore-api',   label: 'Explore · API Models' }
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
  { icon: 'target', label: 'Specialisation', clearable: true, chips: [
    { label: 'All', defaultOn: true }, { label: 'Coding', color: 'saffron', defaultOn: true },
    { label: 'Agentic' }, { label: 'Reasoning' }, { label: 'Long ctx' }, { label: 'Vision' }, { label: 'Small' }] },
  { icon: 'sparkles', label: 'Capability', chips: [
    { label: 'tool-use', color: 'indigo', defaultOn: true }, { label: 'vision' },
    { label: 'structured' }, { label: 'embedding' }, { label: 'reasoning' }] },
  { icon: 'ruler', label: 'Size', note: '(on disk)', range: { min: 0, max: 128, step: 1, unit: ' GB', defaultMin: 0, defaultMax: 128 } },
  { icon: 'binary', label: 'Precision', note: '(bits / weight)', clearable: true, chips: [
    { label: '2' }, { label: '3' }, { label: '4', color: 'saffron', defaultOn: true }, { label: '5' }, { label: '6' }, { label: '8' }] },
  { icon: 'layers', label: 'Quant method', note: '(scheme suffix)', clearable: true, chips: [
    { label: 'K_M', color: 'saffron', defaultOn: true }, { label: 'K_S' }, { label: 'K_L' }, { label: 'K' }, { label: '_0' }, { label: 'IQ' }] },
  { icon: 'scale', label: 'License', chips: [
    { label: 'Apache-2', color: 'indigo', defaultOn: true }, { label: 'MIT' }, { label: 'Llama' }, { label: 'Gemma' }, { label: 'DeepSeek' }] }],

  'api': [
  { icon: 'activity', label: 'Status', chips: [
    { label: 'Connected', color: 'leaf', defaultOn: true }, { label: 'API key', color: 'saffron' }, { label: 'Available' }] },
  { icon: 'sparkles', label: 'Capability', chips: [
    { label: 'tool-use', color: 'indigo' }, { label: 'vision' }, { label: 'reasoning' }, { label: 'structured' }] },
  { icon: 'dollar-sign', label: 'Pricing', chips: [
    { label: 'Free', color: 'leaf' }, { label: '<$1/M' }, { label: '$1–5' }, { label: '$5+' }] },
  { icon: 'plug', label: 'API Format', chips: [
    { label: 'OpenAI-compat', color: 'indigo' }, { label: 'Native API' }] }]

};

function ModelsSidebar({ mode, orgFilters, onPickOrg, onRemoveOrg, onClearOrgs, staffOnly, onToggleStaff }) {
  const { collapsed } = useShell();
  if (mode !== 'local') {
    return (
      <>
        {FILTERS[mode].map((g) =>
        <ShellFilterGroup key={mode + '-' + g.label} icon={g.icon} label={g.label}
        note={g.note} clearable={g.clearable} chips={g.chips} range={g.range} />
        )}
      </>);
  }
  const ctxStops = [...new Set(LOCAL_MODELS.map((m) => m.ctx))].sort((a, b) => a - b);
  const anyPreset = (orgFilters && orgFilters.length) || staffOnly;
  return (
    <>
      {!collapsed &&
      <div className="sb-filter-head">
        <span className="sb-filter-title">Filters</span>
        {anyPreset && <button className="sb-clear-all" onClick={() => { onClearOrgs && onClearOrgs(); if (staffOnly) onToggleStaff(); }}><ShellIcon name="x" size={11} /> Clear all</button>}
      </div>}
      {FILTERS.local.map((g) =>
      <React.Fragment key={'local-' + g.label}>
        {g.label === 'Browse'
          ? <BrowseGroup staffOnly={staffOnly} onToggleStaff={onToggleStaff} />
          : <ShellFilterGroup icon={g.icon} label={g.label} note={g.note} clearable={g.clearable} chips={g.chips} range={g.range} />}
        {g.label === 'Browse' && <>
          <PublisherGroup orgFilters={orgFilters} onPick={onPickOrg} onRemove={onRemoveOrg} />
          <FormatSourceGroup />
        </>}
        {g.label === 'Size' && <ContextGroup stops={ctxStops} />}
      </React.Fragment>
      )}
    </>);
}

/* ── List rows ────────────────────────────────────────────────── */
function MyRow({ item, active, onClick }) {
  let body;
  if (item.type === 'local-file' || item.type === 'model-alias') {
    const isAlias = item.type === 'model-alias';
    const icon = isAlias ? 'tag' : 'hard-drive';
    body = <>
      <div className={'my-icon-box ' + (isAlias ? 'my-icon-model-alias' : 'my-icon-local-file')}><Ic name={icon} size={16} /></div>
      <div className="my-body">
        <div className="my-name">{isAlias ? item.repo : item.org + '/' + item.repo}</div>
        <div className="my-sub">{item.filename}</div>
      </div>
      <span className={'type-badge ' + (isAlias ? 'tb-alias' : 'tb-hf')}><Ic name={icon} size={9} />{isAlias ? 'Model Alias' : 'Local File'}</span>
    </>;
  } else if (item.type === 'fallback') {
    const parts = item.steps.map((s) => s.aliasName);
    const preview = parts.length <= 2 ? parts.join('  →  ') : parts[0] + '  →  …  →  ' + parts[parts.length - 1];
    const enabled = item.steps.filter((s) => s.enabled !== false).length;
    const summary = enabled === item.steps.length ?
    item.steps.length + ' steps · tried in order on error' :
    enabled + ' of ' + item.steps.length + ' steps active · ' + (item.steps.length - enabled) + ' disabled';
    body = <>
      <div className="my-icon-box my-icon-fallback"><Ic name="route" size={16} /></div>
      <div className="my-body">
        <div className="my-name">{item.name}</div>
        <div className="my-sub" style={{ fontFamily: 'var(--font-mono)' }}>{preview}</div>
        <div className="my-exposed">{summary}</div>
      </div>
      <span className="type-badge tb-fallback"><Ic name="route" size={9} />Fallback</span>
    </>;
  } else {
    body = <>
      <div className="my-icon-box my-icon-api-model"><Ic name="at-sign" size={16} /></div>
      <div className="my-body">
        <div className="my-name">{item.name}</div>
        <div className="my-sub">{item.baseUrl}</div>
        <div className="my-exposed">{item.modelsExposed} model{item.modelsExposed > 1 ? 's' : ''} exposed</div>
      </div>
      <div className="my-api-right">
        <span className="my-provider-badge">{item.provider}</span>
        {item.keyStatus === 'connected' ?
        <span className="my-key-ok"><Ic name="check-circle" size={10} />connected</span> :
        <span className="my-key-no"><Ic name="key" size={10} />no key</span>}
      </div>
    </>;
  }
  return <div className={'my-card' + (active ? ' active' : '')} onClick={onClick}><RowLink onActivate={onClick} label={'Open ' + (item.name || item.repo || 'model')} />{body}</div>;
}

function LocalRow({ m, active, onClick, cols, sortKey, onPickOrg, idx }) {
  const stat = (key, value, lbl, extra) => (
    <div className={'m-stat' + (sortKey === key ? ' sorted' : '')}>
      <div className="m-stat-num">{value}</div>
      {extra}
      <div className="m-stat-lbl">{lbl}</div>
    </div>);
  return (
    <div className={'m-row m-row-local' + (active ? ' active' : '')} onClick={onClick}>
      <RowLink onActivate={onClick} label={'Open ' + m.org + '/' + m.repo} />
      <div className="m-num">#{idx}</div>
      <div className="m-body">
        <div className="m-name">
          <button className="m-org m-org-link" onClick={(e) => { e.stopPropagation(); onPickOrg(m.org); }} title={'Filter by ' + m.org}>{m.org}</button>
          <span className="m-sep">/</span><span className="m-repo">{m.repo}</span>
          {m.owner_verified && <VerifiedBadge />}
          {m.staff_pick && <StaffBadge small />}
        </div>
        <div className="m-tags">{m.tags.map((t) => <Tag key={t} t={t} />)}</div>
      </div>
      <div className="m-stats">
        {cols.human && stat('human', m.score, 'HUMAN',
          <div className="m-score-bar"><div className="m-score-fill" style={{ width: Math.round(m.score) + '%' }} /></div>)}
        {cols.downloads && stat('downloads', m.dlLabel, 'DOWNLOADS')}
        {cols.likes && stat('likes', m.likeLabel, 'LIKES')}
      </div>
      <div className="m-row-actions" />
    </div>);

}

function ApiRow({ p, active, onClick }) {
  const sc = STATUS_CFG[p.status] || STATUS_CFG.available;
  const color = PROV_COLORS[p.slug] || '#888';
  const suffix = p.models >= 100 ? '+' : '';
  return (
    <div className={'m-row' + (active ? ' active' : '')} onClick={onClick}>
      <RowLink onActivate={onClick} label={'Open ' + p.provider} />
      <div className="m-num">#{p.rank}</div>
      <div className="prov-avatar" style={{ background: color + '1a', color, border: '1.5px solid ' + color + '40' }}>{p.provider.slice(0, 2).toUpperCase()}</div>
      <div className="m-body">
        <div className="m-name" style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          {p.provider}
          <span className={'status-badge ' + sc.cls}><Ic name={sc.icon} size={9} />{sc.lbl}</span>
        </div>
        <div className="m-meta">{p.meta}</div>
        <div className="m-tags">{p.tags.map((t) => <Tag key={t} t={t} />)}</div>
      </div>
      <div className="m-right">
        <div className="m-score" style={{ minWidth: 44 }}>
          <div className="m-score-num" style={{ fontSize: 16 }}>{p.models}{suffix}</div>
          <div className="m-score-lbl">MODELS</div>
        </div>
        {p.status === 'connected' || p.status === 'api-key' ?
        <button className="act act-use" onClick={(e) => e.stopPropagation()}><Ic name="settings-2" size={11} /> Manage</button> :
        <button className="act act-connect" onClick={(e) => e.stopPropagation()}><Ic name="plug-zap" size={11} /> Connect</button>}
      </div>
    </div>);

}

/* ── Main content (toolbar + list) ────────────────────────────── */
const PLACEHOLDER = {
  'my-models': 'Search by alias, repo, filename, base URL — ⌘K',
  'local': 'Search HuggingFace repos — ⌘K',
  'api': 'Search API providers — ⌘K'
};

function ModelsMain({ mode, sel, onSelect, density, showTags, showScore, onShowDownloads, downloadsOpen, dlCount,
  sort, onSort, cols, onToggleCol, orgFilters, staffOnly, onPickOrg, onRemoveOrg, onClearOrgs, onToggleStaff }) {
  const { openRail } = useShell();
  useListKeyNav({ rootSelector: '.model-list', rowSelector: '.m-row, .my-card' });
  const [q, setQ] = useState('');
  const pick = (kind, item, idx) => {onSelect({ kind, item, idx });openRail();};

  const listClass = ['model-list',
  mode === 'my-models' ? 'my-mode' : '',
  density === 'compact' ? 'compact' : '',
  !showTags ? 'hide-tags' : '',
  !showScore ? 'hide-scorelbl' : ''].
  filter(Boolean).join(' ');

  /* local: apply publisher + recommended presets, then backend-style sort */
  let localRows = LOCAL_MODELS;
  if (mode === 'local') {
    if (orgFilters && orgFilters.length) localRows = localRows.filter((m) => orgFilters.some((o) => o.toLowerCase() === m.org.toLowerCase()));
    if (staffOnly) localRows = localRows.filter((m) => m.staff_pick);
    const val = (m) => sort.key === 'downloads' ? m.dlNum : sort.key === 'likes' ? m.likeNum : m.score;
    localRows = [...localRows].sort((a, b) => sort.order === 'asc' ? val(a) - val(b) : val(b) - val(a));
  }
  const sortLabel = { human: 'Human evals', downloads: 'Downloads', likes: 'Likes' }[sort.key];
  const anyFilter = (orgFilters && orgFilters.length) || staffOnly;

  return (
    <div className="models-main">
      <div className="toolbar" style={{ padding: "12px 16px" }}>
        <ShellSearch value={q} onChange={setQ} placeholder={PLACEHOLDER[mode]} kbd="⌘K" />
        <button className={'l-iconbtn' + (downloadsOpen ? ' on' : '')} title="Downloads" onClick={onShowDownloads}>
          <Ic name="arrow-down-to-line" size={15} />
          {dlCount > 0 && <span className="dl-badge">{dlCount}</span>}
        </button>
      </div>

      {mode === 'local' &&
      <div className="result-bar">
          <span className="result-count">{localRows.length} {localRows.length === 1 ? 'repository' : 'repositories'}</span>
          <span className="result-sort">sorted by <strong>{sortLabel}</strong> · {sort.order === 'asc' ? 'ascending' : 'descending'}</span>
        </div>
      }

      {mode === 'local' &&
      <div className="list-head list-head-local">
          <div className="lh-num lh-label">#</div>
          <div className="lh-model lh-label" style={{ paddingLeft: 4 }}>Repository</div>
          <div className="lh-stats">
            {cols.human && <SortHeaderCell label="Human" k="human" sortKey={sort.key} sortOrder={sort.order} onSort={onSort} />}
            {cols.downloads && <SortHeaderCell label="Downloads" k="downloads" sortKey={sort.key} sortOrder={sort.order} onSort={onSort} />}
            {cols.likes && <SortHeaderCell label="Likes" k="likes" sortKey={sort.key} sortOrder={sort.order} onSort={onSort} />}
          </div>
          <div className="lh-cols"><ColumnsMenu cols={cols} onToggle={onToggleCol} compact /></div>
        </div>
      }
      {mode === 'api' &&
      <div className="list-head">
          <div className="lh-num lh-label">#</div>
          <div className="lh-model lh-label" style={{ paddingLeft: 56 }}>Provider</div>
          <div className="lh-score" style={{ minWidth: 68 }}>Models</div>
          <div className="lh-action" />
        </div>
      }

      <div className={listClass}>
        {mode === 'my-models' && MY_MODELS.map((item, i) =>
        <MyRow key={item.id} item={item} active={sel && sel.kind === 'my' && sel.idx === i} onClick={() => pick('my', item, i)} />)}
        {mode === 'local' && <>
          {localRows.map((m, i) =>
          <LocalRow key={m.org + m.repo} m={m} idx={i + 1} cols={cols} sortKey={sort.key}
            active={sel && sel.kind === 'local' && sel.item.org === m.org && sel.item.repo === m.repo}
            onClick={() => pick('local', m, i)} onPickOrg={onPickOrg} />)}
          {localRows.length === 0 &&
            <div className="list-empty"><Ic name="search-x" size={22} /><div>No repositories match these filters.</div></div>}
          {localRows.length > 0 && <button className="load-more"><Ic name="chevrons-down" size={14} /> Load more</button>}
        </>}
        {mode === 'api' && <>
          {API_PROVIDERS.map((p, i) =>
          <ApiRow key={p.slug} p={p} active={sel && sel.kind === 'api' && sel.idx === i} onClick={() => pick('api', p, i)} />)}
          <button className="load-more"><Ic name="chevrons-down" size={14} /> Load more</button>
        </>}
      </div>
    </div>);

}

/* ── Detail rail ──────────────────────────────────────────────── */
function DetailHeader({ sel, onDeselect, onPickOrg }) {
  if (!sel) return null;

  const { kind, item } = sel;
  let badge,title,mono = false,extra = null;
  if (kind === 'my') {
    if (item.type === 'model-alias') {badge = <span className="type-badge tb-alias"><Ic name="tag" size={9} />Model Alias</span>;title = item.repo;mono = true;} else
    if (item.type === 'local-file') {badge = <span className="type-badge tb-hf"><Ic name="hard-drive" size={9} />Local File</span>;title = item.detail.repo + ':' + (item.filename.split('.').slice(-2, -1)[0] || 'gguf');mono = true;} else
    if (item.type === 'fallback') {badge = <span className="type-badge tb-fallback"><Ic name="route" size={9} />Fallback</span>;title = item.name;mono = true;} else
    {badge = <span className="type-badge tb-api"><Ic name="at-sign" size={9} />API Model</span>;title = item.name;mono = true;}
  } else if (kind === 'local') {
    badge = <span className="type-badge tb-hf"><Ic name="hard-drive" size={9} />hf-repo</span>;
    title = (
      <span className="ph-repo">
        <button className="ph-org" onClick={() => onPickOrg && onPickOrg(item.org)} title={'Filter by ' + item.org}>{item.org}</button>
        <span style={{ opacity: .4 }}>/</span>{item.repo}
        {item.owner_verified && <VerifiedBadge />}
      </span>);
    extra = <button className="panel-copy" title="Copy repo id"><Ic name="copy" size={13} /></button>;
  } else {
    const color = PROV_COLORS[item.slug] || '#888';
    badge = <span className="prov-avatar" style={{ width: 26, height: 26, borderRadius: 7, fontSize: 11, marginRight: 0, background: color + '1a', color, border: '1.5px solid ' + color + '40' }}>{item.provider.slice(0, 2).toUpperCase()}</span>;
    title = item.provider;
  }
  return (
    <div className="panel-head-rail">
      {badge}
      <div className={'panel-head-title' + (mono ? ' ph-mono' : '')}>{title}</div>
      {extra}
      <button className="panel-close" onClick={onDeselect}><Ic name="x" size={15} /></button>
    </div>);

}

function SpecTable({ rows }) {
  return <div className="spec-table">{rows.map((s) =>
    <div className="spec-row" key={s.k}><span className="spec-k">{s.k}</span><span className="spec-v" style={s.small ? { fontSize: 11, wordBreak: 'break-all' } : null}>{s.v}</span></div>)}</div>;
}

function DetailBody({ sel, tab, setTab, starred, toggleStar, onPickOrg }) {
  if (!sel) return null;

  const { kind, item } = sel;

  /* My Models */
  if (kind === 'my') {
    if (item.type === 'local-file' || item.type === 'model-alias') {
      return (
        <div className="panel-body">
          <div className="p-sec-lbl" style={{ marginBottom: 8 }}>File</div>
          <SpecTable rows={[
          { k: 'repo', v: item.detail.repo, small: true },
          { k: 'filename', v: item.detail.filename, small: true },
          { k: 'snapshot', v: item.detail.snapshot, small: true }]
          } />
          <div style={{ marginTop: 14, fontSize: 12, color: 'hsl(var(--muted-foreground))', lineHeight: 1.65 }}>{item.detail.note}</div>
        </div>);

    }
    if (item.type === 'fallback') {
      const enabled = item.steps.filter((s) => s.enabled !== false).length;
      return <>
        <div className="panel-body">
          <div className="panel-lead panel-sub"><span className="panel-stat"><Ic name="layers" size={10} />{enabled} of {item.steps.length} steps active</span></div>
          <div className="p-sec-lbl" style={{ marginBottom: 8 }}>Routing chain</div>
          <div className="fb-chain">
            {item.steps.map((s, i) => {
              const on = s.enabled !== false;
              const cls = s.aliasType === 'api-model' ? 'tb-api' : s.aliasType === 'model-alias' ? 'tb-alias' : 'tb-hf';
              const ico = s.aliasType === 'api-model' ? 'at-sign' : s.aliasType === 'model-alias' ? 'tag' : 'hard-drive';
              const lbl = s.aliasType === 'api-model' ? 'API Model' : s.aliasType === 'model-alias' ? 'Model Alias' : 'Local File';
              return (
                <React.Fragment key={i}>
                  <div className={'fb-step' + (on ? '' : ' disabled')}>
                    <div className="fb-step-num">{i + 1}</div>
                    <div className="fb-step-body">
                      <div className="fb-step-name">{s.aliasName}{!on && <span className="fb-disabled-tag">disabled</span>}</div>
                      {s.model && <div className="fb-step-model">→ {s.model}</div>}
                      <div className="fb-step-meta">
                        <span className={'type-badge ' + cls}><Ic name={ico} size={9} />{lbl}</span>
                        {s.provider && <span className="my-provider-badge">{s.provider}</span>}
                      </div>
                    </div>
                  </div>
                  {i < item.steps.length - 1 && <div className={'fb-arrow' + (on ? '' : ' dim')}><Ic name="arrow-down" size={11} />on error, try next</div>}
                </React.Fragment>);

            })}
          </div>
          <div style={{ marginTop: 16, fontSize: 12, color: 'hsl(var(--muted-foreground))', lineHeight: 1.65 }}>{item.detail.note}</div>
          <div className="p-sec-lbl" style={{ marginTop: 18, marginBottom: 8 }}>Behavior</div>
          <SpecTable rows={[
          { k: 'on error', v: 'try next step', small: true }, { k: 'on success', v: 'return immediately', small: true },
          { k: 'disabled steps', v: 'skipped at runtime', small: true }, { k: 'all failed', v: 'surface final error', small: true }]
          } />
        </div>
        <div className="panel-foot">
          <a href="Create Fallback Model.html" className="btn-add" style={{ background: 'var(--c-teal-text)', color: '#fff' }}><Ic name="pencil" size={14} /> Edit model router</a>
        </div>
      </>;
    }
    /* api-model */
    return (
      <div className="panel-body">
        <div className="panel-lead panel-sub">
          {item.keyStatus === 'connected' ?
          <span className="my-key-ok"><Ic name="check-circle" size={10} />connected</span> :
          <span className="my-key-no"><Ic name="key" size={10} />no key</span>}
        </div>
        <div className="p-sec-lbl" style={{ marginBottom: 8 }}>Connection</div>
        <SpecTable rows={[
        { k: 'base URL', v: item.baseUrl, small: true },
        { k: 'provider', v: <span className="my-provider-badge">{item.provider}</span> },
        { k: 'models', v: item.modelsExposed + ' exposed' }]
        } />
        <div className="p-sec-lbl" style={{ marginTop: 16, marginBottom: 8 }}>Models</div>
        <div className="model-item-list">{item.detail.models.map((m) => <div className="model-item" key={m}><span className="model-item-name">{m}</span></div>)}</div>
      </div>);

  }

  /* Local */
  if (kind === 'local') {
    const d = item.detail;
    const key = item.org + '/' + item.repo;
    const rec = d.quants.find((q) => q.rec) || d.quants[0];
    return <>
      <div className="detail-metabar">
        <MetaChips m={item} />
        <div className="metabar-row">
          <div className="metabar-stats">
            <span className="mb-stat" title="Downloads"><Ic name="download" size={11} />{item.dlLabel}</span>
            <span className="mb-stat" title="Likes"><Ic name="heart" size={11} />{item.likeLabel}</span>
            <span className="mb-stat" title={'Updated ' + fmtAbs(item.updated)}><Ic name="calendar" size={11} />{relTime(item.updated)}</span>
            {item.staff_pick && <StaffBadge small />}
          </div>
          <RunBadge quants={d.quants} />
        </div>
      </div>
      <div className="panel-tabs">
        <button className={'ptab' + (tab === 'overview' ? ' on' : '')} onClick={() => setTab('overview')}>Overview</button>
        <button className={'ptab' + (tab === 'quants' ? ' on' : '')} onClick={() => setTab('quants')}>Download options ({item.quants})</button>
        <button className={'ptab' + (tab === 'readme' ? ' on' : '')} onClick={() => setTab('readme')}>README</button>
      </div>
      <div className="panel-body">
        {tab === 'overview' && <>
          <div className="p-section"><div className="p-sec-lbl">Capabilities</div><div className="cap-chips">{d.caps.map((c) => <Tag key={c} t={c} big />)}</div></div>
          <div className="p-section"><div className="p-sec-lbl">Specs</div><SpecTable rows={d.specs} /></div>
          <MoreFrom org={item.org} items={d.moreFrom} onPickOrg={onPickOrg} />
        </>}
        {tab === 'quants' && <div className="p-section">
          <div className="p-sec-lbl dlopt-head">Download options <span className="host-note" title={LM_HOST.label}><Ic name="cpu" size={10} /> {LM_HOST.vramGB} GB VRAM · {LM_HOST.ramGB} GB RAM</span></div>
          <div className="dlopt-list">{d.quants.map((q) => {
            const fit = quantFit(parseGB(q.size));
            return (
              <div className={'dlopt-row' + (q.rec ? ' rec' : '')} key={q.name}>
                <div className="dlopt-main">
                  <span className="dlopt-name">{q.name}</span>
                  {q.rec && <span className="rec-badge"><Ic name="thumbs-up" size={10} /><span className="rec-lbl">Recommended</span></span>}
                </div>
                <div className="dlopt-side">
                  <FitPill fit={fit} full />
                  <span className="dlopt-size">{q.size}</span>
                  <button className="dlopt-dl-icon" title={'Download ' + q.name + ' · ' + q.size} onClick={(e) => e.stopPropagation()}><Ic name="download" size={14} /></button>
                </div>
              </div>);
          })}</div>
        </div>}
        {tab === 'readme' && <div className="readme-wrap"><MarkdownView src={d.readme} /></div>}
      </div>
      <div className="panel-foot">
        <button className="btn-add"><Ic name="circle-plus" size={14} /> Add to Bodhi</button>
        <div className="panel-foot-row">
          <button className="btn-pull"><Ic name="download" size={12} /> Download {rec.name} · {rec.size}</button>
          <button className={'btn-star' + (starred.has(key) ? ' starred' : '')} onClick={() => toggleStar(key)}><Ic name="star" size={14} /></button>
        </div>
      </div>
    </>;
  }

  /* API provider */
  const d = item.detail,sc = STATUS_CFG[item.status] || STATUS_CFG.available;
  const suffix = item.models >= 100 ? '+' : '';
  return <>
    <div className="panel-tabs">
      <button className={'ptab' + (tab === 'overview' ? ' on' : '')} onClick={() => setTab('overview')}>Overview</button>
      <button className={'ptab' + (tab === 'models' ? ' on' : '')} onClick={() => setTab('models')}>Models ({item.models}{suffix})</button>
    </div>
    <div className="panel-body">
      {tab === 'overview' ? <>
        <div className="panel-lead" style={{ display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'wrap' }}>
          <span className={'status-badge ' + sc.cls}><Ic name={sc.icon} size={9} />{sc.lbl}</span>
          <span className="panel-stat"><Ic name="layers" size={10} />{item.models}{suffix} models</span>
        </div>
        <div className="p-section"><div className="p-sec-lbl">Capabilities</div><div className="cap-chips">{d.caps.map((c) => <Tag key={c} t={c} big />)}</div></div>
        <div className="p-section"><div className="p-sec-lbl">Provider Info</div><SpecTable rows={d.specs} /></div>
      </> :
      <div className="p-section"><div className="p-sec-lbl">Available Models</div>
          <div className="model-item-list">{d.modelList.map((m) => <div className="model-item" key={m}><span className="model-item-name">{m}</span></div>)}</div>
        </div>
      }
    </div>
    <div className="panel-foot">
      {item.status === 'connected' || item.status === 'api-key' ?
      <button className="btn-add" style={{ background: 'hsl(var(--foreground))', color: 'hsl(var(--background))' }}><Ic name="settings-2" size={14} /> Manage Connection</button> :
      <button className="btn-add"><Ic name="plug-zap" size={14} /> Connect Provider</button>}
    </div>
  </>;
}

/* ── Tweaks panel (host protocol) ─────────────────────────────── */
/* ── Root ─────────────────────────────────────────────────────── */
function ModelsApp() {
  const mode = window.MODELS_MODE || 'my-models';
  const cfg = MODE_CFG[mode];
  const [sel, setSel] = useState(null);
  const [tab, setTab] = useState('overview');
  const [starred, setStarred] = useState(() => new Set());
  const [showDownloads, setShowDownloads] = useState(false);
  const [dl, setDl] = useState(DOWNLOADS_INIT);
  const density = 'cozy';
  const showTags = true;
  const showScore = true;

  /* local explore: sort (maps to backend sort + sort_order), visible stat
     columns, and the publisher / staff-pick presets */
  const [sort, setSort] = useState({ key: 'human', order: 'desc' });
  const [cols, setCols] = useState({ human: true, downloads: true, likes: true });
  const [orgFilters, setOrgFilters] = useState([]);
  const [staffOnly, setStaffOnly] = useState(false);
  const onSort = (k) => setSort((s) => s.key === k ? { key: k, order: s.order === 'desc' ? 'asc' : 'desc' } : { key: k, order: 'desc' });
  const onToggleCol = (k) => setCols((c) => ({ ...c, [k]: !c[k] }));
  const onPickOrg = (org) => { if (!org) return; setShowDownloads(false); setOrgFilters((prev) => prev.some((o) => o.toLowerCase() === org.toLowerCase()) ? prev : [...prev, org]); };
  const onRemoveOrg = (org) => setOrgFilters((prev) => prev.filter((o) => o !== org));
  const onClearOrgs = () => setOrgFilters([]);
  const onToggleStaff = () => setStaffOnly((v) => !v);

  /* default selection on desktop */
  useEffect(() => {
    if (window.matchMedia('(max-width:767px)').matches) return;
    if (mode === 'local') setSel({ kind: 'local', item: LOCAL_MODELS[0], idx: 0 });
    else if (mode === 'api') setSel({ kind: 'api', item: API_PROVIDERS[0], idx: 0 });
    else setSel({ kind: 'my', item: MY_MODELS[0], idx: 0 });
  }, []);

  /* live progress — ticks active pulls while the panel is open */
  useEffect(() => {
    if (!showDownloads) return;
    const t = setInterval(() => {
      setDl((prev) => {
        if (!prev.active.length) return prev;
        const stillActive = [];const finished = [];
        prev.active.forEach((d) => {
          const np = d.pct + d.rate + Math.random() * d.rate * 0.5;
          const remMB = d.totalMB * (100 - np) / 100;
          const rateMB = parseFloat(d.speed) || 12;
          const secs = Math.max(0, Math.round(remMB / rateMB));
          const eta = secs > 90 ? Math.round(secs / 60) + ' min left' : secs + 's left';
          if (np >= 100) finished.push({ id: d.id, org: d.org, repo: d.repo, file: d.file, size: fmtSize(d.totalMB), when: 'Just now' });
          else stillActive.push({ ...d, pct: np, eta });
        });
        let queued = prev.queued, promoted = [];
        if (finished.length && queued.length) {
          const first = queued[0];queued = queued.slice(1);
          promoted.push({ id: first.id, org: first.org, repo: first.repo, file: first.file, pct: 0.4, totalMB: gbToMb(first.total), rate: 0.5, speed: '12.0 MB/s' });
        }
        return { ...prev, active: [...stillActive, ...promoted], queued, done: [...finished, ...prev.done] };
      });
    }, 900);
    return () => clearInterval(t);
  }, [showDownloads]);

  const dlDispatch = (a) => {
    if (a.t === 'remove') setDl((p) => ({ ...p, [a.kind]: p[a.kind].filter((x) => x.id !== a.id) }));
    else if (a.t === 'retry') setDl((p) => {
      const item = p.failed.find((x) => x.id === a.id);if (!item) return p;
      return { ...p, failed: p.failed.filter((x) => x.id !== a.id),
        active: [...p.active, { id: item.id, org: item.org, repo: item.repo, file: item.file, pct: 0.3, totalMB: gbToMb(item.size), rate: 0.5, speed: '12.0 MB/s' }] };
    });
  };

  const onSelect = (s) => {setSel(s);setTab('overview');setShowDownloads(false);};
  const openDownloads = () => {setSel(null);setShowDownloads((v) => !v);};
  const toggleStar = (key) => setStarred((prev) => {const n = new Set(prev);n.has(key) ? n.delete(key) : n.add(key);return n;});

  const railContent = showDownloads ?
    <DownloadsRail dl={dl} dispatch={dlDispatch} /> :
    sel ? <DetailBody sel={sel} tab={tab} setTab={setTab} starred={starred} toggleStar={toggleStar} onPickOrg={onPickOrg} /> : null;
  const railHead = showDownloads ?
    <DownloadsHead onClose={() => setShowDownloads(false)} /> :
    sel ? <DetailHeader sel={sel} onDeselect={() => setSel(null)} onPickOrg={onPickOrg} /> : undefined;

  return <>
    <AppShell
      section="models" subPage={cfg.subPage}
      resizeKey="models"
      breadcrumb={[{ label: 'Bodhi', href: 'Bodhi Chat.html' }, { label: 'Models', href: 'Bodhi Models.html' }, { label: cfg.label, current: true }]}
      sidebar={<ModelsSidebar mode={mode} orgFilters={orgFilters} onPickOrg={onPickOrg} onRemoveOrg={onRemoveOrg} onClearOrgs={onClearOrgs} staffOnly={staffOnly} onToggleStaff={onToggleStaff} />}
      contentClass="flush" mainScroll={false} railScroll={false}
      rail={railContent}
      railHeader={railHead}>
      
      <ModelsMain mode={mode} sel={sel} onSelect={onSelect} density={density} showTags={showTags} showScore={showScore}
        onShowDownloads={openDownloads} downloadsOpen={showDownloads} dlCount={dl.active.length + dl.queued.length}
        sort={sort} onSort={onSort} cols={cols} onToggleCol={onToggleCol}
        orgFilters={orgFilters} staffOnly={staffOnly} onPickOrg={onPickOrg} onRemoveOrg={onRemoveOrg} onClearOrgs={onClearOrgs} onToggleStaff={onToggleStaff} />
    </AppShell>
  </>;
}

ReactDOM.createRoot(document.getElementById('root')).render(<ModelsApp />);