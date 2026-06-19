/* ═══════════════════════════════════════════════════════════════
   Bodhi Models — page app
   bodhi-models-app.jsx   (load after bodhi-app-shell.jsx + bodhi-models-data.js)
═══════════════════════════════════════════════════════════════ */
const { useState, useEffect } = React;
const { MY_MODELS, LOCAL_MODELS, API_PROVIDERS, TAG_MAP, STATUS_CFG, PROV_COLORS } = window.MODELS_DATA;

const Ic = ShellIcon;
const tagClass = (t) => 'tag ' + (TAG_MAP[t] || 'tag-muted');
const Tag = ({ t, big }) => <span className={tagClass(t)} style={big ? { fontSize: 12, padding: '3px 9px' } : null}>{t}</span>;

/* ── Mode config (one fixed mode per page) ────────────────────── */
const MODE_CFG = {
  'my-models': { subPage: 'my-models',     label: 'My Models' },
  'local':     { subPage: 'explore-local', label: 'Explore · Local Models' },
  'api':       { subPage: 'explore-api',   label: 'Explore · API Models' }
};


const FILTERS = {
  'my-models': [
  { icon: 'shapes', label: 'Type', chips: [
    { label: 'Local File', color: 'saffron' }, { label: 'Model Alias', color: 'lotus' },
    { label: 'API Model', color: 'indigo' }, { label: 'Fallback', color: 'teal' }] },
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
  { icon: 'ruler', label: 'Size', chips: ['<5 GB', '5–15 GB', '>15 GB', 'ctx <32k', 'ctx 32k+'].map((l) => ({ label: l })) },
  { icon: 'binary', label: 'Quant Format', chips: [
    { label: 'Q2_K' }, { label: 'Q4_K_M', color: 'saffron', defaultOn: true }, { label: 'Q6_K' }, { label: 'Q8_0' }] },
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

function ModelsSidebar({ mode }) {
  return (
    <>
      {FILTERS[mode].map((g) =>
      <ShellFilterGroup key={mode + '-' + g.label} icon={g.icon} label={g.label}
      note={g.note} clearable={g.clearable} chips={g.chips} range={g.range} />
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

function LocalRow({ m, active, onClick }) {
  return (
    <div className={'m-row' + (active ? ' active' : '')} onClick={onClick}>
      <RowLink onActivate={onClick} label={'Open ' + m.org + '/' + m.repo} />
      <div className="m-num">#{m.rank}</div>
      <div className="m-body">
        <div className="m-name"><span className="m-org">{m.org}</span><span className="m-sep">/</span><span className="m-repo">{m.repo}</span></div>
        <div className="m-meta">{m.meta}</div>
        <div className="m-tags">{m.tags.map((t) => <Tag key={t} t={t} />)}</div>
      </div>
      <div className="m-right">
        <div className="m-score">
          <div className="m-score-num">{m.score}</div>
          <div className="m-score-bar"><div className="m-score-fill" style={{ width: Math.round(m.score) + '%' }} /></div>
          <div className="m-score-lbl">HUMAN</div>
        </div>
        <button className="act act-pull" onClick={(e) => e.stopPropagation()}><Ic name="download" size={11} /> Pull</button>
      </div>
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

function ModelsMain({ mode, sel, onSelect, density, showTags, showScore }) {
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

  return (
    <div className="models-main">
      <div className="toolbar" style={{ padding: "12px 16px" }}>
        <ShellSearch value={q} onChange={setQ} placeholder={PLACEHOLDER[mode]} kbd="⌘K" />
        <button className="l-iconbtn" title="Downloads"><Ic name="arrow-down-to-line" size={15} /></button>
      </div>

      {mode === 'local' &&
      <div className="chip-row">
          <span className="chip">Specialisation: Coding <button className="chip-x"><Ic name="x" size={9} /></button></span>
          <span className="chip">capability: tool-use <button className="chip-x"><Ic name="x" size={9} /></button></span>
          <span className="chip">license: Apache-2 <button className="chip-x"><Ic name="x" size={9} /></button></span>
          <span className="chip">format: GGUF <button className="chip-x"><Ic name="x" size={9} /></button></span>
          <button className="clear-btn">clear all</button>
        </div>
      }

      {mode === 'local' &&
      <div className="list-head">
          <div className="lh-num lh-label">#</div>
          <div className="lh-model lh-label" style={{ paddingLeft: 4 }}>Repository</div>
          <div className="lh-score">Score <Ic name="chevrons-up-down" size={10} /></div>
          <div className="lh-action" />
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
          {LOCAL_MODELS.map((m, i) =>
          <LocalRow key={m.org + m.repo} m={m} active={sel && sel.kind === 'local' && sel.idx === i} onClick={() => pick('local', m, i)} />)}
          <button className="load-more"><Ic name="chevrons-down" size={14} /> Load more</button>
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
function DetailHeader({ sel, onDeselect }) {
  if (!sel) return null;

  const { kind, item } = sel;
  let badge,title,mono = false;
  if (kind === 'my') {
    if (item.type === 'model-alias') {badge = <span className="type-badge tb-alias"><Ic name="tag" size={9} />Model Alias</span>;title = item.repo;mono = true;} else
    if (item.type === 'local-file') {badge = <span className="type-badge tb-hf"><Ic name="hard-drive" size={9} />Local File</span>;title = item.detail.repo + ':' + (item.filename.split('.').slice(-2, -1)[0] || 'gguf');mono = true;} else
    if (item.type === 'fallback') {badge = <span className="type-badge tb-fallback"><Ic name="route" size={9} />Fallback</span>;title = item.name;mono = true;} else
    {badge = <span className="type-badge tb-api"><Ic name="at-sign" size={9} />API Model</span>;title = item.name;mono = true;}
  } else if (kind === 'local') {
    badge = <span className="type-badge tb-hf"><Ic name="hard-drive" size={9} />hf-repo</span>;
    title = <span><span style={{ opacity: .5, fontWeight: 500 }}>{item.org}/</span>{item.repo}</span>;
  } else {
    const color = PROV_COLORS[item.slug] || '#888';
    badge = <span className="prov-avatar" style={{ width: 26, height: 26, borderRadius: 7, fontSize: 11, marginRight: 0, background: color + '1a', color, border: '1.5px solid ' + color + '40' }}>{item.provider.slice(0, 2).toUpperCase()}</span>;
    title = item.provider;
  }
  return (
    <div className="panel-head-rail">
      {badge}
      <div className={'panel-head-title' + (mono ? ' ph-mono' : '')}>{title}</div>
      <button className="panel-close" onClick={onDeselect}><Ic name="x" size={15} /></button>
    </div>);

}

function SpecTable({ rows }) {
  return <div className="spec-table">{rows.map((s) =>
    <div className="spec-row" key={s.k}><span className="spec-k">{s.k}</span><span className="spec-v" style={s.small ? { fontSize: 11, wordBreak: 'break-all' } : null}>{s.v}</span></div>)}</div>;
}

function DetailBody({ sel, tab, setTab, starred, toggleStar }) {
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
          <a href="Create Fallback Model.html" className="btn-add" style={{ background: 'var(--c-teal-text)', color: '#fff' }}><Ic name="pencil" size={14} /> Edit fallback alias</a>
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
    const stats = [
    d.date && { i: 'calendar', v: d.date }, d.dl && { i: 'download', v: d.dl }, d.stars && { i: 'star', v: d.stars }].
    filter(Boolean);
    const key = item.org + '/' + item.repo;
    return <>
      <div className="panel-tabs">
        <button className={'ptab' + (tab === 'overview' ? ' on' : '')} onClick={() => setTab('overview')}>Overview</button>
        <button className={'ptab' + (tab === 'quants' ? ' on' : '')} onClick={() => setTab('quants')}>Quants ({item.quants})</button>
      </div>
      <div className="panel-body">
        {tab === 'overview' ? <>
          {stats.length > 0 && <div className="panel-lead panel-sub">{stats.map((s) => <span className="panel-stat" key={s.i}><Ic name={s.i} size={10} />{s.v}</span>)}</div>}
          <div className="p-section"><div className="p-sec-lbl">Capabilities</div><div className="cap-chips">{d.caps.map((c) => <Tag key={c} t={c} big />)}</div></div>
          <div className="p-section"><div className="p-sec-lbl">Specs</div><SpecTable rows={d.specs} /></div>
        </> :
        <div className="p-section"><div className="p-sec-lbl">Available Quantizations</div>
            <div className="quant-list">{d.quants.map((q) =>
            <div className="quant-row" key={q.name}>
                <span className="quant-name">{q.name}</span><span className="quant-size">{q.size}</span>
                <button className="act act-pull" style={{ height: 26, fontSize: 11, padding: '0 8px' }} onClick={(e) => e.stopPropagation()}><Ic name="download" size={11} /> Pull</button>
              </div>)}</div>
          </div>
        }
      </div>
      <div className="panel-foot">
        <button className="btn-add"><Ic name="circle-plus" size={14} /> Add to Bodhi</button>
        <div className="panel-foot-row">
          <button className="btn-pull"><Ic name="download" size={12} /> Pull best quant</button>
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
  const density = 'cozy';
  const showTags = true;
  const showScore = true;

  /* default selection on desktop */
  useEffect(() => {
    if (window.matchMedia('(max-width:767px)').matches) return;
    if (mode === 'local') setSel({ kind: 'local', item: LOCAL_MODELS[0], idx: 0 });
    else if (mode === 'api') setSel({ kind: 'api', item: API_PROVIDERS[0], idx: 0 });
    else setSel({ kind: 'my', item: MY_MODELS[0], idx: 0 });
  }, []);

  const onSelect = (s) => {setSel(s);setTab('overview');};
  const toggleStar = (key) => setStarred((prev) => {const n = new Set(prev);n.has(key) ? n.delete(key) : n.add(key);return n;});

  return <>
    <AppShell
      section="models" subPage={cfg.subPage}
      resizeKey="models"
      breadcrumb={[{ label: 'Bodhi', href: 'Bodhi Chat.html' }, { label: 'Models', href: 'Bodhi Models.html' }, { label: cfg.label, current: true }]}
      sidebar={<ModelsSidebar mode={mode} />}
      contentClass="flush" mainScroll={false} railScroll={false}
      rail={sel ? <DetailBody sel={sel} tab={tab} setTab={setTab} starred={starred} toggleStar={toggleStar} /> : null}
      railHeader={sel ? <DetailHeader sel={sel} onDeselect={() => setSel(null)} /> : undefined}>
      
      <ModelsMain mode={mode} sel={sel} onSelect={onSelect} density={density} showTags={showTags} showScore={showScore} />
    </AppShell>
  </>;
}

ReactDOM.createRoot(document.getElementById('root')).render(<ModelsApp />);