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
const { STATUS_CFG: DETAIL_STATUS_CFG, PROV_COLORS: DETAIL_PROV_COLORS, MY_MODELS: DETAIL_MY_MODELS } = window.MODELS_DATA;

/* Configured API models (from My Models) that expose a given catalog model id. */
function apiModelsForCatalog(id) {
  return (DETAIL_MY_MODELS || []).filter((m) => m.type === 'api-model' && m.detail && (m.detail.models || []).includes(id));
}

/* Catalog status → display label + token-based badge class (status absent = Stable) */
function catStatusBadge(s) {
  if (s === 'deprecated') return { lbl: 'Deprecated', cls: 'status-available' };
  if (s === 'beta') return { lbl: 'Beta', cls: 'status-apikey' };
  if (s === 'alpha') return { lbl: 'Alpha', cls: 'status-apikey' };
  return { lbl: 'Stable', cls: 'status-connected' };
}
const catMoney = (v) => v === 0 ? '$0' : '$' + v;

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
  } else if (kind === 'api') {
    badge = <ProviderLogo slug={item.slug} provider={item.provider} size={26} radius={7} />;
    title = item.provider;
  } else {
    /* api-catalog */
    badge = <ModelLogo family={item.family} name={item.name} size={26} radius={7} />;
    title = item.name;
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
          <a href="Models-New-Router.html" className="btn-add" style={{ background: 'var(--c-teal-text)', color: '#fff' }}><Ic name="pencil" size={14} /> Edit model router</a>
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

  /* API catalog (page A) — per-model spec rail */
  if (kind === 'api-catalog') {
    const m = item;
    const st = catStatusBadge(m.status);
    const caps = [];
    if (m.reasoning) caps.push('reasoning');
    if (m.tool_call) caps.push('tool-use');
    if (m.structured_output) caps.push('structured');
    if (m.attachment) caps.push('attachment');
    if (m.modalities.input.includes('image')) caps.push('vision');
    const pricing = [
      { k: 'Input', v: catMoney(m.cost.input) },
      { k: 'Output', v: catMoney(m.cost.output) }];
    if (m.cost.cache_read != null) pricing.push({ k: 'Cache read', v: catMoney(m.cost.cache_read) });
    if (m.cost.cache_write != null) pricing.push({ k: 'Cache write', v: catMoney(m.cost.cache_write) });
    const meta = [];
    if (m.family) meta.push({ k: 'Family', v: m.family });
    meta.push({ k: 'Open weights', v: m.open_weights ? 'Yes' : 'No' });
    meta.push({ k: 'Status', v: m.status || 'stable' });
    if (m.release_date) meta.push({ k: 'Released', v: m.release_date });
    if (m.last_updated) meta.push({ k: 'Updated', v: m.last_updated });
    if (m.knowledge) meta.push({ k: 'Knowledge cutoff', v: m.knowledge });
    if (m.temperature != null) meta.push({ k: 'Temperature', v: m.temperature ? 'Yes' : 'No' });
    const ro = m.reasoning_options ? Object.keys(m.reasoning_options).join(' · ') : null;
    const primary = m.providers[0];
    const usedBy = apiModelsForCatalog(m.id);
    return <>
      <div className="detail-metabar">
        <div className="metabar-row">
          <div className="cat-rail-fam">{m.family || m.id}</div>
          <div className="metabar-stats" style={{ gap: 6 }}>
            <span className={'status-badge ' + st.cls}><Ic name={m.status === 'deprecated' ? 'circle-slash' : 'activity'} size={9} />{st.lbl}</span>
            <span className={'ow-chip' + (m.open_weights ? ' open' : '')}><Ic name={m.open_weights ? 'unlock' : 'lock'} size={9} />{m.open_weights ? 'Open' : 'Closed'}</span>
          </div>
        </div>
      </div>
      <div className="panel-body">
        <div className="p-section">
          <div className="p-sec-lbl">Cost · $/Mtok</div>
          <SpecTable rows={pricing} />
        </div>
        <div className="p-section">
          <div className="p-sec-lbl">Limits</div>
          <SpecTable rows={[{ k: 'Context', v: fmtCtx(m.limit.context) }, { k: 'Max output', v: m.limit.output ? fmtCtx(m.limit.output) : '—' }]} />
        </div>
        <div className="p-section">
          <div className="p-sec-lbl">Modalities</div>
          <SpecTable rows={[{ k: 'Input', v: m.modalities.input.join(', ') }, { k: 'Output', v: m.modalities.output.join(', ') }]} />
        </div>
        <div className="p-section">
          <div className="p-sec-lbl">Capabilities</div>
          <div className="cap-chips">{caps.length ? caps.map((c) => <Tag key={c} t={c} big />) : <span className="prov-empty">No special capabilities listed.</span>}</div>
          {ro && <div className="cat-ro-note"><Ic name="brain" size={11} />Reasoning controls: <span className="mono">{ro}</span></div>}
        </div>
        <div className="p-section">
          <div className="p-sec-lbl">Meta</div>
          <SpecTable rows={meta} />
        </div>
        <div className="p-section">
          <div className="p-sec-lbl">Served by ({m.providers.length})</div>
          <div className="served-list">
            {m.providers.map((p) =>
            <a className="served-row" key={p.slug} href={'Models-Explore-API-Providers.html?select=' + p.slug} title={'Open ' + p.name + ' in API Providers'}>
                <ProviderLogo slug={p.slug} provider={p.name} size={26} radius={7} />
                <div className="served-meta">
                  <div className="served-name">{p.name}</div>
                  <div className="served-url">{p.base_url}</div>
                </div>
                <div className="served-price">${p.in}<span className="served-sep"> / </span>${p.out}</div>
              </a>
            )}
          </div>
        </div>
        <div className="p-section">
          <div className="p-sec-lbl">Configured in your API models{usedBy.length ? ' (' + usedBy.length + ')' : ''}</div>
          {usedBy.length ?
          <div className="prov-linklist">
            {usedBy.map((am) =>
            <a className="prov-linkrow" key={am.id} href={'Models-My-Models.html?select=' + am.id} title={'Open ' + am.name + ' in My Models'}>
                <span className="my-icon-box my-icon-api-model" style={{ width: 26, height: 26, borderRadius: 7, flex: 'none' }}><Ic name="at-sign" size={13} /></span>
                <span className="prov-linkname">{am.name}</span>
                {am.keyStatus === 'connected' ?
                <span className="my-key-ok"><Ic name="check-circle" size={10} />connected</span> :
                <span className="my-key-no"><Ic name="key" size={10} />no key</span>}
                <Ic name="chevron-right" size={14} />
              </a>
            )}
          </div> :
          <a className="prov-crosslink" href={'Models-New-API.html?provider=' + primary.slug + '&model=' + m.id}>
            <Ic name="plus" size={14} />
            <span className="prov-crosslink-txt">Add an API model for {m.name}</span>
            <Ic name="arrow-right" size={14} />
          </a>}
        </div>
      </div>
      <div className="panel-foot">
        <a className="btn-add" href={'Models-New-API.html?provider=' + primary.slug + '&model=' + m.id}><Ic name="plug-zap" size={14} /> Configure in Bodhi</a>
      </div>
    </>;
  }

  /* API provider */
  const suffix = item.models >= 100 ? '+' : '';
  const fmtPrice = (m) => m.in === 0 && m.out === 0 ? 'Free' : '$' + m.in + ' / $' + m.out;
  return <>
    <div className="panel-body">
      <div className="panel-lead" style={{ display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'wrap' }}>
        {item.connected ?
        <span className="status-badge status-connected"><Ic name="check-circle" size={9} />Connected</span> :
        <span className="status-badge status-available"><Ic name="circle" size={9} />Not connected</span>}
        <span className="prov-format-chip"><Ic name="plug" size={10} />{item.format}</span>
      </div>

      <div className="p-sec-lbl">Connection</div>
      <div className="prov-meta">
        {item.api &&
        <div className="prov-meta-row"><span className="prov-meta-k">Base URL</span><span className="prov-meta-v mono">{item.api}</span></div>}
        {item.env && item.env.length > 0 &&
        <div className="prov-meta-row"><span className="prov-meta-k">Env var</span><span className="prov-meta-envs">{item.env.map((e) => <code className="env-chip" key={e}>{e}</code>)}</span></div>}
        {item.npm &&
        <div className="prov-meta-row"><span className="prov-meta-k">SDK</span><span className="prov-meta-v mono">{item.npm}</span></div>}
        {item.doc &&
        <div className="prov-meta-row"><span className="prov-meta-k">Docs</span><a className="prov-doc-link" href={item.doc} target="_blank" rel="noopener"><Ic name="external-link" size={11} />Reference</a></div>}
      </div>

      <div className="p-sec-lbl">API models using this provider</div>
      {item.apiModels.length ?
      <div className="prov-linklist" style={{ marginBottom: 18 }}>
          {item.apiModels.map((m) =>
        <a className="prov-linkrow" key={m.id} href={'Models-My-Models.html?select=' + m.id}>
              <span className="my-icon-box my-icon-api-model" style={{ width: 26, height: 26, borderRadius: 7, flex: 'none' }}><Ic name="at-sign" size={13} /></span>
              <span className="prov-linkname">{m.name}</span>
              {m.keyStatus === 'connected' ?
          <span className="my-key-ok"><Ic name="check-circle" size={10} />connected</span> :
          <span className="my-key-no"><Ic name="key" size={10} />no key</span>}
              <Ic name="chevron-right" size={14} />
            </a>
        )}
        </div> :
      <div className="prov-empty" style={{ marginBottom: 18 }}>No API models created from this provider yet.</div>}

      <div className="p-sec-lbl prov-mt-head">
        <span>Models ({item.models}{suffix})</span>
        <span className="prov-mt-price-lbl">Price /M · in / out</span>
      </div>
      <div className="prov-mtable">
        {item.modelRows.map((m) =>
        <div className="prov-mrow" key={m.name}>
            <div className="prov-mrow-top">
              <span className="prov-mname">{m.name}</span>
              <span className={'prov-mprice' + (m.in === 0 && m.out === 0 ? ' free' : '')}>{fmtPrice(m)}</span>
            </div>
            <div className="prov-mrow-bot">
              <div className="prov-mcaps">{m.caps.map((c) => <Tag key={c} t={c} />)}</div>
              <span className="prov-mctx" title="Context window"><Ic name="align-left" size={10} />{m.ctx}</span>
            </div>
          </div>
        )}
      </div>

      <a className="prov-crosslink" href={'Models-Explore-API.html?provider=' + item.slug}>
        <Ic name="sparkles" size={14} />
        <span className="prov-crosslink-txt">View all models from {item.provider}</span>
        <Ic name="arrow-right" size={14} />
      </a>
    </div>
    <div className="panel-foot">
      {item.connected ?
      <button className="btn-add" style={{ background: 'hsl(var(--foreground))', color: 'hsl(var(--background))' }}><Ic name="settings-2" size={14} /> Manage Connection</button> :
      <button className="btn-add"><Ic name="plug-zap" size={14} /> Connect Provider</button>}
    </div>
  </>;
}

Object.assign(window, { DetailHeader, SpecTable, DetailBody });
