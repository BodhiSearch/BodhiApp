/* ═══════════════════════════════════════════════════════════════
   BODHI MCP — PLAYGROUND · LIVE INTERACTIONS (elicitation + sampling)
   mcp-playground/pg-live.jsx   (load AFTER pg-render.jsx, BEFORE pg-views.jsx)

   The three "alive" moments of a tool call. This file owns:
     • a small CROSS-PAGE store (localStorage + events) — because every
       capability is its own HTML page, the inbox of requests and the
       paused tool-runs must survive navigation. This is what powers the
       auto-switch-on-request / auto-return-on-resolve pattern.
     • the schema-driven AUTO-FORM builder (the elicitation form),
     • the Elicitation + Sampling request DETAIL surfaces,
     • the inbox row + status chip, the Tools "Waiting on you…" panel,
     • LIVE_CONFIG (merged into CAP_CONFIG by pg-views) so the existing
       list+detail chrome renders these two new pages generically.

   Faithful to MCP spec 2025-06-18 wire shapes (reachable via Raw ⌄).
═══════════════════════════════════════════════════════════════ */
const { useState, useEffect, useRef, useMemo } = React;

/* ════════════════════════════════════════════════════════════════
   1 · CROSS-PAGE STORE  (localStorage is the IPC layer between pages)
═══════════════════════════════════════════════════════════════════ */
const LIVE_NS = 'bodhi.pg.live.v2';
const LIVE_MODELS = ['llama-3.1-8b', 'qwen2.5-7b', 'gpt-4o-mini'];

function liveRead() {
  try { return JSON.parse(localStorage.getItem(LIVE_NS)) || null; } catch (e) { return null; }
}
function liveWrite(state) {
  try { localStorage.setItem(LIVE_NS, JSON.stringify(state)); } catch (e) {}
  window.dispatchEvent(new CustomEvent('pg-live'));
}
function liveState() { ensureSeed(); return liveRead() || { v: 2, elicit: [], sampling: [], runs: {} }; }

function uid(p) { return p + '-' + Date.now().toString(36) + Math.random().toString(36).slice(2, 6); }

/* ── reads ── */
function byNewest(a, b) { return (b.ts || 0) - (a.ts || 0); }
function liveElicitations(instId) {
  return liveState().elicit.filter(r => !instId || r.instId === instId).slice().sort(byNewest);
}
function liveSamplings(instId) {
  return liveState().sampling.filter(r => !instId || r.instId === instId).slice().sort(byNewest);
}
function liveList(cap, instId) { return cap === 'sampling' ? liveSamplings(instId) : liveElicitations(instId); }
function liveGetRun(runId) { return runId ? (liveState().runs[runId] || null) : null; }
function liveLatestRunForTool(instId, toolName) {
  const runs = Object.values(liveState().runs)
    .filter(r => r.instId === instId && r.toolName === toolName)
    .sort((a, b) => (b.ts || 0) - (a.ts || 0));
  return runs[0] || null;
}
function pendingCounts(instId) {
  return {
    elicitation: liveElicitations(instId).filter(r => r.status === 'waiting').length,
    sampling: liveSamplings(instId).filter(r => r.status === 'waiting').length,
  };
}

/* ── writes: create ── */
function liveAddElicitation(o) {
  const st = liveState();
  const id = uid('el');
  st.elicit.push({ id, kind: 'elicitation', status: 'waiting', ts: Date.now(),
    instId: o.instId, instName: o.instName, serverId: o.serverId,
    fromToolName: o.fromToolName, fromToolTitle: o.fromToolTitle, runId: o.runId,
    message: o.message, requestedSchema: o.requestedSchema });
  liveWrite(st);
  return id;
}
function liveAddSampling(o) {
  const st = liveState();
  const id = uid('sm');
  st.sampling.push({ id, kind: 'sampling', status: 'waiting', ts: Date.now(),
    instId: o.instId, instName: o.instName, serverId: o.serverId,
    fromToolName: o.fromToolName, fromToolTitle: o.fromToolTitle, runId: o.runId,
    title: o.title, params: o.params });
  liveWrite(st);
  return id;
}
function liveAddRun(o) {
  const st = liveState();
  st.runs[o.runId] = { runId: o.runId, instId: o.instId, toolName: o.toolName,
    interaction: o.interaction, requestId: o.requestId, status: 'waiting', ts: Date.now() };
  liveWrite(st);
}

/* ── writes: resolve (also finishes the paused tool-run) ── */
function rtMeta() { return '200 OK · ' + (180 + Math.floor(Math.random() * 260)) + 'ms'; }
function finishRun(st, runId, outcome, result) {
  const run = st.runs[runId];
  if (!run) return;
  run.status = 'resolved';
  run.outcome = outcome;
  run.result = result;
  run.meta = rtMeta();
  run.request = { method: 'tools/call', params: { name: run.toolName, arguments: {} } };
  run.raw = JSON.stringify(toolResultEnvelope(result), null, 2);
  run.resolvedTs = Date.now();
}
function liveResolveElicitation(id, action, content) {
  const st = liveState();
  const req = st.elicit.find(r => r.id === id);
  if (!req) return null;
  const map = { accept: 'provided', decline: 'declined', cancel: 'cancelled' };
  req.status = map[action] || 'cancelled';
  req.action = action;
  req.content = action === 'accept' ? (content || {}) : null;
  req.resolvedTs = Date.now();
  if (req.runId) finishRun(st, req.runId, req.status, buildElicitResult(action, content));
  liveWrite(st);
  return req.runId;
}
function liveResolveSampling(id, action, info) {
  const st = liveState();
  const req = st.sampling.find(r => r.id === id);
  if (!req) return null;
  req.status = action === 'approve' ? 'approved' : 'declined';
  req.action = action;
  if (action === 'approve') { req.model = info.model; req.answer = info.answer; }
  req.resolvedTs = Date.now();
  if (req.runId) finishRun(st, req.runId, req.status, buildSamplingResult(action, info));
  liveWrite(st);
  return req.runId;
}

/* ── the tool's final result, by outcome (shown back on Tools) ── */
function buildElicitResult(action, content) {
  if (action === 'accept') return {
    content: [{ type: 'text', text: '✅ Thanks — the server used the details you provided to finish the run.' }],
    structuredContent: content && Object.keys(content).length ? { received: content } : undefined,
  };
  if (action === 'decline') return {
    content: [{ type: 'text', text: 'You declined to share those details. The server was told, and the run continued without them.' }],
  };
  return { content: [{ type: 'text', text: 'Okay — stopped. You cancelled before the run finished, so nothing was sent.' }] };
}
function buildSamplingResult(action, info) {
  if (action === 'approve') return {
    content: [
      { type: 'text', text: 'Here’s the AI-assisted result the server returned:' },
      { type: 'text', format: 'markdown', text: info.answer },
    ],
    structuredContent: { model: info.model, stopReason: 'endTurn' },
  };
  return { content: [{ type: 'text', text: 'You declined to let the server use the AI for this step, so the run stopped here.' }] };
}
function runToDisplay(run) {
  if (!run || run.status !== 'resolved') return null;
  const ok = run.outcome !== 'error';
  return { phase: ok ? 'done' : 'error', kind: 'tool', data: run.result,
    raw: run.raw, request: run.request, meta: run.meta, token: run.resolvedTs };
}

/* ── last chat model (Bodhi persists it; we read it as the default) ── */
function lastChatModel() {
  try { const m = localStorage.getItem('bodhi.pg.lastModel'); if (m && LIVE_MODELS.includes(m)) return m; } catch (e) {}
  return LIVE_MODELS[0];
}
function writeLastModel(m) { try { localStorage.setItem('bodhi.pg.lastModel', m); } catch (e) {} }

/* ── inject typed notes into a sampling request ── */
function samplingParamsWithNotes(base, notes) {
  const p = JSON.parse(JSON.stringify(base));
  if (notes && p.messages && p.messages[0] && p.messages[0].content) {
    p.messages[0].content.text = 'Summarise the notes below into 3 crisp bullet points, then suggest one next step.\n\nNotes:\n' + notes;
  }
  return p;
}

/* ════════════════════════════════════════════════════════════════
   2 · SEED  (1 waiting + a couple resolved, per page — for the
   everything instance only, so other MCPs show clean empty states)
═══════════════════════════════════════════════════════════════════ */
function ensureSeed() {
  const cur = liveRead();
  if (cur && cur.v === 2) return;
  const now = Date.now();
  const M = 60 * 1000, H = 60 * M;
  const I = { instId: 'inst-everything-1', instName: 'everything', serverId: 'everything' };
  const contactSchema = window.CONTACT_ELICIT_SCHEMA || { type: 'object', properties: {}, required: [] };
  const sumParams = window.SUMMARIZE_SAMPLING_PARAMS || { messages: [], systemPrompt: '' };

  const ADDRESS = { type: 'object', properties: {
      street: { type: 'string', title: 'Street address', description: 'Where should we ship it?' },
      city: { type: 'string', title: 'City' },
      country: { type: 'string', title: 'Country', enum: ['US', 'UK', 'India', 'Germany'], enumNames: ['United States', 'United Kingdom', 'India', 'Germany'] },
    }, required: ['street', 'city'] };
  const PHONE = { type: 'object', properties: {
      phone: { type: 'string', title: 'Phone number', description: 'For SMS delivery updates' },
    }, required: ['phone'] };
  const CLASSIFY = { messages: [{ role: 'user', content: { type: 'text', text: 'Classify this customer feedback as positive, neutral, or negative, and explain in one line:\n\n“Honestly the new playground is a joy to use — fast and clear.”' } }],
    systemPrompt: 'You are a precise text classifier.', modelPreferences: { hints: [{ name: 'claude-3-haiku' }], intelligencePriority: 0.4, speedPriority: 0.9, costPriority: 0.7 }, maxTokens: 60, temperature: 0.2 };
  const DRAFT = { messages: [{ role: 'user', content: { type: 'text', text: 'Draft a friendly two-sentence reply thanking the sender and confirming the meeting for Thursday at 3pm.' } }],
    systemPrompt: 'You write warm, concise emails.', modelPreferences: { hints: [{ name: 'claude' }] }, maxTokens: 120, temperature: 0.6 };

  const state = {
    v: 2,
    elicit: [
      { id: 'el-seed-wait', kind: 'elicitation', status: 'waiting', ts: now - 2 * M, ...I,
        fromToolName: 'collect_contact_info', fromToolTitle: 'Request your details', runId: 'run-seed-el',
        message: 'Please provide your contact information', requestedSchema: contactSchema },
      { id: 'el-seed-prov', kind: 'elicitation', status: 'provided', ts: now - 3 * H, resolvedTs: now - 3 * H + 40000, ...I,
        fromToolName: 'collect_contact_info', fromToolTitle: 'Request your details', action: 'accept',
        message: 'Confirm your shipping address', requestedSchema: ADDRESS,
        content: { street: '12 Banyan Road', city: 'Bengaluru', country: 'India' } },
      { id: 'el-seed-decl', kind: 'elicitation', status: 'declined', ts: now - 26 * H, resolvedTs: now - 26 * H + 12000, ...I,
        fromToolName: 'create_note', fromToolTitle: 'Create a note', action: 'decline',
        message: 'Share a phone number for SMS updates', requestedSchema: PHONE, content: null },
    ],
    sampling: [
      { id: 'sm-seed-wait', kind: 'sampling', status: 'waiting', ts: now - 5 * M, ...I,
        fromToolName: 'summarize_with_ai', fromToolTitle: 'Summarise with AI', runId: 'run-seed-sm',
        title: 'Summarise the meeting notes into 3 bullets', params: sumParams },
      { id: 'sm-seed-appr', kind: 'sampling', status: 'approved', ts: now - 5 * H, resolvedTs: now - 5 * H + 9000, ...I,
        fromToolName: 'summarize_with_ai', fromToolTitle: 'Summarise with AI', action: 'approve',
        title: 'Classify this customer feedback', params: CLASSIFY, model: 'llama-3.1-8b',
        answer: '**Sentiment: Positive.** The user praises the playground as fast and clear, expressing clear satisfaction.' },
      { id: 'sm-seed-decl', kind: 'sampling', status: 'declined', ts: now - 30 * H, resolvedTs: now - 30 * H + 6000, ...I,
        fromToolName: 'summarize_with_ai', fromToolTitle: 'Summarise with AI', action: 'decline',
        title: 'Draft a reply to this email', params: DRAFT },
    ],
    runs: {
      'run-seed-el': { runId: 'run-seed-el', instId: I.instId, toolName: 'collect_contact_info', interaction: 'elicitation', requestId: 'el-seed-wait', status: 'waiting', ts: now - 2 * M },
      'run-seed-sm': { runId: 'run-seed-sm', instId: I.instId, toolName: 'summarize_with_ai', interaction: 'sampling', requestId: 'sm-seed-wait', status: 'waiting', ts: now - 5 * M },
    },
  };
  try { localStorage.setItem(LIVE_NS, JSON.stringify(state)); } catch (e) {}
}

/* ════════════════════════════════════════════════════════════════
   3 · REACTIVITY HOOKS
═══════════════════════════════════════════════════════════════════ */
function useLiveBump() {
  const [n, setN] = useState(0);
  useEffect(() => {
    const h = () => setN(x => x + 1);
    window.addEventListener('pg-live', h);
    window.addEventListener('storage', h);
    return () => { window.removeEventListener('pg-live', h); window.removeEventListener('storage', h); };
  }, []);
  return n;
}

/* ════════════════════════════════════════════════════════════════
   4 · SMALL HELPERS / PRIMITIVES
═══════════════════════════════════════════════════════════════════ */
function relTime(ts) {
  if (!ts) return '';
  const s = Math.round((Date.now() - ts) / 1000);
  if (s < 45) return 'just now';
  const m = Math.round(s / 60); if (m < 60) return m + 'm ago';
  const h = Math.round(m / 60); if (h < 24) return h + 'h ago';
  const d = Math.round(h / 24); if (d === 1) return 'yesterday'; if (d < 7) return d + 'd ago';
  return new Date(ts).toLocaleDateString();
}
function truncate(str, n) { str = String(str || ''); return str.length > n ? str.slice(0, n - 1).trimEnd() + '…' : str; }
function msgText(m) { const c = m && m.content; if (c == null) return ''; if (typeof c === 'string') return c; if (c.text != null) return c.text; return JSON.stringify(c, null, 2); }

const ST_META = {
  waiting:   { label: 'Waiting',   cls: 'waiting',   dot: true },
  provided:  { label: 'Provided',  cls: 'provided',  icon: 'check' },
  approved:  { label: 'Approved',  cls: 'approved',  icon: 'check' },
  declined:  { label: 'Declined',  cls: 'declined',  icon: 'ban' },
  cancelled: { label: 'Cancelled', cls: 'cancelled', icon: 'x' },
  expired:   { label: 'Expired',   cls: 'expired',   icon: 'clock' },
  error:     { label: 'Error',     cls: 'error',     icon: 'circle-alert' },
};
function StatusChip({ status }) {
  const m = ST_META[status] || ST_META.waiting;
  return <span className={'pg-st ' + m.cls}>{m.dot ? <span className="pg-livedot" /> : <Ic name={m.icon} size={11} />}{m.label}</span>;
}

/* the inbox row (list side) */
function InboxRow({ req }) {
  const from = req.fromToolTitle || req.fromToolName || 'a tool';
  return (
    <>
      <span className="pg-inbox-top">
        <span className="pg-inbox-title">{truncate(req.message || req.title, 58)}</span>
        <StatusChip status={req.status} />
      </span>
      <span className="pg-inbox-meta">
        <span className="pg-inbox-from"><Ic name="wrench" size={11} />{from}</span>
        <span className="pg-inbox-dot" />
        <span className="pg-inbox-time">{relTime(req.ts)}</span>
      </span>
    </>
  );
}

/* collapsed wire payload (the Developer / raw view) */
function RawDisclosure({ payload, tag, label }) {
  const [open, setOpen] = useState(false);
  const json = typeof payload === 'string' ? payload : JSON.stringify(payload, null, 2);
  return (
    <div className={'pg-raw' + (open ? ' open' : '')}>
      <button type="button" className="pg-raw-sum" onClick={() => setOpen(o => !o)}>
        <Ic name={open ? 'chevron-down' : 'chevron-right'} size={13} />
        <Ic name="code-2" size={13} /> <span>{label || 'Raw'}</span>
        <span className="pg-raw-tag">{tag}</span>
      </button>
      {open && (
        <div className="pg-raw-body">
          <div className="pg-raw-copy"><CopyBtn text={json} /></div>
          <pre className="pg-code" dangerouslySetInnerHTML={{ __html: syntaxHighlight(json) }} />
        </div>
      )}
    </div>
  );
}

/* "which run is asking" line */
function AskingLine({ req }) {
  return (
    <div className="pg-asking">
      <span>Asked by</span>
      <span className="pg-asking-chip"><Ic name="wrench" size={12} />{req.fromToolTitle || req.fromToolName}</span>
      <span>on <b>{req.instName}</b> · the run is {req.status === 'waiting' ? <b>waiting on your answer</b> : 'resolved'}</span>
    </div>
  );
}

/* ════════════════════════════════════════════════════════════════
   5 · SCHEMA-DRIVEN AUTO-FORM  (the elicitation form builder)
═══════════════════════════════════════════════════════════════════ */
function fieldKind(p) {
  if (!p) return 'string';
  if (p.type === 'boolean') return 'boolean';
  if (Array.isArray(p.oneOf)) return 'oneof';
  if (p.type === 'array') return 'multi';
  if (Array.isArray(p.enum)) return 'enum';
  if (p.type === 'number' || p.type === 'integer') return 'number';
  return 'string';
}
function enumOptions(p) {
  if (Array.isArray(p.oneOf)) return p.oneOf.map(o => ({ value: o.const, label: o.title || String(o.const) }));
  const names = p.enumNames || [];
  return (p.enum || []).map((v, i) => ({ value: v, label: names[i] || String(v) }));
}
function itemOptions(p) {
  const it = p.items || {};
  const names = it.enumNames || [];
  return (it.enum || []).map((v, i) => ({ value: v, label: names[i] || String(v) }));
}
function schemaDefaults(schema) {
  const out = {}; const props = (schema && schema.properties) || {};
  Object.entries(props).forEach(([k, p]) => {
    if (p.type === 'boolean') out[k] = p.default != null ? p.default : false;
    else if (p.default !== undefined) out[k] = p.default;
    else if (p.type === 'array') out[k] = [];
  });
  return out;
}
function validateSchema(schema, values) {
  const errs = {}; const props = (schema && schema.properties) || {}; const req = (schema && schema.required) || [];
  Object.entries(props).forEach(([k, p]) => {
    const v = values[k];
    const isReq = req.includes(k);
    if (p.type === 'boolean') return;
    if (p.type === 'array') {
      const len = (v || []).length;
      if (isReq && len === 0) errs[k] = true;
      if (p.minItems && len > 0 && len < p.minItems) errs[k] = true;
      if (p.maxItems && len > p.maxItems) errs[k] = true;
      return;
    }
    const empty = v == null || String(v).trim() === '';
    if (isReq && empty) { errs[k] = true; return; }
    if (empty) return;
    if (p.format === 'email' && !/^\S+@\S+\.\S+$/.test(v)) errs[k] = true;
    if (p.format === 'uri' && !/^https?:\/\/\S+/.test(v)) errs[k] = true;
    if (p.type === 'number' || p.type === 'integer') {
      const n = Number(v);
      if (Number.isNaN(n)) errs[k] = true;
      else { if (p.minimum != null && n < p.minimum) errs[k] = true; if (p.maximum != null && n > p.maximum) errs[k] = true; }
    }
    if (p.minLength && String(v).length < p.minLength) errs[k] = true;
    if (p.maxLength && String(v).length > p.maxLength) errs[k] = true;
  });
  return errs;
}
function collectValues(schema, values) {
  const out = {}; const props = (schema && schema.properties) || {}; const req = (schema && schema.required) || [];
  Object.entries(props).forEach(([k, p]) => {
    const v = values[k];
    if (p.type === 'boolean') { out[k] = !!v; return; }
    if (p.type === 'array') { if ((v || []).length || req.includes(k)) out[k] = v || []; return; }
    if (v == null || String(v).trim() === '') return;
    if (p.type === 'number' || p.type === 'integer') out[k] = p.type === 'integer' ? parseInt(v, 10) : Number(v);
    else out[k] = v;
  });
  return out;
}

const INPUT_TYPE = { email: 'email', uri: 'url', date: 'date', 'date-time': 'datetime-local' };
function FieldShell({ label, req, help, children, extra, cls }) {
  return (
    <label className={'pg-field' + (cls ? ' ' + cls : '')}>
      <span className="pg-field-label">
        <span className="pg-field-titlerow">{label}</span>
        {req ? <span className="pg-req">required</span> : <span className="pg-opt">optional</span>}
        {extra}
      </span>
      {children}
      {help && <span className="pg-field-hint">{help}</span>}
    </label>
  );
}
function SchemaField({ name, prop, req, value, onChange, err }) {
  const label = prop.title || prettyKey(name);
  const help = prop.description;
  const kind = fieldKind(prop);

  if (kind === 'boolean') {
    return (
      <div className="pg-field pg-switch-field">
        <button type="button" className={'pg-switch' + (value ? ' on' : '')} aria-pressed={!!value} onClick={() => onChange(!value)} />
        <span className="pg-switch-text">
          <span className="pg-field-label"><span className="pg-field-titlerow">{label}</span>{req ? <span className="pg-req">required</span> : <span className="pg-opt">optional</span>}</span>
          {help && <span className="pg-field-hint">{help}</span>}
        </span>
      </div>
    );
  }
  if (kind === 'enum' || kind === 'oneof') {
    const opts = enumOptions(prop);
    return (
      <FieldShell label={label} req={req} help={help}>
        <select className={'pg-input pg-select' + (err ? ' err' : '')} value={value || ''} onChange={e => onChange(e.target.value)}>
          <option value="" disabled={req}>{req ? 'Choose one…' : '— none —'}</option>
          {opts.map((o, i) => <option key={i} value={o.value}>{o.label}</option>)}
        </select>
      </FieldShell>
    );
  }
  if (kind === 'multi') {
    const opts = itemOptions(prop);
    const arr = value || [];
    const min = prop.minItems, max = prop.maxItems;
    const rangeLabel = max ? (min && min !== max ? `pick ${min}–${max}` : `pick up to ${max}`) : (min ? `pick ${min}+` : 'pick any');
    const toggle = o => {
      const has = arr.includes(o);
      if (has) onChange(arr.filter(x => x !== o));
      else if (!max || arr.length < max) onChange([...arr, o]);
    };
    return (
      <div className={'pg-field pg-chip-group' + (err ? ' err' : '')}>
        <span className="pg-field-label">
          <span className="pg-field-titlerow">{label}</span>
          {req ? <span className="pg-req">required</span> : <span className="pg-opt">optional</span>}
          <span className="pg-pickrange">{rangeLabel}</span>
        </span>
        <div className="pg-chips">
          {opts.map((o, i) => {
            const on = arr.includes(o.value);
            return (
              <button key={i} type="button" className={'pg-chip-opt' + (on ? ' on' : '')} onClick={() => toggle(o.value)}>
                <span className="pg-chip-check"><Ic name="check" size={12} /></span>{o.label}
              </button>
            );
          })}
        </div>
        {help && <span className="pg-field-hint">{help}</span>}
      </div>
    );
  }
  if (kind === 'number') {
    const { minimum: min, maximum: max } = prop;
    const range = (min != null || max != null) ? `${min != null ? min : '−∞'} – ${max != null ? max : '∞'}` : null;
    return (
      <FieldShell label={label} req={req} help={help}>
        <div className="pg-num-row">
          <input type="number" className={'pg-input' + (err ? ' err' : '')} value={value == null ? '' : value}
            min={min} max={max} step={prop.type === 'integer' ? 1 : 'any'} placeholder={prop.default != null ? String(prop.default) : ''}
            onChange={e => onChange(e.target.value)} />
          {range && <span className="pg-range-hint">{range}</span>}
        </div>
      </FieldShell>
    );
  }
  // string (+ formats)
  return (
    <FieldShell label={label} req={req} help={help}>
      <input type={INPUT_TYPE[prop.format] || 'text'} className={'pg-input' + (err ? ' err' : '')}
        value={value || ''} placeholder={prop.default != null ? String(prop.default) : (prop.format === 'email' ? 'name@example.com' : prop.format === 'uri' ? 'https://…' : '')}
        onChange={e => onChange(e.target.value)} />
    </FieldShell>
  );
}
function SchemaForm({ schema, values, onChange, errors }) {
  const props = (schema && schema.properties) || {};
  const req = (schema && schema.required) || [];
  const names = Object.keys(props);
  if (!names.length) return <div className="pg-noargs">No details needed.</div>;
  return (
    <div className="pg-form">
      {names.map(n => (
        <SchemaField key={n} name={n} prop={props[n]} req={req.includes(n)}
          value={values[n]} onChange={v => onChange(n, v)} err={errors[n]} />
      ))}
    </div>
  );
}

/* the returning beat shown right before auto-return to Tools */
function ReturningPanel({ outcome }) {
  const map = {
    accept: { ic: 'check', t: 'Saved — thank you' }, approve: { ic: 'check', t: 'Done' },
    decline: { ic: 'ban', t: 'Declined' }, cancel: { ic: 'x', t: 'Cancelled' },
  };
  const m = map[outcome] || map.accept;
  return (
    <div className="pg-result-waiting">
      <div className="pg-waiting-mark"><Ic name={m.ic} size={22} /></div>
      <div className="pg-waiting-t">{m.t}</div>
      <div className="pg-waiting-s">Taking you back to the tool to finish the run…</div>
    </div>
  );
}

/* ════════════════════════════════════════════════════════════════
   6 · ELICITATION DETAIL
═══════════════════════════════════════════════════════════════════ */
function returnToTool(req) {
  const inst = { instId: req.instId, instName: req.instName, serverId: req.serverId };
  const base = window.capHref ? window.capHref('tools', inst) : 'MCP-Playground-Tools.html?' + instQS(inst);
  window.location.href = base + '&tool=' + encodeURIComponent(req.fromToolName || '') + (req.runId ? '&resume=' + encodeURIComponent(req.runId) : '');
}

function ElicitationDetail({ req, inst }) {
  const [values, setValues] = useState(() => schemaDefaults(req.requestedSchema));
  const [errors, setErrors] = useState({});
  const [returning, setReturning] = useState(null);
  const resolved = req.status !== 'waiting';

  const wire = { method: 'elicitation/create', params: { message: req.message, requestedSchema: req.requestedSchema } };

  const finish = (action) => {
    let content = null;
    if (action === 'accept') {
      const errs = validateSchema(req.requestedSchema, values);
      if (Object.keys(errs).length) { setErrors(errs); setTimeout(() => setErrors({}), 2400); return; }
      content = collectValues(req.requestedSchema, values);
    }
    liveResolveElicitation(req.id, action, content);
    setReturning(action);
    setTimeout(() => { if (req.runId) returnToTool(req); }, 750);
  };

  return (
    <div className="pg-detail" data-screen-label="Elicitation request">
      <div className="pg-req-head">
        <div className="pg-req-head-ico"><Ic name="message-square-dashed" size={18} /></div>
        <div className="pg-req-head-text">
          <div className="pg-req-head-row">
            <span className="pg-req-title">{resolved ? 'Information request' : 'The server needs some information'}</span>
            <span className="pg-dh-tag">Elicitation</span>
            {resolved && <StatusChip status={req.status} />}
          </div>
          <div className="pg-req-message">{req.message}</div>
          <AskingLine req={req} />
        </div>
      </div>

      <div className="pg-detail-scroll">
        {returning ? <ReturningPanel outcome={returning} /> : resolved ? (
          <ResolvedElicitation req={req} />
        ) : (
          <div className="pg-run-card">
            <div className="pg-sec-title"><Ic name="clipboard-list" size={13} /> Provide the details</div>
            <SchemaForm schema={req.requestedSchema} values={values} errors={errors}
              onChange={(n, v) => setValues(p => ({ ...p, [n]: v }))} />
            <div className="pg-safety"><Ic name="shield-check" size={14} /> Only share what you’re comfortable with. Bodhi never asks for passwords or full card numbers here — and you can always Decline or Cancel.</div>
            <div className="pg-actions">
              <button className="pg-btn pg-btn-primary" onClick={() => finish('accept')}><Ic name="check" size={15} /> Submit</button>
              <button className="pg-btn pg-btn-decline" onClick={() => finish('decline')}><Ic name="ban" size={14} /> Decline</button>
              <button className="pg-btn pg-btn-cancel" onClick={() => finish('cancel')}>Cancel</button>
            </div>
          </div>
        )}
        {!returning && <RawDisclosure payload={wire} tag="elicitation/create" label="Raw request" />}
      </div>
    </div>
  );
}
function ResolvedElicitation({ req }) {
  const note = { provided: 'You provided these details:', declined: 'You declined this request — the server was told no.', cancelled: 'You cancelled this request before answering.' }[req.status];
  return (
    <>
      <div className={'pg-answered ' + req.status}>
        <div className="pg-answered-head"><StatusChip status={req.status} /> {note}</div>
        {req.status === 'provided' && (
          <div className="pg-answered-body"><DataView value={req.content} /></div>
        )}
      </div>
      {req.runId && (
        <div className="pg-actions">
          <a className="pg-btn pg-btn-ghost" href={(window.capHref ? window.capHref('tools', { instId: req.instId, instName: req.instName, serverId: req.serverId }) : '#') + '&tool=' + encodeURIComponent(req.fromToolName || '') + '&resume=' + encodeURIComponent(req.runId)}>
            <Ic name="arrow-up-right" size={14} /> View the tool’s result
          </a>
        </div>
      )}
    </>
  );
}

/* ════════════════════════════════════════════════════════════════
   7 · SAMPLING DETAIL
═══════════════════════════════════════════════════════════════════ */
function SamplingPreview({ params }) {
  return (
    <div className="pg-chatprev">
      {params.systemPrompt && (
        <div className="pg-cbub system">
          <div className="pg-cbub-role"><Ic name="settings-2" size={11} /> System</div>
          <div className="pg-cbub-body"><Markdownish text={params.systemPrompt} /></div>
        </div>
      )}
      {(params.messages || []).map((m, i) => (
        <div key={i} className={'pg-cbub ' + (m.role || 'user')}>
          <div className="pg-cbub-role"><Ic name={m.role === 'assistant' ? 'sparkles' : 'user'} size={11} /> {m.role || 'user'}</div>
          <div className="pg-cbub-body"><Markdownish text={msgText(m)} /></div>
        </div>
      ))}
    </div>
  );
}
function cannedAnswer(req) {
  const t = (req.title || '').toLowerCase();
  if (t.includes('classif')) return '**Sentiment: Positive.** The message praises the playground as “a joy to use — fast and clear,” which is clearly favourable.';
  if (t.includes('draft') || t.includes('reply')) return 'Thanks so much for reaching out — I really appreciate it! I can confirm Thursday at 3pm works perfectly, and I’m looking forward to it.';
  return '• The team aligned on shipping the v1 playground interactions behind a flag.\n• Elicitation and sampling each get their **own page** for clarity and stability.\n• Completion stays inline and ships last.\n\n**Next step:** book a 30-minute review once Phase 1 lands.';
}

function SamplingDetail({ req, inst }) {
  const models = samplingModels();
  const [model, setModel] = useState(() => req.model || lastChatModel());
  const [phase, setPhase] = useState('idle');           // idle · asking · answered · returning
  const [answerText, setAnswerText] = useState('');
  const [review, setReview] = useState(false);
  const [returnOutcome, setReturnOutcome] = useState(null);
  const resolved = req.status !== 'waiting';

  const wire = { method: 'sampling/createMessage', params: req.params };

  const send = (text) => {
    setPhase('returning'); setReturnOutcome('approve');
    writeLastModel(model);
    liveResolveSampling(req.id, 'approve', { model, answer: text });
    setTimeout(() => { if (req.runId) returnToTool(req); }, 750);
  };
  const approve = () => {
    setPhase('asking');
    setTimeout(() => {
      const text = cannedAnswer(req);
      setAnswerText(text);
      if (review) setPhase('answered'); else send(text);
    }, 850);
  };
  const decline = () => {
    setPhase('returning'); setReturnOutcome('decline');
    liveResolveSampling(req.id, 'decline', null);
    setTimeout(() => { if (req.runId) returnToTool(req); }, 750);
  };

  return (
    <div className="pg-detail" data-screen-label="Sampling request">
      <div className="pg-req-head sampling">
        <div className="pg-req-head-ico"><Ic name="sparkles" size={18} /></div>
        <div className="pg-req-head-text">
          <div className="pg-req-head-row">
            <span className="pg-req-title">{resolved ? 'AI request' : 'The server wants to use the AI'}</span>
            <span className="pg-dh-tag">Sampling</span>
            {resolved && <StatusChip status={req.status} />}
          </div>
          <div className="pg-req-message">{resolved ? req.title : 'It needs a model to complete a step. Bodhi will run it against one of your own models — the server has no AI of its own.'}</div>
          <AskingLine req={req} />
        </div>
      </div>

      <div className="pg-detail-scroll">
        {phase === 'returning' ? <ReturningPanel outcome={returnOutcome} /> :
         resolved ? <ResolvedSampling req={req} /> : (
          <>
            <div className="pg-run-card">
              <div className="pg-sec-title"><Ic name="messages-square" size={13} /> What the server wants the AI to do</div>
              <SamplingPreview params={req.params} />
              <div className="pg-sampling-meta">
                {req.params.maxTokens != null && <span className="pg-meta-ic"><Ic name="ruler" size={12} /> max tokens <b>{req.params.maxTokens}</b></span>}
                {req.params.temperature != null && <span className="pg-meta-ic"><Ic name="thermometer" size={12} /> temperature <b>{req.params.temperature}</b></span>}
              </div>
            </div>

            {phase === 'asking' ? (
              <div className="pg-run-card">
                <div className="pg-asking-ai">
                  <div className="pg-asking-ai-mark"><PgSpinner size={20} /></div>
                  <div className="pg-asking-ai-t">Asking the AI…</div>
                  <div className="pg-asking-ai-s">Running against <span className="mono">{model}</span></div>
                </div>
              </div>
            ) : phase === 'answered' ? (
              <div className="pg-run-card">
                <div className="pg-sec-title"><Ic name="sparkles" size={13} /> The AI’s answer — review before sending</div>
                <div className="pg-answer">
                  <div className="pg-answer-head"><Ic name="check" size={13} /> Answer ready<span className="pg-answer-model">{model}</span></div>
                  <div className="pg-answer-review">
                    <textarea value={answerText} onChange={e => setAnswerText(e.target.value)} />
                  </div>
                </div>
                <div className="pg-actions">
                  <button className="pg-btn pg-btn-primary" onClick={() => send(answerText)}><Ic name="send" size={14} /> Send to server</button>
                  <button className="pg-btn pg-btn-cancel" onClick={() => setPhase('idle')}>Back</button>
                </div>
              </div>
            ) : (
              <div className="pg-run-card">
                <div className="pg-sec-title"><Ic name="cpu" size={13} /> Approve &amp; run</div>
                {models.length === 0 ? (
                  <div className="pg-nomodels">
                    <span className="pg-nomodels-t"><Ic name="plug" size={15} /> No models connected</span>
                    <span className="pg-nomodels-s">To let a server use the AI, add a model first. You can connect a local or API model from the Models page, then come back to approve this request.</span>
                    <a className="pg-btn pg-btn-ghost" href="Bodhi Models.html"><Ic name="arrow-up-right" size={14} /> Connect a model</a>
                  </div>
                ) : (
                  <>
                    <div className="pg-model-field">
                      <span className="pg-model-label"><Ic name="cpu" size={14} /> Run it with
                        {model === lastChatModel() && <span className="pg-model-now"><Ic name="history" size={11} /> last used in chat</span>}
                      </span>
                      <select className="pg-input pg-select" value={model} onChange={e => setModel(e.target.value)}>
                        {models.map(m => <option key={m} value={m}>{m}</option>)}
                      </select>
                      <span className="pg-model-hint">Bodhi runs this against your own model — the server borrows it and never sees your keys.</span>
                    </div>
                    <label className="pg-dev-toggle" style={{ marginTop: 14 }}>
                      <button type="button" className={'pg-switch' + (review ? ' on' : '')} aria-pressed={review} onClick={() => setReview(r => !r)} />
                      Review the AI’s answer before it’s sent back <span className="pg-raw-tag">Developer</span>
                    </label>
                    <div className="pg-actions">
                      <button className="pg-btn pg-btn-primary" onClick={approve}><Ic name="play" size={14} /> Approve &amp; run</button>
                      <button className="pg-btn pg-btn-decline" onClick={decline}><Ic name="ban" size={14} /> Decline</button>
                    </div>
                  </>
                )}
              </div>
            )}
          </>
        )}
        {phase !== 'returning' && <RawDisclosure payload={wire} tag="sampling/createMessage" label="Raw request" />}
      </div>
    </div>
  );
}
function ResolvedSampling({ req }) {
  if (req.status === 'declined') {
    return (
      <div className="pg-answered declined">
        <div className="pg-answered-head"><StatusChip status="declined" /> You declined — the server couldn’t use the AI for this step.</div>
      </div>
    );
  }
  const respWire = { role: 'assistant', content: { type: 'text', text: req.answer }, model: req.model, stopReason: 'endTurn' };
  return (
    <>
      <div className="pg-run-card">
        <div className="pg-sec-title"><Ic name="messages-square" size={13} /> What the server asked the AI</div>
        <SamplingPreview params={req.params} />
      </div>
      <div className="pg-run-card">
        <div className="pg-sec-title"><Ic name="sparkles" size={13} /> The AI’s answer</div>
        <div className="pg-answer">
          <div className="pg-answer-head"><Ic name="check" size={13} /> Sent back to the server<span className="pg-answer-model">{req.model}</span></div>
          <div className="pg-answer-body"><Markdownish text={req.answer} /></div>
        </div>
        <RawDisclosure payload={respWire} tag="result" label="Raw response" />
      </div>
    </>
  );
}

/* ════════════════════════════════════════════════════════════════
   8 · TOOLS-SIDE "WAITING ON YOU…" + the live empty state
═══════════════════════════════════════════════════════════════════ */
function WaitingPanel({ switching, interaction, requestId, inst }) {
  const label = interaction === 'sampling' ? 'Sampling' : 'Elicitation';
  if (switching) {
    return (
      <div className="pg-result">
        <div className="pg-result-waiting">
          <div className="pg-waiting-mark"><Ic name="arrow-right" size={22} /></div>
          <div className="pg-waiting-t">The server needs you</div>
          <div className="pg-waiting-s">Taking you to the {label} page to respond…</div>
        </div>
      </div>
    );
  }
  const href = (window.capHref ? window.capHref(interaction, inst) : '#') + '&focus=' + encodeURIComponent(requestId);
  return (
    <div className="pg-result">
      <div className="pg-result-waiting">
        <div className="pg-waiting-mark"><Ic name="hourglass" size={22} /></div>
        <div className="pg-waiting-t">Waiting on you…</div>
        <div className="pg-waiting-s">This run paused — the server is waiting for your response on the {label} page. The result will appear here once you’ve answered.</div>
        <a className="pg-waiting-go" href={href}><Ic name="arrow-up-right" size={15} /> Go to the request</a>
      </div>
    </div>
  );
}
function LiveEmpty({ cap }) {
  const cfg = (window.CAP_CONFIG && window.CAP_CONFIG[cap] && window.CAP_CONFIG[cap].emptyCentre) || {};
  return (
    <div className="pg-live-empty">
      <div className="pg-live-empty-mark"><Ic name={cfg.icon || 'inbox'} size={26} /></div>
      <div className="pg-live-empty-t">{cfg.title || 'Nothing waiting'}</div>
      <div className="pg-live-empty-s">{cfg.desc || 'Requests will show up here.'}</div>
    </div>
  );
}

/* ════════════════════════════════════════════════════════════════
   9 · LIVE_CONFIG  (merged into CAP_CONFIG by pg-views)
═══════════════════════════════════════════════════════════════════ */
const LIVE_CONFIG = {
  elicitation: {
    live: true,
    listTitle: 'Requests',
    getItems: (sid, instId) => liveElicitations(instId),
    getId: r => r.id,
    searchKeys: ['message', 'fromToolTitle', 'fromToolName'],
    searchPlaceholder: 'Search requests…',
    renderRow: r => <InboxRow req={r} />,
    renderDetail: (r, inst) => <ElicitationDetail key={r.id} req={r} inst={inst} />,
    empty: { icon: 'inbox', title: 'No requests', desc: 'When a server needs information from you, it’ll show up here.' },
    emptyCentre: { icon: 'message-square-dashed', title: 'Nothing to fill in — yet', desc: 'When a tool needs details from you, the request lands here and you’re brought over automatically. Try “Request your details” on the Tools page.' },
    pick: 'a request',
  },
  sampling: {
    live: true,
    listTitle: 'Requests',
    getItems: (sid, instId) => liveSamplings(instId),
    getId: r => r.id,
    searchKeys: ['title', 'fromToolTitle', 'fromToolName'],
    searchPlaceholder: 'Search requests…',
    renderRow: r => <InboxRow req={r} />,
    renderDetail: (r, inst) => <SamplingDetail key={r.id} req={r} inst={inst} />,
    empty: { icon: 'sparkles', title: 'No requests', desc: 'When a server wants to use the AI, its request appears here for your approval.' },
    emptyCentre: { icon: 'sparkles', title: 'No AI requests — yet', desc: 'When a tool wants to borrow your AI, you’ll approve it here first. Try “Summarise with AI” on the Tools page.' },
    pick: 'a request',
  },
};
function samplingModels() { return LIVE_MODELS.slice(); }

Object.assign(window, {
  LIVE_NS, LIVE_MODELS, liveState, liveElicitations, liveSamplings, liveList,
  liveGetRun, liveLatestRunForTool, pendingCounts,
  liveAddElicitation, liveAddSampling, liveAddRun, liveResolveElicitation, liveResolveSampling,
  buildElicitResult, buildSamplingResult, runToDisplay, uid,
  lastChatModel, writeLastModel, samplingParamsWithNotes, samplingModels,
  useLiveBump, relTime, truncate, msgText, StatusChip, InboxRow, RawDisclosure,
  fieldKind, enumOptions, schemaDefaults, validateSchema, collectValues, SchemaForm, SchemaField,
  ElicitationDetail, SamplingDetail, SamplingPreview, WaitingPanel, LiveEmpty, ReturningPanel,
  LIVE_CONFIG, ensureSeed,
});
