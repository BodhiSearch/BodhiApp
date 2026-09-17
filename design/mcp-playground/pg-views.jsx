/* ═══════════════════════════════════════════════════════════════
   BODHI MCP — PLAYGROUND DETAILS + CAPABILITY CONFIG
   mcp-playground/pg-views.jsx   (load AFTER pg-render.jsx, BEFORE pg-chrome)

   The centre-panel DETAIL surfaces — one per capability (Tools / Prompts /
   Resources / Templates) plus the Overview dashboard — and CAP_CONFIG,
   which describes each capability's list + detail so pg-chrome can render
   the right-rail list and the centre detail generically.

   The master-detail is now SPLIT across the shell: the list lives in the
   pinned right rail (pg-chrome) and the detail here in the centre.
═══════════════════════════════════════════════════════════════ */

/* ── shared run simulator ────────────────────────────────────── */
function useRunner() {
  const [run, setRun] = useState(null);
  const exec = ({ request, kind, build, meta, raw }) => {
    setRun({ phase: 'running' });
    setTimeout(() => {
      const data = build();
      const rawStr = raw !== undefined
        ? (typeof raw === 'string' ? raw : JSON.stringify(raw, null, 2))
        : JSON.stringify(data, null, 2);
      setRun({
        phase: 'done', kind, data, raw: rawStr, request,
        meta: meta || ('200 OK · ' + (170 + Math.floor(Math.random() * 250)) + 'ms'),
        token: Date.now(),
      });
    }, 460 + Math.random() * 420);
  };
  return [run, exec, setRun];
}
function chatHref(inst, kind, name) {
  return 'Chat.html?mcp=' + encodeURIComponent(inst ? inst.instId : '') + '&' + kind + '=' + encodeURIComponent(name);
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

/* the centre "pick something" placeholder when nothing is selected */
function PickSomething({ what }) {
  return <div className="pg-pick"><Ic name="mouse-pointer-click" size={26} /><div>Pick {what} on the right to begin.</div></div>;
}

/* ════════════════════ TOOLS ════════════════════ */
function ToolRow({ tool }) {
  // Friendly title leads; the protocol name rides along in brackets. The
  // primary behaviour hint (read-only / makes changes …) shows as a dot,
  // always — hover it for the full explanation.
  const hasTitle = !!tool.title;
  const primary = hasTitle ? tool.title : tool.name;
  const hint = hintsForTool(tool)[0];
  return (
    <>
      <span className="pg-row-name">
        <span className={'pg-row-text' + (hasTitle ? '' : ' mono')}>
          {primary}{hasTitle && <span className="pg-row-code"> ({tool.name})</span>}
        </span>
        {hint && <span className={'pg-row-dot shell-tip tone-' + hint.tone} data-tip={hint.label + ' \u2014 ' + hint.tip} />}
      </span>
      <span className="pg-row-sub">{tool.desc}</span>
    </>
  );
}

function ToolHeader({ tool, inst }) {
  const hasTitle = !!tool.title;
  return (
    <div className="pg-dh pg-dh-tool">
      <div className="pg-dh-ico"><Ic name="wrench" size={18} /></div>
      <div className="pg-dh-text">
        <div className="pg-dh-name-row">
          <span className="pg-dh-name">{hasTitle ? tool.title : tool.name}</span>
          {hasTitle && <span className="pg-dh-codename mono">({tool.name})</span>}
          <span className="pg-dh-tag">Tool</span>
        </div>
        {tool.desc && <div className="pg-dh-desc">{tool.desc}</div>}
        <BehaviourHints tool={tool} />
      </div>
    </div>
  );
}

function ToolDetail({ tool, inst }) {
  const [values, setValues] = useState({});
  const [errors, setErrors] = useState({});
  const [run, exec, setRun] = useRunner();
  const bump = useLiveBump();                      // re-read the cross-page store on change

  useEffect(() => { setValues({}); setErrors({}); setRun(null); }, [tool.name]);

  const args = tool.params.map(p => ({ name: p.name, label: prettyKey(p.name), required: p.required, desc: p.desc, placeholder: p.placeholder, type: p.type }));
  const isInteractive = !!tool.interaction;

  /* a paused / resumed run for an interactive tool, from the store. The
     ?resume= run wins (auto-return targets it); else the latest for this tool. */
  let storeRun = null;
  if (isInteractive) {
    const resumeId = urlParams().get('resume');
    const r = resumeId ? liveGetRun(resumeId) : null;
    storeRun = (r && r.toolName === tool.name) ? r : liveLatestRunForTool(inst.instId, tool.name);
  }

  const runIt = () => {
    const errs = {};
    tool.params.forEach(p => { if (p.required && !String(values[p.name] || '').trim()) errs[p.name] = true; });
    if (Object.keys(errs).length) { setErrors(errs); setTimeout(() => setErrors({}), 2200); return; }

    if (isInteractive) {
      // start the call; the server "sends back" a request → auto-switch to its page
      const runId = uid('run');
      let requestId;
      if (tool.interaction === 'elicitation') {
        requestId = liveAddElicitation({ instId: inst.instId, instName: inst.instName, serverId: inst.serverId,
          fromToolName: tool.name, fromToolTitle: tool.title, runId, message: tool.elicitMessage, requestedSchema: tool.elicitSchema });
      } else {
        const params = samplingParamsWithNotes(tool.samplingParams, values.notes);
        requestId = liveAddSampling({ instId: inst.instId, instName: inst.instName, serverId: inst.serverId,
          fromToolName: tool.name, fromToolTitle: tool.title, runId, title: truncate(msgText(params.messages[0]), 60), params });
      }
      liveAddRun({ runId, instId: inst.instId, toolName: tool.name, interaction: tool.interaction, requestId });
      setRun({ phase: 'switching' });
      setTimeout(() => { window.location.href = window.capHref(tool.interaction, inst) + '&focus=' + encodeURIComponent(requestId); }, 850);
      return;
    }

    const argsOut = {};
    tool.params.forEach(p => { const v = values[p.name]; if (v != null && String(v).trim() !== '') argsOut[p.name] = p.type === 'number' ? Number(v) : v; });
    const request = { method: 'tools/call', params: { name: tool.name, arguments: argsOut } };
    const model = toolResultModel(tool);
    exec({ request, kind: 'tool', build: () => model, raw: toolResultEnvelope(model) });
  };

  /* what to show beneath the inputs */
  let resultArea;
  if (run && run.phase === 'switching') {
    resultArea = <WaitingPanel switching interaction={tool.interaction} />;
  } else if (!run && storeRun && storeRun.status === 'waiting') {
    resultArea = <WaitingPanel interaction={storeRun.interaction} requestId={storeRun.requestId} inst={inst} />;
  } else {
    const effective = run || (storeRun && storeRun.status === 'resolved' ? runToDisplay(storeRun) : null);
    resultArea = <ResultPanel run={effective} title="Result" emptyHint={isInteractive ? 'Run this tool to start the exchange' : 'Run this tool to see what comes back'} />;
  }

  return (
    <div className="pg-detail" data-screen-label={'Tool · ' + (tool.title || tool.name)}>
      <ToolHeader tool={tool} inst={inst} />
      <div className="pg-detail-scroll">
        <div className="pg-run-card">
          {args.length > 0 ? (
            <>
              <div className="pg-run-head"><span className="pg-run-title">Inputs</span></div>
              <ArgForm args={args} values={values} onChange={(n, v) => setValues(p => ({ ...p, [n]: v }))} errors={errors} />
            </>
          ) : (
            <div className="pg-noargs">{isInteractive
              ? (tool.interaction === 'sampling' ? 'No inputs needed — running this asks you to approve using the AI.' : 'No inputs needed — running this asks you for a few details.')
              : 'No inputs needed — just run it.'}</div>
          )}
          <div className="pg-run-row">
            <button className="pg-run-btn" onClick={runIt}><Ic name="play" size={14} /> Run tool</button>
            {(args.length > 0) && <button className="pg-clear-btn" onClick={() => { setValues({}); setRun(null); }}>Reset</button>}
          </div>
        </div>
        {resultArea}
      </div>
    </div>
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
function makeTransientResource(link) {
  const mime = link.mimeType || 'text/markdown';
  const isJson = /json/.test(mime);
  return {
    uri: link.uri, name: link.name || link.uri, icon: 'file-text', mimeType: mime, transient: true,
    desc: link.description || 'Opened from a tool result.',
    contents: isJson
      ? { uri: link.uri, openedFrom: 'tool result', note: 'Mock resource resolved from a tool link.' }
      : `# ${link.name || 'Resource'}\n\n${link.description || ''}\n\nThis resource was opened from a tool result. Its address is \`${link.uri}\`.`,
  };
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

/* ════════════════════ OVERVIEW ════════════════════ */
function OverviewView({ inst, counts, status, capHref }) {
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
        {cards.map(c => {
          const disabled = c.n === 0;
          return (
            <a key={c.id} className={'pg-ov-card' + (disabled ? ' disabled' : '')}
              href={disabled ? undefined : capHref(c.id)}>
              <div className="pg-ov-card-top"><span className="pg-ov-card-ico"><Ic name={c.icon} size={16} /></span><span className="pg-ov-card-n">{c.n}</span></div>
              <div className="pg-ov-card-label">{c.label}</div>
              <div className="pg-ov-card-blurb">{c.blurb}</div>
            </a>
          );
        })}
      </div>
    </div>
  );
}

/* ════════════════════ CAPABILITY CONFIG ════════════════════
   Drives the right-rail list + the centre detail for each capability. */
const CAP_CONFIG = {
  tools: {
    listTitle: 'Tools',
    getItems: sid => playgroundToolsFor(sid),
    getId: t => t.name,
    searchKeys: ['title', 'name', 'desc'],
    searchPlaceholder: 'Search tools…',
    renderRow: t => <ToolRow tool={t} />,
    renderDetail: (t, inst) => <ToolDetail key={t.name} tool={t} inst={inst} />,
    empty: { icon: 'wrench', title: 'No tools', desc: 'This MCP doesn’t expose any tools.' },
    pick: 'a tool',
  },
  prompts: {
    listTitle: 'Prompts',
    getItems: sid => promptsFor(sid),
    getId: p => p.name,
    searchKeys: ['title', 'desc', 'name'],
    searchPlaceholder: 'Search prompts…',
    renderRow: p => (<><span className="pg-row-name"><Ic name={p.icon || 'message-square-quote'} size={13} /> <span className="pg-row-text">{p.title}</span></span><span className="pg-row-sub">{p.desc}</span></>),
    renderDetail: (p, inst) => <PromptDetail key={p.name} prompt={p} inst={inst} />,
    empty: { icon: 'message-square-quote', title: 'No prompts', desc: 'This MCP doesn’t publish any ready-made prompts.' },
    pick: 'a prompt',
  },
  resources: {
    listTitle: 'Resources',
    getItems: sid => resourcesFor(sid),
    getId: r => r.uri,
    searchKeys: ['name', 'desc', 'uri'],
    searchPlaceholder: 'Search resources…',
    renderRow: r => (<><span className="pg-row-name"><Ic name={r.icon || 'file-text'} size={13} /> <span className="pg-row-text">{r.name}</span>{r.transient && <span className="pg-row-badge">from tool</span>}</span><span className="pg-row-sub mono">{r.uri}</span></>),
    renderDetail: (r) => <ResourceDetail key={r.uri} resource={r} />,
    empty: { icon: 'folder-open', title: 'No resources', desc: 'This MCP doesn’t expose any resources to read.' },
    pick: 'a resource',
  },
  templates: {
    listTitle: 'Templates',
    getItems: sid => templatesFor(sid),
    getId: t => t.uriTemplate,
    searchKeys: ['name', 'desc', 'uriTemplate'],
    searchPlaceholder: 'Search templates…',
    renderRow: t => (<><span className="pg-row-name"><Ic name={t.icon || 'layout-template'} size={13} /> <span className="pg-row-text">{t.name}</span></span><span className="pg-row-sub mono">{t.uriTemplate}</span></>),
    renderDetail: (t) => <TemplateDetail key={t.uriTemplate} template={t} />,
    empty: { icon: 'layout-template', title: 'No templates', desc: 'This MCP doesn’t expose any resource templates.' },
    pick: 'a template',
  },
};

/* Merge the live-interaction capabilities (Elicitation / Sampling) defined in
   pg-live.jsx into the same config the chrome renders generically. */
if (window.LIVE_CONFIG) Object.assign(CAP_CONFIG, window.LIVE_CONFIG);

Object.assign(window, {
  useRunner, chatHref, DetailHead, PickSomething,
  ToolRow, ToolHeader, ToolDetail, PromptDetail, ResourceDetail, TemplateDetail, OverviewView,
  makeTransientResource, fillTemplate, CAP_CONFIG,
});
