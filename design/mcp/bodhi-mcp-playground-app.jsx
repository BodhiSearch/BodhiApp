/* ═══════════════════════════════════════════════════
   BODHI MCP — PLAYGROUND (on AppShell)
   bodhi-mcp-playground-app.jsx  (load after bodhi-app-shell.jsx)
═══════════════════════════════════════════════════ */
const { useState, useEffect, useRef, useMemo } = React;
const Ic = ShellIcon;

const SERVER_TOOLS = {
  deepwiki: [
    { name: 'read_wiki_structure', desc: 'Get a list of documentation topics for a GitHub repository.',
      params: [{ name: 'repoName', type: 'string', required: true, desc: 'GitHub repository in owner/repo format (e.g. "facebook/react")', placeholder: 'facebook/react' }],
      mockResponse: JSON.stringify([{ type: 'text', text: 'Available pages for facebook/react:\n\n- 1 Overview\n- 2 Architecture\n- 3 Reconciler\n- 4 Hooks\n- 5 Concurrent Mode\n- 6 Server Components\n- 7 Testing' }], null, 2) },
    { name: 'read_wiki_contents', desc: 'View documentation about a GitHub repository or a specific topic.',
      params: [{ name: 'repoName', type: 'string', required: true, desc: 'GitHub repository in owner/repo format', placeholder: 'facebook/react' }, { name: 'topic', type: 'string', required: false, desc: 'Specific topic page to read', placeholder: 'Hooks' }],
      mockResponse: JSON.stringify({ type: 'text', text: 'React Hooks allow you to use state and other React features without writing a class component...' }, null, 2) },
    { name: 'ask_question', desc: 'Ask any question about a GitHub repository.',
      params: [{ name: 'repoName', type: 'string', required: true, desc: 'GitHub repository in owner/repo format', placeholder: 'facebook/react' }, { name: 'question', type: 'string', required: true, desc: 'Your question about the repository', placeholder: 'How does the reconciler work?' }],
      mockResponse: JSON.stringify({ type: 'text', text: 'The React reconciler determines what needs to change in the UI by comparing the current virtual DOM tree with the new one (diffing). It uses a heuristic O(n) algorithm...' }, null, 2) },
  ],
  notion: [
    { name: 'notion-search', desc: 'Perform a search across Notion — "internal" Search api.',
      params: [{ name: 'query', type: 'string', required: true, desc: 'Search query string', placeholder: 'Project notes' }, { name: 'filter', type: 'string', required: false, desc: 'Filter by "page" or "database"', placeholder: 'page' }],
      mockResponse: JSON.stringify({ results: [{ id: 'abc123', title: 'Q1 Project Notes', type: 'page' }, { id: 'def456', title: 'Meeting Notes', type: 'page' }] }, null, 2) },
    { name: 'notion-fetch', desc: 'Retrieves details about a Notion entity (page, database, block).',
      params: [{ name: 'pageId', type: 'string', required: true, desc: 'The Notion page or block ID', placeholder: 'abc123def456' }],
      mockResponse: JSON.stringify({ id: 'abc123', object: 'page', properties: { title: { title: [{ text: { content: 'My Page' } }] } } }, null, 2) },
    { name: 'notion-create-pages', desc: 'Overview: creates one or more Notion pages with given properties.',
      params: [{ name: 'parent_id', type: 'string', required: true, desc: 'Parent page or database ID', placeholder: 'abc123' }, { name: 'title', type: 'string', required: true, desc: 'Page title', placeholder: 'My new page' }, { name: 'content', type: 'string', required: false, desc: 'Initial page content in markdown', placeholder: '## Hello' }],
      mockResponse: JSON.stringify({ id: 'new-page-id', object: 'page', created_time: '2026-05-06T14:00:00Z', url: 'https://notion.so/My-new-page' }, null, 2) },
    { name: 'notion-update-page', desc: 'Overview: update a Notion page properties.',
      params: [{ name: 'page_id', type: 'string', required: true, desc: 'The Notion page ID to update', placeholder: 'abc123' }, { name: 'title', type: 'string', required: false, desc: 'New page title', placeholder: 'Updated title' }],
      mockResponse: JSON.stringify({ id: 'abc123', object: 'page', last_edited_time: '2026-05-06T15:00:00Z' }, null, 2) },
    { name: 'notion-delete', desc: 'Archive or permanently delete a Notion block or page.',
      params: [{ name: 'block_id', type: 'string', required: true, desc: 'The Notion block or page ID to delete', placeholder: 'abc123' }],
      mockResponse: JSON.stringify({ id: 'abc123', object: 'page', archived: true }, null, 2) },
  ],
  exa: [
    { name: 'exa-search', desc: 'Perform a semantic web search using Exa.',
      params: [{ name: 'query', type: 'string', required: true, desc: 'Search query', placeholder: 'latest AI model benchmarks 2026' }, { name: 'num_results', type: 'number', required: false, desc: 'Number of results to return (default 10)', placeholder: '10' }, { name: 'type', type: 'string', required: false, desc: '"keyword" or "neural" (default "neural")', placeholder: 'neural' }],
      mockResponse: JSON.stringify({ results: [{ url: 'https://arxiv.org/abs/2405.0001', title: 'GPT-5 Benchmark Results', score: 0.97 }, { url: 'https://huggingface.co/blog/evals', title: 'Open LLM Leaderboard 2026', score: 0.94 }] }, null, 2) },
    { name: 'exa-get-contents', desc: 'Retrieve contents of specific URLs.',
      params: [{ name: 'urls', type: 'string', required: true, desc: 'Comma-separated list of URLs to fetch', placeholder: 'https://example.com' }],
      mockResponse: JSON.stringify([{ url: 'https://example.com', text: 'Example Domain\nThis domain is for use in illustrative examples...' }], null, 2) },
    { name: 'exa-find-similar', desc: 'Find pages similar to a given URL.',
      params: [{ name: 'url', type: 'string', required: true, desc: 'URL to find similar pages for', placeholder: 'https://arxiv.org/abs/2405.0001' }, { name: 'num_results', type: 'number', required: false, desc: 'Number of results', placeholder: '5' }],
      mockResponse: JSON.stringify({ results: [{ url: 'https://arxiv.org/abs/2405.0002', title: 'Related Paper', score: 0.91 }] }, null, 2) },
  ],
};
const DEFAULT_TOOLS = [
  { name: 'list_tools', desc: 'List all available tools on this MCP server.', params: [], mockResponse: JSON.stringify({ tools: ['list_tools', 'ping', 'echo'] }, null, 2) },
  { name: 'ping', desc: 'Check connectivity to the MCP server.', params: [], mockResponse: JSON.stringify({ status: 'ok', latency_ms: 42 }, null, 2) },
  { name: 'echo', desc: 'Echo back the provided message.', params: [{ name: 'message', type: 'string', required: true, desc: 'Message to echo back', placeholder: 'Hello, world!' }], mockResponse: JSON.stringify({ echo: 'Hello, world!' }, null, 2) },
];

function escapeHtml(str) { return String(str).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;'); }
function syntaxHighlight(json) {
  if (typeof json !== 'string') json = JSON.stringify(json, null, 2);
  return escapeHtml(json).replace(
    /("(\\u[a-zA-Z0-9]{4}|\\[^u]|[^\\"])*"(\s*:)?|\b(true|false|null)\b|-?\d+(?:\.\d*)?(?:[eE][+\-]?\d+)?)/g,
    match => {
      if (/^"/.test(match)) return /:$/.test(match) ? `<span class="json-key">${match}</span>` : `<span class="json-str">${match}</span>`;
      if (/true|false/.test(match)) return `<span class="json-bool">${match}</span>`;
      if (/null/.test(match)) return `<span class="json-null">${match}</span>`;
      return `<span class="json-num">${match}</span>`;
    }
  );
}

/* ── Tool list ── */
function ToolList({ tools, activeName, onSelect, count }) {
  const [query, setQuery] = useState('');
  const filtered = query ? tools.filter(t => t.name.includes(query) || t.desc.toLowerCase().includes(query.toLowerCase())) : tools;
  return (
    <div className="tool-sidebar">
      <div className="ts-header">
        <div className="ts-label">Tools <span className="ts-count">{count}</span></div>
        <ShellSearch size="sm" value={query} onChange={setQuery} placeholder="Search tools…" />
      </div>
      <div className="ts-list">
        {filtered.map(t => (
          <div key={t.name} className={'tool-item' + (activeName === t.name ? ' active' : '')} onClick={() => onSelect(t)}>
            <div className="tool-name">{t.name}</div>
            <div className="tool-desc-short">{t.desc}</div>
          </div>
        ))}
      </div>
    </div>
  );
}

/* ── Result area ── */
function ResultArea({ result, running, resultTab, setResultTab, onCopy }) {
  return (
    <div className="result-area">
      <div className="result-tabs">
        <span className={'result-status ' + (running ? 'idle' : result ? (result.ok ? 'success' : 'error') : 'idle')}>
          {running ? <><span className="spinner"></span> Executing…</> : result ? (result.ok ? <><Ic name="check" size={11} /> Success</> : <><Ic name="x" size={11} /> Error</>) : '—'}
        </span>
        {!running && <>
          <button className={'rtab' + (resultTab === 'response' ? ' on' : '')} onClick={() => setResultTab('response')}>Response</button>
          <button className={'rtab' + (resultTab === 'raw' ? ' on' : '')} onClick={() => setResultTab('raw')}>Raw JSON</button>
          <button className={'rtab' + (resultTab === 'request' ? ' on' : '')} onClick={() => setResultTab('request')}>Request</button>
          <button className="rtab-copy" title="Copy" onClick={onCopy}><Ic name="copy" size={13} /></button>
        </>}
      </div>
      <div className="result-body">
        {running ? (
          <div className="result-running"><div className="result-running-inner"><div className="result-running-spin"></div><div style={{ fontSize: 13 }}>Running…</div></div></div>
        ) : !result ? (
          <div className="result-idle"><Ic name="play-circle" size={28} /><div className="ri-t">No result yet</div><div className="ri-s">Fill in the parameters above and click Execute</div></div>
        ) : resultTab === 'raw' ? (
          <pre className="result-code">{result.raw}</pre>
        ) : resultTab === 'request' ? (
          <pre className="result-code" dangerouslySetInnerHTML={{ __html: syntaxHighlight(JSON.stringify(result.request, null, 2)) }} />
        ) : (
          <pre className="result-code" dangerouslySetInnerHTML={{ __html: syntaxHighlight(result.response) }} />
        )}
      </div>
    </div>
  );
}

/* ── Tool detail ── */
function ToolDetail({ tool, onLog }) {
  const [tab, setTab] = useState('form');
  const [values, setValues] = useState({});
  const [jsonText, setJsonText] = useState('');
  const [result, setResult] = useState(null);
  const [running, setRunning] = useState(false);
  const [resultTab, setResultTab] = useState('response');
  const [errFields, setErrFields] = useState({});

  useEffect(() => { setTab('form'); setValues({}); setJsonText(''); setResult(null); setRunning(false); setResultTab('response'); setErrFields({}); }, [tool && tool.name]);

  if (!tool) {
    return (
      <div className="tool-detail">
        <div className="tool-empty"><Ic name="wrench" size={40} /><div className="te-t">Select a tool</div><div className="te-s">Choose a tool from the list to explore its parameters and try it out</div></div>
      </div>
    );
  }

  const setVal = (name, v) => setValues(p => ({ ...p, [name]: v }));

  const execute = () => {
    let valid = true;
    const errs = {};
    if (tab === 'form') {
      tool.params.forEach(p => { if (p.required && !(values[p.name] || '').trim()) { valid = false; errs[p.name] = true; } });
    }
    if (!valid) { setErrFields(errs); setTimeout(() => setErrFields({}), 2000); onLog('error', 'Missing required parameters'); return; }

    const reqParams = {};
    tool.params.forEach(p => { if (values[p.name]) reqParams[p.name] = values[p.name]; });
    const request = { tool: tool.name, params: reqParams };
    onLog('info', `Executing ${tool.name}…`);
    setRunning(true);
    setTimeout(() => {
      setResult({ ok: true, response: tool.mockResponse, raw: tool.mockResponse, request });
      setRunning(false);
      setResultTab('response');
      onLog('success', `${tool.name} executed · 200 OK · 214ms`);
    }, 650 + Math.random() * 400);
  };

  const clearForm = () => { setValues({}); setJsonText(''); };
  const copyResult = () => { if (result) { navigator.clipboard?.writeText(result.raw); onLog('info', 'Result copied to clipboard'); } };

  const jsonPlaceholder = (() => {
    const obj = {};
    tool.params.forEach(p => { if (p.required) obj[p.name] = p.type === 'number' ? 0 : ''; });
    return JSON.stringify(obj, null, 2);
  })();

  return (
    <div className="tool-detail">
      <div className="td-inner">
        <div className="td-head">
          <div className="td-tool-name">{tool.name}</div>
          <div className="td-tool-desc">{tool.desc}</div>
        </div>
        <div className="td-tabs">
          <button className={'tdtab' + (tab === 'form' ? ' on' : '')} onClick={() => setTab('form')}>Form</button>
          <button className={'tdtab' + (tab === 'json' ? ' on' : '')} onClick={() => setTab('json')}>JSON</button>
        </div>
        <div className="td-body">
          {tab === 'form' ? (
            <div className="form-tab">
              {tool.params.length === 0 ? (
                <div style={{ fontSize: 13, color: 'hsl(var(--muted-foreground))', padding: '8px 0', marginBottom: 16 }}>This tool takes no parameters.</div>
              ) : tool.params.map(p => (
                <div className="form-field" key={p.name}>
                  <div className="form-label">{p.name} {p.required && <span className="req">*</span>}<span className="type-badge">{p.type}</span></div>
                  <input className="form-input" placeholder={p.placeholder || ''} value={values[p.name] || ''}
                    style={errFields[p.name] ? { borderColor: 'hsl(var(--destructive))' } : null}
                    onChange={e => setVal(p.name, e.target.value)} />
                  <div className="form-hint">{p.desc}</div>
                </div>
              ))}
              <div className="execute-row">
                <button className="btn-execute" onClick={execute}><Ic name="play" size={14} /> Execute</button>
                {tool.params.length > 0 && <button className="btn-clear" onClick={clearForm}>Clear</button>}
              </div>
            </div>
          ) : (
            <div className="json-tab">
              <textarea placeholder={jsonPlaceholder} value={jsonText} onChange={e => setJsonText(e.target.value)} />
              <div className="execute-row">
                <button className="btn-execute" onClick={execute}><Ic name="play" size={14} /> Execute</button>
                <button className="btn-clear" onClick={clearForm}>Clear</button>
              </div>
            </div>
          )}
        </div>
        <ResultArea result={result} running={running} resultTab={resultTab} setResultTab={setResultTab} onCopy={copyResult} />
      </div>
    </div>
  );
}

/* ── Execution log ── */
function ExecLog({ open, lines, onClose }) {
  const bodyRef = useRef(null);
  useEffect(() => { if (bodyRef.current) bodyRef.current.scrollTop = bodyRef.current.scrollHeight; }, [lines]);
  if (!open) return null;
  return (
    <div className="exec-log">
      <div className="exec-log-head">Execution Log<button onClick={onClose}><Ic name="x" size={13} /></button></div>
      <div className="exec-log-body" ref={bodyRef}>
        {lines.map((l, i) => <div key={i} className={'log-line ' + l.type}>{l.ts}  {l.msg}</div>)}
      </div>
    </div>
  );
}

/* ── Root ── */
function PlaygroundApp() {
  const params = useMemo(() => new URLSearchParams(window.location.search), []);
  const instanceName = params.get('name') || 'my-instance';
  const serverId = params.get('server') || 'deepwiki';
  const tools = SERVER_TOOLS[serverId] || DEFAULT_TOOLS;

  const [status, setStatus] = useState('connecting');
  const [activeTool, setActiveTool] = useState(null);
  const [logOpen, setLogOpen] = useState(false);
  const [logLines, setLogLines] = useState([]);

  const addLog = (type, msg) => {
    const ts = new Date().toLocaleTimeString('en-US', { hour12: false });
    setLogLines(prev => [...prev, { type, msg, ts }]);
  };

  useEffect(() => { document.title = `Bodhi · ${instanceName} Playground`; }, [instanceName]);

  useEffect(() => {
    addLog('info', `Connecting to ${serverId} MCP server…`);
    const t = setTimeout(() => {
      setStatus('connected');
      addLog('success', `Connected · ${tools.length} tools available`);
      addLog('info', `Instance: ${instanceName}`);
      if (tools.length) setActiveTool(tools[0]);
    }, 900);
    return () => clearTimeout(t);
  }, []);

  const reconnect = () => {
    setStatus('connecting');
    addLog('info', 'Reconnecting…');
    setTimeout(() => { setStatus('connected'); addLog('success', 'Reconnected successfully'); }, 1100);
  };

  const headerActions = (
    <>
      <div className="pg-title-wrap" style={{ display: 'flex', alignItems: 'center', gap: 8, marginRight: 'auto', minWidth: 0 }}>
        <span style={{ fontSize: 14, fontWeight: 700, letterSpacing: '-.02em', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{instanceName} — Playground</span>
        {status === 'connected'
          ? <span className="status-pill connected"><Ic name="circle-check" size={10} /> Connected</span>
          : <span className="status-pill connecting"><span className="spinner"></span> Connecting…</span>}
      </div>
      <button className="icon-btn" title="Reconnect" onClick={reconnect}><Ic name="refresh-cw" size={14} /></button>
      <button className="icon-btn" title="Settings"><Ic name="settings-2" size={14} /></button>
      <button className="icon-btn" title="Execution log" onClick={() => setLogOpen(o => !o)}><Ic name="terminal" size={14} /></button>
    </>
  );

  return (
    <>
      <AppShell
        section="mcp" subPage="my-instances" resizeKey="mcp"
        breadcrumb={[
          { label: 'MCP', href: 'Bodhi MCP Discover v2.html' },
          { label: 'My Instances', href: 'Bodhi MCP Discover v2.html' },
          { label: 'Playground', current: true },
        ]}
        headerActions={headerActions}
        contentClass="flush" mainScroll={false}
      >
        <div className="pg-grid">
          <ToolList tools={tools} activeName={activeTool && activeTool.name} count={status === 'connected' ? tools.length : ''} onSelect={setActiveTool} />
          <ToolDetail tool={status === 'connected' ? activeTool : null} onLog={addLog} />
        </div>
      </AppShell>
      <ExecLog open={logOpen} lines={logLines} onClose={() => setLogOpen(false)} />
    </>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<PlaygroundApp />);
