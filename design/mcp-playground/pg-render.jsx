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

/* ── light markdown (headings, bullets, **bold**, `code`, links,
   > quotes, fenced code blocks and tables) ──────────────────── */
function inlineMd(text) {
  const nodes = [];
  const re = /(\*\*[^*]+\*\*|`[^`]+`|\[[^\]]+\]\([^)]+\))/g;
  let last = 0, m, i = 0;
  while ((m = re.exec(text))) {
    if (m.index > last) nodes.push(<React.Fragment key={i++}>{text.slice(last, m.index)}</React.Fragment>);
    const tok = m[0];
    if (tok.startsWith('**')) nodes.push(<strong key={i++}>{tok.slice(2, -2)}</strong>);
    else if (tok.startsWith('`')) nodes.push(<code key={i++} className="md-code">{tok.slice(1, -1)}</code>);
    else {
      const mm = tok.match(/^\[([^\]]+)\]\(([^)]+)\)$/);
      nodes.push(<a key={i++} className="md-link" href={mm[2]} target="_blank" rel="noopener noreferrer">{mm[1]}</a>);
    }
    last = m.index + tok.length;
  }
  if (last < text.length) nodes.push(<React.Fragment key={i++}>{text.slice(last)}</React.Fragment>);
  return nodes;
}
function Markdownish({ text }) {
  const lines = String(text).split('\n');
  const out = [];
  let bullets = null, i = 0;
  const flush = () => { if (bullets) { out.push(<ul className="md-ul" key={'ul' + out.length}>{bullets}</ul>); bullets = null; } };
  const parseRow = r => r.replace(/^\s*\|/, '').replace(/\|\s*$/, '').split('|').map(c => c.trim());
  while (i < lines.length) {
    const t = lines[i].trimEnd();
    if (/^```/.test(t.trim())) {                       // fenced code block
      flush(); const buf = []; i++;
      while (i < lines.length && !/^```/.test(lines[i].trim())) { buf.push(lines[i]); i++; }
      i++;
      out.push(<pre className="md-pre" key={'cd' + out.length}><code>{buf.join('\n')}</code></pre>);
      continue;
    }
    if (t.includes('|') && i + 1 < lines.length && /-/.test(lines[i + 1]) && /^[\s:|-]+$/.test(lines[i + 1].trim())) {
      flush();                                          // table
      const headers = parseRow(t); i += 2; const rows = [];
      while (i < lines.length && lines[i].includes('|') && lines[i].trim() !== '') { rows.push(parseRow(lines[i])); i++; }
      out.push(
        <div className="md-table-wrap" key={'tb' + out.length}>
          <table className="md-table">
            <thead><tr>{headers.map((h, hi) => <th key={hi}>{inlineMd(h)}</th>)}</tr></thead>
            <tbody>{rows.map((r, ri) => <tr key={ri}>{r.map((c, ci) => <td key={ci}>{inlineMd(c)}</td>)}</tr>)}</tbody>
          </table>
        </div>
      );
      continue;
    }
    if (/^#{1,3}\s/.test(t)) { flush(); const lvl = t.match(/^#+/)[0].length; out.push(<div className={'md-h md-h' + lvl} key={i}>{inlineMd(t.replace(/^#+\s/, ''))}</div>); }
    else if (/^>\s?/.test(t)) { flush(); out.push(<blockquote className="md-quote" key={i}>{inlineMd(t.replace(/^>\s?/, ''))}</blockquote>); }
    else if (/^[-*]\s/.test(t)) { (bullets = bullets || []).push(<li key={i}>{inlineMd(t.replace(/^[-*]\s/, ''))}</li>); }
    else if (t === '') { flush(); }
    else { flush(); out.push(<p className="md-p" key={i}>{inlineMd(t)}</p>); }
    i++;
  }
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

/* ══ BEHAVIOUR HINTS — friendly chips; protocol term in the tooltip ══ */
function BehaviourHints({ tool }) {
  const hints = hintsForTool(tool);
  if (!hints.length) return null;
  return (
    <div className="pg-hints">
      {hints.map(h => (
        <span key={h.key} className={'pg-hint shell-tip tone-' + h.tone} data-tip={h.term + ' — ' + h.tip}>
          <Ic name={h.icon} size={11} /> <span className="pg-hint-label">{h.label}</span>
        </span>
      ))}
    </div>
  );
}

/* ── one content block (text/image/link/embedded resource) ──── */
function TextBlock({ block }) {
  const fmt = block.format || 'markdown';
  if (fmt === 'pre') return <pre className="pg-pre"><code>{block.text}</code></pre>;
  if (fmt === 'plain') return <div className="pg-plain">{block.text}</div>;
  return <Markdownish text={block.text} />;
}
const IMG_TINT = { indigo: 'var(--c-indigo-text)', lotus: 'var(--c-lotus-text)', leaf: 'var(--c-leaf-text)', saffron: 'var(--c-saffron-text)' };
function ImageBlock({ block }) {
  const w = block.w || 240, h = block.h || 150;
  const accent = IMG_TINT[block.tint] || 'var(--c-indigo-text)';
  return (
    <figure className="pg-img">
      <div className="pg-img-tile" style={{ aspectRatio: w + ' / ' + h, '--img-accent': accent }} role="img" aria-label={block.alt || block.name}>
        <Ic name="image" size={22} />
        <span className="pg-img-dims">{w}×{h}</span>
      </div>
      <figcaption className="pg-img-cap">
        <span className="mono">{block.name || 'image'}</span>
        <span className="pg-img-mime">{block.mimeType || 'image/png'}</span>
      </figcaption>
    </figure>
  );
}
function ResourceLinkBlock({ block }) {
  const { openResource } = usePgNav();
  return (
    <button type="button" className="pg-rlink" onClick={() => openResource(block)}>
      <span className="pg-rlink-ico"><Ic name="file-text" size={15} /></span>
      <span className="pg-rlink-body">
        <span className="pg-rlink-name">{block.name || block.uri}</span>
        {block.description && <span className="pg-rlink-desc">{block.description}</span>}
        <span className="pg-rlink-uri mono">{block.uri}</span>
      </span>
      <span className="pg-rlink-go"><Ic name="arrow-up-right" size={14} /></span>
    </button>
  );
}
function ResourceBlock({ block }) {
  const isText = /text|markdown|json/.test(block.mimeType || '');
  return (
    <div className="pg-embed">
      <div className="pg-embed-head"><Ic name="paperclip" size={12} /> <span className="mono">{block.uri}</span><span className="pg-mime">{block.mimeType}</span></div>
      {isText ? <Markdownish text={block.text || ''} /> : <DataView value={block.text} />}
    </div>
  );
}
function ContentBlock({ block }) {
  if (block.type === 'image') return <ImageBlock block={block} />;
  if (block.type === 'resource_link') return <ResourceLinkBlock block={block} />;
  if (block.type === 'resource') return <ResourceBlock block={block} />;
  return <TextBlock block={block} />;
}

/* structured data — caption a single {key:[rows]} as a titled table ── */
function StructuredView({ data }) {
  if (data && typeof data === 'object' && !Array.isArray(data)) {
    const keys = Object.keys(data);
    if (keys.length === 1 && Array.isArray(data[keys[0]]) && data[keys[0]].every(v => v && typeof v === 'object')) {
      return (
        <div className="pg-structured">
          <div className="pg-structured-cap">{prettyKey(keys[0])}</div>
          <DataTable rows={data[keys[0]]} />
        </div>
      );
    }
  }
  if (Array.isArray(data) && data.every(v => v && typeof v === 'object')) return <DataTable rows={data} />;
  return <DataView value={data} />;
}

/* ── tool result: ordered content blocks + optional structured data ── */
function StructDetails({ data, defaultOpen }) {
  const [open, setOpen] = useState(defaultOpen !== false);
  return (
    <div className={'pg-struct-wrap' + (open ? ' open' : '')}>
      <button type="button" className="pg-struct-sum" onClick={() => setOpen(o => !o)}>
        <Ic name={open ? 'chevron-down' : 'chevron-right'} size={12} />
        <Ic name="table-2" size={12} /> <span>Structured data</span>
      </button>
      {open && <div className="pg-struct-body"><StructuredView data={data} /></div>}
    </div>
  );
}
function ToolResultView({ model }) {
  const blocks = model.content || [];
  const links = blocks.filter(b => b.type === 'resource_link');
  const nonLinks = blocks.filter(b => b.type !== 'resource_link');
  return (
    <div className="pg-toolresult">
      {nonLinks.map((b, i) => <ContentBlock key={i} block={b} />)}
      {links.length > 0 && (
        <div className="pg-rlinks">
          {nonLinks.length > 0 && <div className="pg-rlinks-cap">Linked resources</div>}
          {links.map((b, i) => <ResourceLinkBlock key={i} block={b} />)}
        </div>
      )}
      {model.structuredContent !== undefined && <StructDetails data={model.structuredContent} defaultOpen={true} />}
      {blocks.length === 0 && model.structuredContent === undefined && <div className="pg-noresult">The tool returned nothing.</div>}
    </div>
  );
}

/* ── readable switchboard by result kind ─────────────────────── */
function ReadableResult({ run }) {
  if (run.kind === 'tool') return <ToolResultView model={run.data} />;
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
   Shared output surface. The friendly Result reads first and is selected
   on load; Raw and Request sit alongside it, always one click away. */
function ResultPanel({ run, title = 'Result', emptyHint }) {
  const [tab, setTab] = useState('readable');
  useEffect(() => { setTab('readable'); }, [run && run.token]);

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
          {run.request && <button className={'pg-rtab' + (tab === 'request' ? ' on' : '')} onClick={() => setTab('request')}>Request</button>}
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
  CopyBtn, ArgForm, Markdownish, inlineMd, DataView, DataTable,
  TextContentView, MessagesView, ReadableResult, ResultPanel, normalizeText,
  BehaviourHints, ToolResultView, ContentBlock, ResourceLinkBlock, StructuredView,
});
