/* ═══════════════════════════════════════════════════════════════
   BODHI MCP — PLAYGROUND VIEWS
   mcp/pg-views.jsx   (load AFTER pg-render.jsx, BEFORE the app root)

   The five capability surfaces the Playground shows once an instance
   is chosen: Overview (a friendly dashboard) + Tools / Prompts /
   Resources / Templates (each a searchable master-detail you can run).
═══════════════════════════════════════════════════════════════ */

/* ── shared run simulator ────────────────────────────────────── */
function useRunner() {
  const [run, setRun] = useState(null);
  const exec = ({ request, kind, build, meta }) => {
    setRun({ phase: 'running' });
    setTimeout(() => {
      const data = build();
      setRun({
        phase: 'done', kind, data,
        raw: JSON.stringify(data, null, 2), request,
        meta: meta || ('200 OK · ' + (170 + Math.floor(Math.random() * 250)) + 'ms'),
        token: Date.now(),
      });
    }, 460 + Math.random() * 420);
  };
  return [run, exec, setRun];
}
function inferKind(parsed) {
  if (Array.isArray(parsed) && parsed.every(b => b && b.type === 'text')) return 'text';
  if (parsed && parsed.type === 'text') return 'text';
  return 'data';
}
function chatHref(inst, kind, name) {
  return 'Bodhi Chat.html?mcp=' + encodeURIComponent(inst ? inst.instId : '') + '&' + kind + '=' + encodeURIComponent(name);
}

/* ── detail header (icon · name · desc · actions) ────────────── */
function DetailHead({ icon, name, mono, desc, tag, actions }) {
  return (
    <div className="pg-dh">
      <div className="pg-dh-ico"><Ic name={icon} size={18} /></div>
      <div className="pg-dh-text">
        <div className="pg-dh-name-row">
          <span className={'pg-dh-name' + (mono ? ' mono' : '')}>{name}</span>
          {tag && <span className="pg-dh-tag">{tag}</span>}
        </div>
        {desc && <div className="pg-dh-desc">{desc}</div>}
      </div>
      {actions && <div className="pg-dh-actions">{actions}</div>}
    </div>
  );
}

/* ── generic master-detail frame ─────────────────────────────── */
function MasterDetail({ items, getId, renderRow, renderDetail, searchKeys, searchPlaceholder, empty }) {
  const [q, setQ] = useState('');
  const [activeId, setActiveId] = useState(() => (items[0] ? getId(items[0]) : null));
  useEffect(() => { setActiveId(items[0] ? getId(items[0]) : null); setQ(''); }, [items]);

  if (!items || items.length === 0) {
    return (
      <div className="pg-cap-empty">
        <Ic name={empty.icon} size={34} />
        <div className="pg-cap-empty-t">{empty.title}</div>
        <div className="pg-cap-empty-s">{empty.desc}</div>
      </div>
    );
  }
  const ql = q.toLowerCase();
  const filtered = ql ? items.filter(it => searchKeys.some(k => String(it[k] || '').toLowerCase().includes(ql))) : items;
  const active = items.find(it => getId(it) === activeId) || null;

  return (
    <div className="pg-md">
      <div className="pg-md-list">
        <div className="pg-md-search">
          <ShellSearch size="sm" value={q} onChange={setQ} placeholder={searchPlaceholder} />
        </div>
        <div className="pg-md-rows">
          {filtered.length === 0 && <div className="pg-md-none">No matches</div>}
          {filtered.map(it => (
            <button key={getId(it)} type="button"
              className={'pg-row' + (getId(it) === activeId ? ' on' : '')}
              onClick={() => setActiveId(getId(it))}>
              {renderRow(it)}
            </button>
          ))}
        </div>
      </div>
      <div className="pg-md-detail">
        {active ? renderDetail(active) : (
          <div className="pg-pick"><Ic name="hand-pointer" size={26} /><div>Pick something on the left to begin.</div></div>
        )}
      </div>
    </div>
  );
}

/* ════════════════════ TOOLS ════════════════════ */
function ToolDetail({ tool, inst }) {
  const { dev } = useDev();
  const [values, setValues] = useState({});
  const [errors, setErrors] = useState({});
  const [jsonMode, setJsonMode] = useState(false);
  const [jsonText, setJsonText] = useState('');
  const [run, exec, setRun] = useRunner();

  useEffect(() => { setValues({}); setErrors({}); setJsonMode(false); setJsonText(''); setRun(null); }, [tool.name]);
  useEffect(() => { if (!dev) setJsonMode(false); }, [dev]);

  const args = tool.params.map(p => ({ name: p.name, label: prettyKey(p.name), required: p.required, desc: p.desc, placeholder: p.placeholder, type: p.type }));

  const runIt = () => {
    let vals = values;
    if (jsonMode) {
      try { vals = JSON.parse(jsonText || '{}'); }
      catch (e) { setRun({ phase: 'error', error: 'That isn’t valid JSON — ' + e.message, token: Date.now() }); return; }
    } else {
      const errs = {};
      tool.params.forEach(p => { if (p.required && !String(values[p.name] || '').trim()) errs[p.name] = true; });
      if (Object.keys(errs).length) { setErrors(errs); setTimeout(() => setErrors({}), 2200); return; }
    }
    const argsOut = {};
    tool.params.forEach(p => { const v = vals[p.name]; if (v != null && String(v).trim() !== '') argsOut[p.name] = p.type === 'number' ? Number(v) : v; });
    const request = { method: 'tools/call', params: { name: tool.name, arguments: argsOut } };
    let parsed; try { parsed = JSON.parse(tool.mockResponse); } catch (e) { parsed = tool.mockResponse; }
    exec({ request, kind: inferKind(parsed), build: () => parsed });
  };

  const jsonPlaceholder = (() => {
    const o = {}; tool.params.forEach(p => { if (p.required) o[p.name] = p.type === 'number' ? 0 : ''; });
    return JSON.stringify(o, null, 2);
  })();

  return (
    <div className="pg-detail">
      <DetailHead icon="wrench" name={tool.name} mono desc={tool.desc} tag="Tool"
        actions={<a className="pg-ghost-btn" href={chatHref(inst, 'tool', tool.name)}><Ic name="message-circle" size={13} /> Use in Chat</a>} />
      <div className="pg-detail-scroll">
        <div className="pg-run-card">
          <div className="pg-run-head">
            <span className="pg-run-title">{args.length ? 'Inputs' : 'Run'}</span>
            {dev && args.length > 0 && (
              <button className={'pg-mini-toggle' + (jsonMode ? ' on' : '')} onClick={() => setJsonMode(m => !m)}>
                <Ic name="braces" size={12} /> JSON
              </button>
            )}
          </div>
          {jsonMode
            ? <textarea className="pg-json" placeholder={jsonPlaceholder} value={jsonText} onChange={e => setJsonText(e.target.value)} />
            : <ArgForm args={args} values={values} onChange={(n, v) => setValues(p => ({ ...p, [n]: v }))} errors={errors} />}
          <div className="pg-run-row">
            <button className="pg-run-btn" onClick={runIt}><Ic name="play" size={14} /> Run tool</button>
            {(args.length > 0) && <button className="pg-clear-btn" onClick={() => { setValues({}); setJsonText(''); setRun(null); }}>Reset</button>}
          </div>
        </div>
        <ResultPanel run={run} title="Result" emptyHint="Run this tool to see what comes back" />
      </div>
    </div>
  );
}
function ToolsView({ serverId, inst }) {
  const tools = toolsFor(serverId);
  return (
    <MasterDetail items={tools} getId={t => t.name} searchKeys={['name', 'desc']}
      searchPlaceholder="Search tools…"
      empty={{ icon: 'wrench', title: 'No tools', desc: 'This MCP doesn’t expose any tools.' }}
      renderRow={t => (<><span className="pg-row-name mono">{t.name}</span><span className="pg-row-sub">{t.desc}</span></>)}
      renderDetail={t => <ToolDetail key={t.name} tool={t} inst={inst} />} />
  );
}

/* ════════════════════ PROMPTS ════════════════════ */
function PromptDetail({ prompt, inst }) {
  const [values, setValues] = useState({});
  const [errors, setErrors] = useState({});
  const [run, exec, setRun] = useRunner();
  useEffect(() => { setValues({}); setErrors({}); setRun(null); }, [prompt.name]);

  const previewIt = () => {
    const errs = {};
    prompt.args.forEach(a => { if (a.required && !String(values[a.name] || '').trim()) errs[a.name] = true; });
    if (Object.keys(errs).length) { setErrors(errs); setTimeout(() => setErrors({}), 2200); return; }
    const messages = prompt.build(values);
    const request = { method: 'prompts/get', params: { name: prompt.name, arguments: values } };
    exec({ request, kind: 'messages', build: () => messages });
  };

  return (
    <div className="pg-detail">
      <DetailHead icon={prompt.icon || 'message-square-quote'} name={prompt.title} desc={prompt.desc} tag="Prompt"
        actions={<a className="pg-ghost-btn" href={chatHref(inst, 'prompt', prompt.name)}><Ic name="message-circle" size={13} /> Use in Chat</a>} />
      <div className="pg-detail-scroll">
        <div className="pg-run-card">
          <div className="pg-run-head"><span className="pg-run-title">Fill in the blanks</span><span className="pg-run-name mono">{prompt.name}</span></div>
          <ArgForm args={prompt.args} values={values} onChange={(n, v) => setValues(p => ({ ...p, [n]: v }))} errors={errors} />
          <div className="pg-run-row">
            <button className="pg-run-btn" onClick={previewIt}><Ic name="eye" size={14} /> Preview messages</button>
            <a className="pg-clear-btn pg-clear-link" href={chatHref(inst, 'prompt', prompt.name)}><Ic name="arrow-right" size={13} /> Send to Chat</a>
          </div>
        </div>
        <ResultPanel run={run} title="Messages" emptyHint="Preview the messages this prompt produces" />
      </div>
    </div>
  );
}
function PromptsView({ serverId, inst }) {
  const prompts = promptsFor(serverId);
  return (
    <MasterDetail items={prompts} getId={p => p.name} searchKeys={['title', 'desc', 'name']}
      searchPlaceholder="Search prompts…"
      empty={{ icon: 'message-square-quote', title: 'No prompts', desc: 'This MCP doesn’t publish any ready-made prompts.' }}
      renderRow={p => (<><span className="pg-row-name"><Ic name={p.icon || 'message-square-quote'} size={13} /> {p.title}</span><span className="pg-row-sub">{p.desc}</span></>)}
      renderDetail={p => <PromptDetail key={p.name} prompt={p} inst={inst} />} />
  );
}

/* ════════════════════ RESOURCES ════════════════════ */
function ResourceDetail({ resource }) {
  const [run, exec, setRun] = useRunner();
  useEffect(() => { setRun(null); }, [resource.uri]);
  const readIt = () => {
    const request = { method: 'resources/read', params: { uri: resource.uri } };
    exec({ request, kind: 'resource', build: () => ({ uri: resource.uri, mimeType: resource.mimeType, contents: resource.contents }) });
  };
  return (
    <div className="pg-detail">
      <DetailHead icon={resource.icon || 'file-text'} name={resource.name} desc={resource.desc} tag="Resource" />
      <div className="pg-detail-scroll">
        <div className="pg-run-card">
          <div className="pg-meta-grid">
            <div className="pg-meta-row"><span className="pg-meta-k">Address</span><span className="pg-meta-v mono">{resource.uri}</span></div>
            <div className="pg-meta-row"><span className="pg-meta-k">Type</span><span className="pg-meta-v">{resource.mimeType}</span></div>
          </div>
          <div className="pg-run-row">
            <button className="pg-run-btn" onClick={readIt}><Ic name="book-open-text" size={14} /> Read resource</button>
          </div>
        </div>
        <ResultPanel run={run} title="Contents" emptyHint="Read this resource to see its contents" />
      </div>
    </div>
  );
}
function ResourcesView({ serverId }) {
  const resources = resourcesFor(serverId);
  return (
    <MasterDetail items={resources} getId={r => r.uri} searchKeys={['name', 'desc', 'uri']}
      searchPlaceholder="Search resources…"
      empty={{ icon: 'folder-open', title: 'No resources', desc: 'This MCP doesn’t expose any resources to read.' }}
      renderRow={r => (<><span className="pg-row-name"><Ic name={r.icon || 'file-text'} size={13} /> {r.name}</span><span className="pg-row-sub mono">{r.uri}</span></>)}
      renderDetail={r => <ResourceDetail key={r.uri} resource={r} />} />
  );
}

/* ════════════════════ TEMPLATES ════════════════════ */
function fillTemplate(tpl, values) {
  return tpl.replace(/\{([^}]+)\}/g, (m, k) => (values[k] ? values[k] : '{' + k + '}'));
}
function TemplateDetail({ template }) {
  const [values, setValues] = useState({});
  const [errors, setErrors] = useState({});
  const [run, exec, setRun] = useRunner();
  useEffect(() => { setValues({}); setErrors({}); setRun(null); }, [template.uriTemplate]);

  const resolved = fillTemplate(template.uriTemplate, values);
  const ready = template.args.every(a => !a.required || String(values[a.name] || '').trim());

  const resolveIt = () => {
    const errs = {};
    template.args.forEach(a => { if (a.required && !String(values[a.name] || '').trim()) errs[a.name] = true; });
    if (Object.keys(errs).length) { setErrors(errs); setTimeout(() => setErrors({}), 2200); return; }
    const res = template.resolve(values);
    const request = { method: 'resources/read', params: { uri: res.uri } };
    exec({ request, kind: 'resource', build: () => res });
  };

  return (
    <div className="pg-detail">
      <DetailHead icon={template.icon || 'layout-template'} name={template.name} desc={template.desc} tag="Template" />
      <div className="pg-detail-scroll">
        <div className="pg-run-card">
          <div className="pg-tpl-preview">
            <span className="pg-meta-k">Resolves to</span>
            <span className={'pg-tpl-uri mono' + (ready ? ' ready' : '')}>{resolved}</span>
          </div>
          <ArgForm args={template.args} values={values} onChange={(n, v) => setValues(p => ({ ...p, [n]: v }))} errors={errors} />
          <div className="pg-run-row">
            <button className="pg-run-btn" onClick={resolveIt}><Ic name="book-open-text" size={14} /> Resolve & read</button>
            {template.args.length > 0 && <button className="pg-clear-btn" onClick={() => { setValues({}); setRun(null); }}>Reset</button>}
          </div>
        </div>
        <ResultPanel run={run} title="Contents" emptyHint="Fill in the blanks, then resolve to read" />
      </div>
    </div>
  );
}
function TemplatesView({ serverId }) {
  const templates = templatesFor(serverId);
  return (
    <MasterDetail items={templates} getId={t => t.uriTemplate} searchKeys={['name', 'desc', 'uriTemplate']}
      searchPlaceholder="Search templates…"
      empty={{ icon: 'layout-template', title: 'No templates', desc: 'This MCP doesn’t expose any resource templates.' }}
      renderRow={t => (<><span className="pg-row-name"><Ic name={t.icon || 'layout-template'} size={13} /> {t.name}</span><span className="pg-row-sub mono">{t.uriTemplate}</span></>)}
      renderDetail={t => <TemplateDetail key={t.uriTemplate} template={t} />} />
  );
}

/* ════════════════════ OVERVIEW ════════════════════ */
function OverviewView({ inst, counts, onSelect, status }) {
  const s = inst.server;
  const authLabel = (AUTH_META[inst.authType] || AUTH_META.none).label;
  const cards = [
    { id: 'tools', icon: 'wrench', label: 'Tools', n: counts.tools, blurb: 'Actions you can run' },
    { id: 'prompts', icon: 'message-square-quote', label: 'Prompts', n: counts.prompts, blurb: 'Ready-made requests' },
    { id: 'resources', icon: 'folder-open', label: 'Resources', n: counts.resources, blurb: 'Data you can read' },
    { id: 'templates', icon: 'layout-template', label: 'Templates', n: counts.templates, blurb: 'Fill-in-the-blank reads' },
  ];
  return (
    <div className="pg-overview">
      <div className="pg-ov-hero">
        <ServerGlyph s={s} size={56} radius={14} />
        <div className="pg-ov-hero-text">
          <div className="pg-ov-title-row">
            <h1 className="pg-ov-title">{inst.instName}</h1>
            <span className={'pg-pill ' + (status === 'connected' ? 'ok' : 'warn')}>
              {status === 'connected' ? <><Ic name="circle-check" size={11} /> Connected</> : <><PgSpinner size={11} /> Connecting…</>}
            </span>
          </div>
          <div className="pg-ov-sub">{s.name} · {s.publisher}</div>
          <p className="pg-ov-desc">{s.desc}</p>
        </div>
      </div>

      <div className="pg-ov-facts">
        <div className="pg-fact"><span className="pg-fact-k"><Ic name="link" size={12} /> Endpoint</span><span className="pg-fact-v mono">{s.url}</span></div>
        <div className="pg-fact"><span className="pg-fact-k"><Ic name="radio" size={12} /> Transport</span><span className="pg-fact-v">{TRANSPORT_LABEL[s.transport] || s.transport}</span></div>
        <div className="pg-fact"><span className="pg-fact-k"><Ic name="shield" size={12} /> Authentication</span><span className="pg-fact-v">{authLabel}</span></div>
      </div>

      <div className="pg-ov-section-label">What you can do here</div>
      <div className="pg-ov-cards">
        {cards.map(c => (
          <button key={c.id} className="pg-ov-card" onClick={() => onSelect(c.id)} disabled={c.n === 0}>
            <div className="pg-ov-card-top"><span className="pg-ov-card-ico"><Ic name={c.icon} size={16} /></span><span className="pg-ov-card-n">{c.n}</span></div>
            <div className="pg-ov-card-label">{c.label}</div>
            <div className="pg-ov-card-blurb">{c.blurb}</div>
          </button>
        ))}
      </div>
    </div>
  );
}

Object.assign(window, {
  useRunner, inferKind, chatHref, DetailHead, MasterDetail,
  ToolsView, PromptsView, ResourcesView, TemplatesView, OverviewView,
});
