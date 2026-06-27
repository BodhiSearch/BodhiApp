/* ═══════════════════════════════════════════════════════════════
   BODHI MCP — PLAYGROUND RENDERERS
   mcp/pg-render.jsx   (load AFTER pg-shared.jsx, BEFORE pg-views)

   Turns raw MCP payloads into something a non-technical person can
   read: friendly argument forms, recursive data → cards/tables, light
   markdown for text content, chat bubbles for prompt messages, and the
   shared Result panel with its tucked-away Raw / Request tabs.
═══════════════════════════════════════════════════════════════ */

/* ── copy button ─────────────────────────────────────────────── */
function CopyBtn({ text, label }) {
  const [done, setDone] = useState(false);
  const copy = () => {
    navigator.clipboard && navigator.clipboard.writeText(typeof text === 'string' ? text : JSON.stringify(text, null, 2));
    setDone(true); setTimeout(() => setDone(false), 1400);
  };
  return (
    <button className="pg-copy" onClick={copy} title="Copy">
      <Ic name={done ? 'check' : 'copy'} size={13} /> {label && <span>{done ? 'Copied' : label}</span>}
    </button>
  );
}

/* ── friendly argument form ──────────────────────────────────── */
function ArgForm({ args, values, onChange, errors }) {
  if (!args || args.length === 0) {
    return <div className="pg-noargs">No inputs needed — just run it.</div>;
  }
  return (
    <div className="pg-form">
      {args.map(a => (
        <label className="pg-field" key={a.name}>
          <span className="pg-field-label">
            {a.label || prettyKey(a.name)}
            {a.required && <span className="pg-req">required</span>}
            {!a.required && <span className="pg-opt">optional</span>}
          </span>
          <input className={'pg-input' + (errors && errors[a.name] ? ' err' : '')}
            placeholder={a.placeholder || ''} value={values[a.name] || ''}
            onChange={e => onChange(a.name, e.target.value)} />
          {a.desc && <span className="pg-field-hint">{a.desc}</span>}
        </label>
      ))}
    </div>
  );
}

/* ── light markdown (headings, bullets, **bold**) ────────────── */
function inlineBold(text) {
  const parts = String(text).split(/(\*\*[^*]+\*\*)/g);
  return parts.map((p, i) => /^\*\*[^*]+\*\*$/.test(p)
    ? <strong key={i}>{p.slice(2, -2)}</strong> : <React.Fragment key={i}>{p}</React.Fragment>);
}
function Markdownish({ text }) {
  const lines = String(text).split('\n');
  const out = [];
  let bullets = null;
  const flush = () => { if (bullets) { out.push(<ul className="md-ul" key={'ul' + out.length}>{bullets}</ul>); bullets = null; } };
  lines.forEach((ln, i) => {
    const t = ln.trimEnd();
    if (/^#{1,3}\s/.test(t)) {
      flush();
      const lvl = t.match(/^#+/)[0].length;
      const txt = t.replace(/^#+\s/, '');
      out.push(<div className={'md-h md-h' + lvl} key={i}>{inlineBold(txt)}</div>);
    } else if (/^[-*]\s/.test(t)) {
      (bullets = bullets || []).push(<li key={i}>{inlineBold(t.replace(/^[-*]\s/, ''))}</li>);
    } else if (t === '') {
      flush();
    } else {
      flush();
      out.push(<p className="md-p" key={i}>{inlineBold(t)}</p>);
    }
  });
  flush();
  return <div className="md">{out}</div>;
}

/* ── recursive data view (object → list, array of objects → table) ── */
function DataTable({ rows }) {
  const cols = [];
  rows.forEach(r => Object.keys(r).forEach(k => { if (!cols.includes(k)) cols.push(k); }));
  return (
    <div className="pg-table-wrap">
      <table className="pg-table">
        <thead><tr>{cols.map(c => <th key={c}>{prettyKey(c)}</th>)}</tr></thead>
        <tbody>
          {rows.map((r, i) => (
            <tr key={i}>{cols.map(c => <td key={c}>{r[c] == null ? <span className="dv-null">—</span> : <DataView value={r[c]} compact />}</td>)}</tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
function DataView({ value, compact }) {
  if (value === null || value === undefined) return <span className="dv-null">—</span>;
  if (typeof value === 'string') return compact ? <span className="dv-str">{value}</span> : <span className="dv-str dv-block">{value}</span>;
  if (typeof value === 'number') return <span className="dv-num">{String(value)}</span>;
  if (typeof value === 'boolean') return <span className={'dv-bool ' + (value ? 'on' : 'off')}>{value ? 'Yes' : 'No'}</span>;
  if (Array.isArray(value)) {
    if (value.length === 0) return <span className="dv-null">— empty —</span>;
    const allObj = value.every(v => v && typeof v === 'object' && !Array.isArray(v));
    if (allObj && !compact) return <DataTable rows={value} />;
    return <ul className="dv-list">{value.map((v, i) => <li key={i}><DataView value={v} compact={compact} /></li>)}</ul>;
  }
  if (typeof value === 'object') {
    const entries = Object.entries(value);
    if (compact) return <span className="dv-inline">{entries.map(([k, v], i) => <span key={k} className="dv-chip">{prettyKey(k)}: <DataView value={v} compact /></span>)}</span>;
    return (
      <dl className="dv-obj">
        {entries.map(([k, v]) => (
          <div className="dv-row" key={k}>
            <dt className="dv-key">{prettyKey(k)}</dt>
            <dd className="dv-val"><DataView value={v} /></dd>
          </div>
        ))}
      </dl>
    );
  }
  return <span>{String(value)}</span>;
}

/* ── text content blocks ([{type:'text',text}] | {…} | string) ── */
function normalizeText(data) {
  if (typeof data === 'string') return [data];
  if (Array.isArray(data)) return data.map(b => (b && b.text != null) ? b.text : (typeof b === 'string' ? b : JSON.stringify(b, null, 2)));
  if (data && data.text != null) return [data.text];
  return [JSON.stringify(data, null, 2)];
}
function TextContentView({ data }) {
  return <div className="pg-textblocks">{normalizeText(data).map((t, i) => <Markdownish key={i} text={t} />)}</div>;
}

/* ── prompt messages → chat bubbles ──────────────────────────── */
function MessagesView({ messages }) {
  return (
    <div className="pg-messages">
      {messages.map((m, i) => (
        <div className={'pg-msg pg-msg-' + (m.role || 'user')} key={i}>
          <div className="pg-msg-role">{m.role}</div>
          <div className="pg-msg-body"><Markdownish text={typeof m.content === 'string' ? m.content : JSON.stringify(m.content, null, 2)} /></div>
        </div>
      ))}
    </div>
  );
}

/* ── readable switchboard by result kind ─────────────────────── */
function ReadableResult({ run }) {
  if (run.kind === 'messages') return <MessagesView messages={run.data} />;
  if (run.kind === 'text') return <TextContentView data={run.data} />;
  if (run.kind === 'resource') {
    const c = run.data && run.data.contents;
    const isText = run.data && /text|markdown/.test(run.data.mimeType || '');
    return (
      <div className="pg-resource-out">
        <div className="pg-resource-meta">
          <span className="pg-uri">{run.data.uri}</span>
          <span className="pg-mime">{run.data.mimeType}</span>
        </div>
        {isText ? <TextContentView data={c} /> : <DataView value={c} />}
      </div>
    );
  }
  return <DataView value={run.data} />;
}

/* ══ RESULT PANEL ════════════════════════════════════════════════
   Shared output surface. Readable by default; Raw is always one click
   away; Request shows only in Developer mode. */
function ResultPanel({ run, title = 'Result', emptyHint }) {
  const { dev } = useDev();
  const [tab, setTab] = useState('readable');
  useEffect(() => { setTab('readable'); }, [run && run.token]);
  useEffect(() => { if (tab === 'request' && !dev) setTab('readable'); }, [dev]);

  if (!run || run.phase === 'idle') {
    return (
      <div className="pg-result">
        <div className="pg-result-idle">
          <Ic name="sparkles" size={26} />
          <div className="pg-ri-t">{emptyHint || 'Nothing run yet'}</div>
          <div className="pg-ri-s">Fill in anything needed, then run it to see the result here.</div>
        </div>
      </div>
    );
  }
  if (run.phase === 'running') {
    return (
      <div className="pg-result">
        <div className="pg-result-running"><PgSpinner size={26} /><div>Working…</div></div>
      </div>
    );
  }

  const ok = run.phase === 'done';
  return (
    <div className="pg-result">
      <div className="pg-result-head">
        <span className={'pg-status ' + (ok ? 'ok' : 'err')}>
          <Ic name={ok ? 'circle-check' : 'circle-alert'} size={12} /> {ok ? 'Success' : 'Error'}
        </span>
        {run.meta && <span className="pg-result-meta">{run.meta}</span>}
        <div className="pg-result-tabs">
          <button className={'pg-rtab' + (tab === 'readable' ? ' on' : '')} onClick={() => setTab('readable')}>{title}</button>
          <button className={'pg-rtab' + (tab === 'raw' ? ' on' : '')} onClick={() => setTab('raw')}>Raw</button>
          {dev && run.request && <button className={'pg-rtab' + (tab === 'request' ? ' on' : '')} onClick={() => setTab('request')}>Request</button>}
        </div>
        <CopyBtn text={tab === 'request' ? run.request : (tab === 'raw' ? run.raw : run.data)} />
      </div>
      <div className="pg-result-body">
        {!ok ? (
          <div className="pg-error"><Ic name="circle-alert" size={15} /> {run.error || 'Something went wrong.'}</div>
        ) : tab === 'readable' ? (
          <ReadableResult run={run} />
        ) : tab === 'raw' ? (
          <pre className="pg-code" dangerouslySetInnerHTML={{ __html: syntaxHighlight(run.raw) }} />
        ) : (
          <pre className="pg-code" dangerouslySetInnerHTML={{ __html: syntaxHighlight(JSON.stringify(run.request, null, 2)) }} />
        )}
      </div>
    </div>
  );
}

Object.assign(window, {
  CopyBtn, ArgForm, Markdownish, DataView, DataTable,
  TextContentView, MessagesView, ReadableResult, ResultPanel, normalizeText,
});
