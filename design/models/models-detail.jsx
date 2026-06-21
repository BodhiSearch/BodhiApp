/* ═══════════════════════════════════════════════════════════════
   Bodhi Models — detail rail
   models/models-detail.jsx   (load after models-base.jsx)

   The right-rail detail panel: DetailHeader (badge + title + close) and
   DetailBody (per-kind body for My Models items, local HF repos, and
   API providers). Local-repo rendering leans on helpers from
   bodhi-models-local.jsx (MetaChips, RunBadge, MoreFrom, MarkdownView,
   FitPill, quantFit, parseGB, LM_HOST, relTime, fmtAbs, badges).

   Exports: DetailHeader, SpecTable, DetailBody
═══════════════════════════════════════════════════════════════ */
const { STATUS_CFG: DETAIL_STATUS_CFG, PROV_COLORS: DETAIL_PROV_COLORS } = window.MODELS_DATA;

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
    const color = DETAIL_PROV_COLORS[item.slug] || '#888';
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
            <span className="mb-stat" title={'Updated ' + fmtAbs(item.updated)}><Ic name="calendar" size={11} />Updated {relTime(item.updated)}</span>
            {item.trending >= 70 && <span className="mb-stat mb-flame" title={'Trending score ' + item.trending}><Ic name="flame" size={11} />{item.trending}</span>}
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
          <div className="p-section"><div className="p-sec-lbl">Specs</div><SpecTable rows={[...d.specs, { k: 'Created', v: fmtAbs(item.created) }, { k: 'Updated', v: fmtAbs(item.updated) }]} /></div>
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
  const d = item.detail,sc = DETAIL_STATUS_CFG[item.status] || DETAIL_STATUS_CFG.available;
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

Object.assign(window, { DetailHeader, SpecTable, DetailBody });
