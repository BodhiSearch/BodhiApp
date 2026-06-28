/* ═══════════════════════════════════════════════════════════════
   Bodhi Chat — page app (React, on the shared AppShell)
   chat/bodhi-chat-app.jsx   (load after the shell modules)

   Same layout system as every other page: <AppShell> provides the
   sidebar nav, breadcrumb + collapse toggle, hover-resize, the rail
   and responsive drawers. This file supplies the chat-specific slots:
     • sidebar  → chat history
     • main     → conversation + composer
     • railHeader → the two rail tabs (Parameters / MCP servers)
     • rail     → the active tab's pane
═══════════════════════════════════════════════════════════════ */
const { useState, useEffect, useRef } = React;
const Ic = ShellIcon;

/* ── Catalogs (stand-ins for backend data) ───────────────────── */
const MODEL_GROUPS = [
  { label: 'Local', models: ['llama-3.1-8b-instruct', 'qwen2.5-7b-instruct', 'phi-3.5-mini-instruct'] },
  { label: 'API',   models: ['claude-sonnet-4.5', 'gpt-4o', 'gemini-2.0-flash'] },
];

/* Every MCP server configured in this workspace, with the tools it exposes.
   The chat only lists the ones added to THIS conversation; the rest are
   offered in the "add server" combobox. */
const MCP_CATALOG = [
  { id: 'filesystem', name: 'filesystem', status: 'connected',  tools: ['read_file', 'write_file', 'list_directory', 'search_files', 'get_file_info', 'move_file', 'create_directory', 'delete_file'] },
  { id: 'git',        name: 'git',        status: 'connected',  tools: ['status', 'log', 'diff', 'commit', 'branch', 'checkout', 'stash', 'show'] },
  { id: 'postgres',   name: 'postgres',   status: 'connecting', tools: ['query', 'list_tables', 'describe_table', 'explain'] },
  { id: 'notion',     name: 'Notion',     status: 'connected',  tools: ['search', 'get_page', 'create_page', 'update_page', 'query_database'] },
  { id: 'github',     name: 'GitHub',     status: 'connected',  tools: ['create_issue', 'list_repos', 'get_pr', 'create_pr', 'search_code', 'list_commits'] },
  { id: 'exa',        name: 'Exa Search', status: 'connected',  tools: ['web_search', 'find_similar', 'get_contents'] },
];

/* Build a server instance (tools as {name,on}) with a set of enabled tools. */
function makeServer(id, enabledTools) {
  const def = MCP_CATALOG.find(s => s.id === id);
  const on = new Set(enabledTools || def.tools);
  return { id: def.id, name: def.name, status: def.status, expanded: false,
           tools: def.tools.map(name => ({ name, on: on.has(name) })) };
}

/* ── Small reusable controls ─────────────────────────────────── */
function Switch({ on, onToggle }) {
  return <button className={'sw' + (on ? ' on' : '')} onClick={onToggle} aria-pressed={on} />;
}

/* Model picker — free-text autocomplete (any value allowed; the list is
   just suggestions, not a constraint). */
function ModelCombo({ value, onChange }) {
  const [open, setOpen] = useState(false);
  const ref = useRef(null);
  useEffect(() => {
    if (!open) return;
    const h = e => { if (ref.current && !ref.current.contains(e.target)) setOpen(false); };
    document.addEventListener('mousedown', h);
    return () => document.removeEventListener('mousedown', h);
  }, [open]);
  const q = (value || '').toLowerCase().trim();
  const all = MODEL_GROUPS.flatMap(g => g.models);
  const isMatch = m => m.toLowerCase().includes(q);
  const byName = (a, b) => a.localeCompare(b);
  // matching first (the current model floated to the top), then the rest A→Z
  const matching = all.filter(isMatch).sort((a, b) => (a === value ? -1 : b === value ? 1 : byName(a, b)));
  const nonMatching = all.filter(m => !isMatch(m)).sort(byName);
  const opt = m => (
    <button key={m} className={'rail-pop-opt' + (m === value ? ' sel' : '')}
            onMouseDown={e => e.preventDefault()} onClick={() => { onChange(m); setOpen(false); }}>
      <span className="rail-pop-opt-name mono">{m}</span>
      {m === value && <Ic name="check" size={14} />}
    </button>
  );
  return (
    <div className={'rail-combo' + (open ? ' open' : '')} ref={ref}>
      <input className="rail-input" type="text" value={value} placeholder="Search or type a model name…"
             spellCheck={false} autoComplete="off"
             onChange={e => { onChange(e.target.value); setOpen(true); }}
             onFocus={() => setOpen(true)}
             onKeyDown={e => { if (e.key === 'Escape') { setOpen(false); e.currentTarget.blur(); } }} />
      <span className="rail-combo-caret"><Ic name="chevron-down" size={13} /></span>
      {open && (
        <div className="rail-pop">
          {matching.map(opt)}
          {matching.length > 0 && nonMatching.length > 0 && <div className="rail-pop-div" />}
          {nonMatching.map(opt)}
        </div>
      )}
    </div>
  );
}

/* MCP add — a combobox with a search bar that filters configured servers. */
function McpAddCombo({ available, onAdd }) {
  const [open, setOpen] = useState(false);
  const [q, setQ] = useState('');
  const ref = useRef(null);
  useEffect(() => {
    if (!open) return;
    const h = e => { if (ref.current && !ref.current.contains(e.target)) { setOpen(false); setQ(''); } };
    document.addEventListener('mousedown', h);
    return () => document.removeEventListener('mousedown', h);
  }, [open]);
  const filtered = available.filter(s => s.name.toLowerCase().includes(q.toLowerCase()));
  return (
    <div className={'rail-combo' + (open ? ' open' : '')} ref={ref}>
      <button className="mcp-add-trigger" onClick={() => setOpen(o => !o)}>
        <Ic name="plus" size={14} />
        <span className="lbl">Add an MCP server…</span>
        <span className="caret"><Ic name="chevron-down" size={14} /></span>
      </button>
      {open && (
        <div className="rail-pop">
          <div className="rail-pop-search">
            <Ic name="search" size={13} />
            <input autoFocus type="text" value={q} placeholder="Search servers…"
                   spellCheck={false} onChange={e => setQ(e.target.value)} />
          </div>
          {filtered.map(s => (
            <button key={s.id} className="rail-pop-opt"
                    onMouseDown={e => e.preventDefault()} onClick={() => { onAdd(s.id); setOpen(false); setQ(''); }}>
              <span className={'mcp-dot ' + s.status} />
              <span className="rail-pop-opt-name">{s.name}</span>
              <span className="rail-pop-opt-meta">{s.tools.length} tools</span>
            </button>
          ))}
          {filtered.length === 0 && <div className="rail-pop-empty">No servers found</div>}
        </div>
      )}
    </div>
  );
}

/* A single setting row: label + help + (value) + an override switch, with the
   control below. Every setting can be switched OFF to skip applying it; the
   control stays visible but muted/inert while off (matches the app's panel). */
function Setting({ name, help, defaultOn, value, children }) {
  const [on, setOn] = useState(!!defaultOn);
  return (
    <div className={'setting' + (on ? ' on' : '')}>
      <div className="setting-head">
        <span className="setting-name">{name}{help && <span className="setting-help"><Ic name="help-circle" size={12} /></span>}</span>
        <div className="setting-right">
          {value != null && <span className="setting-val">{value}</span>}
          <Switch on={on} onToggle={() => setOn(v => !v)} />
        </div>
      </div>
      {children && <div className="setting-control">{children}</div>}
    </div>
  );
}

function Slider({ pct }) {
  return (
    <div className="sl-track-wrap">
      <div className="sl-track"><div className="sl-fill" style={{ width: pct + '%' }} /><div className="sl-thumb" style={{ left: pct + '%' }} /></div>
    </div>
  );
}

/* ── Sidebar: chat history (collapse-aware) ──────────────────── */
const CHAT_GROUPS = [
  { group: 'Today', items: ['MCP tool calling basics', 'Quantization tradeoffs for 8B'] },
  { group: 'Yesterday', items: ['Debugging the filesystem server', 'Draft release notes · v0.8'] },
  { group: 'Previous 7 days', items: ['How does top-p sampling work?', 'Rate limiting for the proxy endpoint', 'Exploring Claude Haiku 4.5 in depth', 'OpenAI Responses API migration'] },
];

function ChatSidebar() {
  const { collapsed, openPop, setOpenPop } = useShell();
  const [searchOpen, setSearchOpen] = useState(false);
  const [menuOpen, setMenuOpen] = useState(null);
  const [active, setActive] = useState('MCP tool calling basics');
  const histRef = useRef(null);
  const histOpen = openPop === 'chat-history';

  useEffect(() => {
    if (menuOpen === null) return;
    const h = () => setMenuOpen(null);
    document.addEventListener('click', h);
    return () => document.removeEventListener('click', h);
  }, [menuOpen]);

  if (collapsed) {
    return (<>
      <button className="shell-railbtn shell-tip" data-tip="New chat"><Ic name="square-pen" size={18} /></button>
      <button ref={histRef} className={'shell-railbtn shell-tip' + (histOpen ? ' on' : '')} data-tip="Chat history"
              onClick={e => { e.stopPropagation(); setOpenPop(histOpen ? null : 'chat-history'); }}>
        <Ic name="history" size={18} />
      </button>
      <AnchoredPopover open={histOpen} anchorRef={histRef} onClose={() => setOpenPop(null)}>
        <div className="shell-pop-title">Chat history</div>
        <div className="hist-pop">
          {CHAT_GROUPS.map(g => (
            <React.Fragment key={g.group}>
              <div className="hist-pop-group">{g.group}</div>
              {g.items.map(label => (
                <button key={label} className={'hist-pop-item' + (active === label ? ' on' : '')}
                        onClick={() => { setActive(label); setOpenPop(null); }}>
                  <Ic name="message-circle" size={13} />
                  <span className="label">{label}</span>
                </button>
              ))}
            </React.Fragment>
          ))}
        </div>
      </AnchoredPopover>
    </>);
  }

  return (
    <div className="chat-hist">
      <button className="new-chat"><Ic name="plus" size={14} />New chat</button>
      <div className="hist-head">
        <span className="t">History</span>
        <button onClick={() => setSearchOpen(o => !o)} title="Search chats"><Ic name="search" size={13} /></button>
      </div>
      <div className={'hist-search' + (searchOpen ? ' open' : '')}>
        <input type="text" placeholder="Search conversations…" />
      </div>
      <div className="chats">
        {CHAT_GROUPS.map(g => (
          <React.Fragment key={g.group}>
            <div className="chat-group">{g.group}</div>
            {g.items.map(label => {
              const id = label;
              return (
                <div key={id} className={'chat-item' + (active === id ? ' on' : '')} onClick={() => setActive(id)}>
                  <span className="label">{label}</span>
                  <button className="more" onClick={e => { e.stopPropagation(); setMenuOpen(menuOpen === id ? null : id); }}>
                    <Ic name="more-horizontal" size={13} />
                  </button>
                  {menuOpen === id && (
                    <div className="ctx-menu" onClick={e => e.stopPropagation()}>
                      <div className="ci"><Ic name="edit-3" size={13} />Rename</div>
                      <div className="ci"><Ic name="pin" size={13} />Pin</div>
                      <div className="ci"><Ic name="copy" size={13} />Duplicate</div>
                      <div className="ci"><Ic name="download" size={13} />Export</div>
                      <div className="sep" />
                      <div className="ci danger"><Ic name="trash-2" size={13} />Delete</div>
                    </div>
                  )}
                </div>
              );
            })}
          </React.Fragment>
        ))}
      </div>
    </div>
  );
}

/* ── Header breadcrumb (no header actions — Share/Export/⋯ removed) ── */
function ChatTitle() {
  return (
    <div className="chat-title">
      <div className="crumb"><Ic name="message-circle" size={10} />Chat<Ic name="chevron-right" size={10} /><span>Today, 11:24</span></div>
      <div className="title"><span className="name">MCP tool calling basics</span><span className="edit-icon"><Ic name="edit-3" size={12} /></span></div>
    </div>
  );
}

/* ── Main: conversation + composer ───────────────────────────── */
function MetaActs() {
  return (
    <div className="meta-acts">
      <button title="Copy"><Ic name="copy" size={13} /></button>
      <button title="Regenerate"><Ic name="refresh-cw" size={13} /></button>
      <button title="Branch from here"><Ic name="git-branch" size={13} /></button>
      <button title="Good response"><Ic name="thumbs-up" size={13} /></button>
    </div>
  );
}

function ChatMain() {
  const taRef = useRef(null);
  const onKey = e => {
    if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') { e.preventDefault(); const ta = e.target; if (ta.value.trim()) { ta.value = ''; ta.style.height = ''; } }
  };
  const onInput = e => { const t = e.target; t.style.height = 'auto'; t.style.height = Math.min(t.scrollHeight, 160) + 'px'; };

  return (
    <div className="chat-main">
      <div className="conv">
        <div className="conv-inner">

          <div className="msg">
            <div className="m-avatar av-user">YO</div>
            <div className="bubble">
              <div className="nm">You <span className="msg-time">11:24</span></div>
              <div className="body"><div className="user-body">Explain how tool calls work with the <span className="ic">filesystem</span> MCP server, and show me a concrete read example.</div></div>
            </div>
          </div>

          <div className="msg">
            <div className="m-avatar av-ai"><Ic name="sparkles" size={14} /></div>
            <div className="bubble" tabIndex={0}>
              <div className="nm">Bodhi <span className="model-tag">llama-3.1-8b-instruct</span><span className="msg-time">11:24 · streamed</span></div>
              <div className="body">
                <p>When the model calls a tool via the <span className="ic">filesystem</span> MCP server, the flow is:</p>
                <p>(1) the model emits a structured tool request, (2) Bodhi validates the call against the server's declared schema and your permission grants, (3) the call executes locally, and (4) the result is streamed back into the model's context for the next turn.</p>
                <div className="tool-call">
                  <div className="tc-h"><Ic name="plug" size={12} />filesystem.read_file<span className="ok">✓ 38ms</span></div>
                  <code>{'{ path: "~/notes/mcp.md" } → 2.1 KB'}</code>
                </div>
                <p style={{ marginTop: 10 }}>Here's what the raw tool-call JSON looks like on the wire:</p>
                <div className="code-block" dangerouslySetInnerHTML={{ __html:
                  `<span class="lang">JSON</span><span class="cm">// request from model → bodhi</span>\n{ <span class="kw">"tool"</span>: <span class="str">"filesystem.read_file"</span>,\n  <span class="kw">"arguments"</span>: { <span class="kw">"path"</span>: <span class="str">"~/notes/mcp.md"</span>, <span class="kw">"encoding"</span>: <span class="str">"utf-8"</span> } }` }} />
                <p style={{ marginTop: 10 }}>You can cap chained tool calls with <span className="ic">max_tool_iterations</span>.</p>
              </div>
              <div className="meta-strip">
                <span className="mi"><Ic name="arrow-down-to-line" size={11} /><b>842</b>&thinsp;in</span>
                <span className="mi"><Ic name="arrow-up-from-line" size={11} /><b>316</b>&thinsp;out</span>
                <span className="mi"><Ic name="zap" size={11} /><b>47</b>&thinsp;t/s</span>
                <span className="mi"><Ic name="clock" size={11} /><b>6.7s</b></span>
                <span className="mi local"><Ic name="circle-dollar-sign" size={11} /><b>local</b></span>
                <span className="meta-spacer" />
                <MetaActs />
              </div>
            </div>
          </div>

          <div className="msg">
            <div className="m-avatar av-user">YO</div>
            <div className="bubble">
              <div className="nm">You <span className="msg-time">11:28</span></div>
              <div className="body"><div className="user-body">Now switch to Claude Sonnet and compare its reasoning.</div></div>
            </div>
          </div>

          <div className="msg">
            <div className="m-avatar av-ai"><Ic name="sparkles" size={14} /></div>
            <div className="bubble" tabIndex={0}>
              <div className="nm">Bodhi <span className="model-tag api">claude-sonnet-4.5</span><span className="msg-time">11:29 · via API</span></div>
              <div className="body">
                <p>Switched. Compared to <span className="ic">llama-3.1-8b</span>, Sonnet reasons about tool selection more aggressively — it's more willing to chain calls and self-correct mid-turn, which is why tool-iteration budgets matter more on capable models.</p>
                <p>Two practical differences: (a) Sonnet rarely needs the explicit schema reminder in the system prompt, (b) it emits tool arguments with tighter types, so downstream validation fails less often.</p>
              </div>
              <div className="meta-strip">
                <span className="mi"><Ic name="arrow-down-to-line" size={11} /><b>1.2k</b>&thinsp;in</span>
                <span className="mi"><Ic name="arrow-up-from-line" size={11} /><b>287</b>&thinsp;out</span>
                <span className="mi"><Ic name="zap" size={11} /><b>68</b>&thinsp;t/s</span>
                <span className="mi"><Ic name="clock" size={11} /><b>4.2s</b></span>
                <span className="mi api-cost"><Ic name="circle-dollar-sign" size={11} /><b>$0.006</b></span>
                <span className="meta-spacer" />
                <MetaActs />
              </div>
            </div>
          </div>

        </div>
      </div>

      <div className="composer">
        <div className="composer-inner">
          <textarea ref={taRef} placeholder="Reply to Bodhi…   ⌘↵ to send" rows={2} onKeyDown={onKey} onInput={onInput} />
          <div className="composer-row">
            <span className="ctx-indicator" title="Context usage · 1.2k of 8k tokens">
              <svg className="ctx-ring" viewBox="0 0 15 15"><circle className="bg" cx="7.5" cy="7.5" r="5.5" /><circle className="fg" cx="7.5" cy="7.5" r="5.5" strokeDasharray="34.56" strokeDashoffset="29.4" /></svg>
              <b>15%</b><span className="ctx-sub">1.2k / 8k</span>
            </span>
            <button className="comp-icon" title="Attach file"><Ic name="paperclip" size={15} /></button>
            <button className="send"><span className="send-lbl">Send</span><Ic name="arrow-up" size={13} /></button>
          </div>
        </div>
      </div>
    </div>
  );
}

/* ── Rail header: the two tabs (sit on the shared 56px gridline) ── */
function RailTabs({ tab, setTab, mcpCount }) {
  return (
    <div className="rail-tabs">
      <button className={'rail-tab' + (tab === 'parameters' ? ' active' : '')} onClick={() => setTab('parameters')}>
        <Ic name="sliders-horizontal" size={13} />Parameters
      </button>
      <button className={'rail-tab' + (tab === 'mcp' ? ' active' : '')} onClick={() => setTab('mcp')}>
        <Ic name="plug" size={13} />MCP servers<span className="rt-badge">{mcpCount}</span>
      </button>
    </div>
  );
}

/* ── Rail pane: Parameters ───────────────────────────────────── */
function ParametersPane({ model, setModel }) {
  return (
    <div className="rail-pane">
      <div className="model-field">
        <div className="setting-head">
          <span className="setting-name">Model<span className="setting-help"><Ic name="help-circle" size={12} /></span></span>
        </div>
        <div className="setting-control"><ModelCombo value={model} onChange={setModel} /></div>
      </div>

      <Setting name="Stream response" help defaultOn />

      <Setting name="API token" help>
        <input className="rail-input" type="text" placeholder="Enter your API token" autoComplete="off" spellCheck={false} />
      </Setting>

      <Setting name="Max tool iterations" help defaultOn>
        <input className="rail-input mono" type="number" defaultValue={5} min={1} />
      </Setting>

      <Setting name="Seed" help>
        <input className="rail-input mono" type="number" placeholder="Random" />
      </Setting>

      <Setting name="System prompt" help>
        <textarea className="rail-textarea" placeholder="Enter system prompt…" rows={3} />
      </Setting>

      <Setting name="Stop words" help>
        <input className="rail-input" type="text" placeholder="Type and press Enter to add…" />
      </Setting>

      <div className="rail-div" />

      <Setting name="Temperature" help value="1"><Slider pct={50} /></Setting>
      <Setting name="Top P" help value="1"><Slider pct={100} /></Setting>

      <div className="rail-div" />

      <Setting name="Max tokens" help value="2048"><Slider pct={100} /></Setting>
      <Setting name="Presence penalty" help value="0"><Slider pct={50} /></Setting>
      <Setting name="Frequency penalty" help value="0"><Slider pct={50} /></Setting>

      <button className="reset-defaults"><Ic name="rotate-ccw" size={12} />Reset to defaults</button>
    </div>
  );
}

/* ── Rail pane: MCP servers (combobox to add · accordion tool picker) ── */
function McpServerRow({ srv, onToggleExpand, onToggleTool, onSelectAll, onRemove }) {
  const onCount = srv.tools.filter(t => t.on).length;
  return (
    <div className={'mcp-srv' + (srv.expanded ? ' open' : '')}>
      <div className="mcp-srv-head" onClick={onToggleExpand}>
        <span className={'mcp-dot ' + srv.status} title={srv.status === 'connected' ? 'Connected' : 'Connecting…'} />
        <span className="mcp-srv-name">{srv.name}</span>
        <span className="mcp-srv-count">{onCount}/{srv.tools.length}</span>
        <span className="mcp-chev"><Ic name="chevron-down" size={13} /></span>
        <button className="mcp-trash" title="Remove from chat" onClick={e => { e.stopPropagation(); onRemove(); }}><Ic name="trash-2" size={13} /></button>
      </div>
      <div className="mcp-srv-tools">
        <div className="mcp-tool-quick">
          <span>{onCount} of {srv.tools.length} tools</span>
          <div className="mcp-quick-links">
            <button onClick={() => onSelectAll(true)}>All</button>
            <span className="sepdot">·</span>
            <button onClick={() => onSelectAll(false)}>None</button>
          </div>
        </div>
        {srv.tools.map(t => (
          <label className="mcp-tool" key={t.name}>
            <input type="checkbox" checked={t.on} onChange={() => onToggleTool(t.name)} />
            <span className="mcp-tool-name">{t.name}</span>
          </label>
        ))}
      </div>
    </div>
  );
}

function McpServersPane({ servers, setServers }) {
  const added = new Set(servers.map(s => s.id));
  const available = MCP_CATALOG.filter(s => !added.has(s.id));

  const addServer = id => { if (id) setServers(list => [...list, makeServer(id)]); };
  const removeServer = id => setServers(list => list.filter(s => s.id !== id));
  const toggleExpand = id => setServers(list => list.map(s => s.id === id ? { ...s, expanded: !s.expanded } : s));
  const toggleTool = (id, name) => setServers(list => list.map(s => s.id !== id ? s : { ...s, tools: s.tools.map(t => t.name === name ? { ...t, on: !t.on } : t) }));
  const selectAll = (id, on) => setServers(list => list.map(s => s.id !== id ? s : { ...s, tools: s.tools.map(t => ({ ...t, on })) }));

  return (
    <div className="rail-pane">
      <div className="mcp-add">
        {available.length ? (
          <McpAddCombo available={available} onAdd={addServer} />
        ) : (
          <div className="mcp-add-done">All configured servers added</div>
        )}
      </div>

      {servers.length === 0 ? (
        <div className="mcp-empty">
          <Ic name="plug" size={20} />
          <div className="mcp-empty-t">No MCP servers in this chat</div>
          <div className="mcp-empty-s">Add one above to let the model call its tools.</div>
        </div>
      ) : (
        <div className="mcp-list">
          {servers.map(srv => (
            <McpServerRow key={srv.id} srv={srv}
              onToggleExpand={() => toggleExpand(srv.id)}
              onToggleTool={name => toggleTool(srv.id, name)}
              onSelectAll={on => selectAll(srv.id, on)}
              onRemove={() => removeServer(srv.id)} />
          ))}
        </div>
      )}
    </div>
  );
}

/* ── Root ────────────────────────────────────────────────────── */
function ChatApp() {
  const [tab, setTab] = useState('parameters');
  const [model, setModel] = useState('llama-3.1-8b-instruct');
  const [servers, setServers] = useState(() => [
    makeServer('filesystem', ['read_file', 'list_directory', 'search_files', 'get_file_info']),
    makeServer('git', ['status', 'log', 'diff']),
  ]);

  return (
    <AppShell
      section="chat" resizeKey="chat"
      sidebarWidth={260} railWidth={360}
      breadcrumb={<ChatTitle />}
      sidebar={<ChatSidebar />}
      contentClass="flush" mainScroll={false} railScroll={false}
      railHeader={<RailTabs tab={tab} setTab={setTab} mcpCount={servers.length} />}
      rail={
        tab === 'parameters'
          ? <ParametersPane model={model} setModel={setModel} />
          : <McpServersPane servers={servers} setServers={setServers} />
      }
    >
      <ChatMain />
    </AppShell>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<ChatApp />);
