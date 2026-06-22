/* ═══════════════════════════════════════════════════════════════
   Bodhi Models — list rows
   models/models-rows.jsx   (load after models-base.jsx)

   The three row renderers for the main list: MyRow (My Models — local
   files, aliases, fallbacks, API models), LocalRow (HuggingFace repo
   results), and ApiRow (API provider results). Badges/verified marks
   used by the local rows come from bodhi-models-local.jsx.

   Exports: MyRow, LocalRow, ApiRow
═══════════════════════════════════════════════════════════════ */
const { STATUS_CFG, PROV_COLORS, FAMILY_SLUG } = window.MODELS_DATA;

/* catalog formatters (mirror models.dev / fmtPrice) */
const fmtCtx = (n) => !n ? '—' : n >= 1000000 ? (n % 1000000 === 0 ? n / 1000000 + 'M' : (n / 1000000).toFixed(1) + 'M') : Math.round(n / 1000) + 'K';
const catCaps = (m) => {
  const c = [];
  if (m.reasoning) c.push('reasoning');
  if (m.tool_call) c.push('tool-use');
  if (m.modalities && m.modalities.input && m.modalities.input.includes('image')) c.push('vision');
  if (m.structured_output) c.push('structured');
  return c;
};

/* Provider logo — real brand glyph from the Simple Icons CDN, tinted to the
   provider's brand color on a soft tile. Only slugs actually published on the
   CDN are listed; the rest (e.g. OpenAI / Groq / Together, removed for
   trademark) render a tasteful 2-letter brand monogram instead. */
const PROV_ICON_SLUG = { anthropic:'anthropic', openrouter:'openrouter', 'nvidia-nim':'nvidia' };
function ProviderLogo({ slug, provider, size = 36, radius = 9 }) {
  const [err, setErr] = React.useState(false);
  const color = PROV_COLORS[slug] || '#888';
  const si = PROV_ICON_SLUG[slug];
  const glyph = Math.round(size * 0.52);
  return (
    <div className="prov-avatar" style={{ width: size, height: size, borderRadius: radius, background: color + '1a', border: '1.5px solid ' + color + '40', flex: 'none' }}>
      {si && !err ?
        <img src={'https://cdn.simpleicons.org/' + si + '/' + color.replace('#', '')} width={glyph} height={glyph} alt={provider + ' logo'} onError={() => setErr(true)} style={{ display: 'block' }} /> :
        <span style={{ color, fontSize: Math.round(size * 0.32), fontWeight: 700 }}>{provider.slice(0, 2).toUpperCase()}</span>}
    </div>);
}

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
          {m.trending >= 70 && <span className="m-flame" title={'Trending · score ' + m.trending}><Ic name="flame" size={12} /></span>}
          {m.task === 'image-text-to-text' && <span className="m-modality" title="Image-Text-to-Text (multimodal)"><Ic name="image" size={10} />multimodal</span>}
          {m.staff_pick && <StaffBadge small />}
        </div>
        <div className="m-tags">
          {m.tags.map((t) => <Tag key={t} t={t} />)}
          {m.arch && <span className="m-arch-chip" title={'Architecture: ' + m.arch}>{m.arch}</span>}
        </div>
      </div>
      <div className="m-stats">
        {cols.downloads && stat('downloads', m.dlLabel, 'DOWNLOADS')}
        {cols.likes && stat('likes', m.likeLabel, 'LIKES')}
      </div>
      <div className="m-row-actions" />
    </div>);

}

function ApiRow({ p, active, onClick }) {
  const suffix = p.models >= 100 ? '+' : '';
  return (
    <div className={'m-row' + (active ? ' active' : '')} onClick={onClick}>
      <RowLink onActivate={onClick} label={'Open ' + p.provider} />
      <div className="m-num">#{p.rank}</div>
      <ProviderLogo slug={p.slug} provider={p.provider} />
      <div className="m-body">
        <div className="m-name" style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          {p.provider}
          {p.connected && <span className="status-badge status-connected"><Ic name="check-circle" size={9} />Connected</span>}
        </div>
        <div className="m-meta">{p.models}{suffix} models · {p.format}</div>
        <div className="m-tags">{p.tags.map((t) => <Tag key={t} t={t} />)}</div>
      </div>
      <div className="m-right">
        <div className="m-score" style={{ minWidth: 44 }}>
          <div className="m-score-num" style={{ fontSize: 16 }}>{p.models}{suffix}</div>
          <div className="m-score-lbl">MODELS</div>
        </div>
      </div>
    </div>);

}

/* Model monogram tile — family-tinted, mirrors ProviderLogo styling but keyed
   off the model family (logos use brand hexes, not UI tokens). */
function ModelLogo({ family, name, size = 36, radius = 9 }) {
  const color = PROV_COLORS[FAMILY_SLUG[family]] || '#888';
  const mono = String(family || name || '?').replace(/[^A-Za-z0-9]/g, '').slice(0, 2).toUpperCase();
  return (
    <div className="prov-avatar" style={{ width: size, height: size, borderRadius: radius, background: color + '1a', border: '1.5px solid ' + color + '40', flex: 'none' }}>
      <span style={{ color, fontSize: Math.round(size * 0.34), fontWeight: 700 }}>{mono}</span>
    </div>);
}

function ApiModelRow({ m, idx, active, onClick, sortKey }) {
  const free = (m.cost.input || 0) === 0 && (m.cost.output || 0) === 0;
  const caps = catCaps(m);
  return (
    <div className={'m-row m-row-cat' + (active ? ' active' : '')} onClick={onClick}>
      <RowLink onActivate={onClick} label={'Open ' + m.name} />
      <div className="cat-num">#{idx}</div>
      <div className="cat-model">
        <div className="cat-name">{m.name}</div>
        {m.family && <div className="cat-fam">{m.family}</div>}
      </div>
      <div className={'cat-cell cat-ctx' + (sortKey === 'context' ? ' sorted' : '')}>{fmtCtx(m.limit.context)}</div>
      <div className={'cat-cell cat-price' + (sortKey === 'input' ? ' sorted' : '')}>{free ? <span className="cat-free">Free</span> : <>${m.cost.input}<span className="cat-unit">/M</span></>}</div>
      <div className={'cat-cell cat-price' + (sortKey === 'output' ? ' sorted' : '')}>{free ? <span className="cat-free">Free</span> : <>${m.cost.output}<span className="cat-unit">/M</span></>}</div>
      <div className="cat-caps">{caps.length ? caps.map((c) => <Tag key={c} t={c} />) : <span className="cat-caps-none">—</span>}</div>
      <div className={'cat-prov' + (sortKey === 'providers' ? ' sorted' : '')}>
        <div className="cat-prov-num">{m.providers.length}</div>
      </div>
    </div>);
}

Object.assign(window, { MyRow, LocalRow, ApiRow, ApiModelRow, ProviderLogo, ModelLogo, fmtCtx, catCaps });
