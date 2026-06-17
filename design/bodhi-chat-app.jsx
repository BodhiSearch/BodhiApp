/* ═══════════════════════════════════════════════════════════════
   Bodhi Chat — page app (React, on the shared AppShell)
   bodhi-chat-app.jsx   (load after bodhi-app-shell.jsx)

   Same layout system as every other page: <AppShell> provides the
   sidebar nav, breadcrumb + collapse toggle, hover-resize, the rail
   and responsive drawers. This file only supplies the chat-specific
   slots: history (sidebar), conversation + composer (main), and the
   parameters / connectors / artifacts panel (rail).
═══════════════════════════════════════════════════════════════ */
const { useState, useEffect, useRef } = React;
const Ic = ShellIcon;

/* ── Small reusable controls ─────────────────────────────────── */
function Switch({ on, onToggle }) {
  return <button className={'sw' + (on ? ' on' : '')} onClick={onToggle} />;
}

function Collapsible({ title, count, defaultOpen, edited, children }) {
  const [open, setOpen] = useState(!!defaultOpen);
  return (
    <div className={'collapsible' + (open ? ' open' : '') + (edited ? ' edited' : '')}>
      <div className="coll-head" onClick={() => setOpen(o => !o)}>
        <span className="title">{title}</span>
        <span className="count">{count}</span>
        <span className="chev"><Ic name="chevron-down" size={13} /></span>
      </div>
      <div className="coll-body">{children}</div>
    </div>
  );
}

function Param({ name, help, defaultOn, children }) {
  const [on, setOn] = useState(!!defaultOn);
  return (
    <div className={'param' + (on ? ' enabled' : '')}>
      <div className="param-head">
        <span className="pname">{name}{help && <Ic name="help-circle" size={11} />}</span>
        <Switch on={on} onToggle={() => setOn(v => !v)} />
      </div>
      {children && <div className="param-body">{children}</div>}
    </div>
  );
}

function Slider({ range, value, pct }) {
  return (<>
    <div className="slider-head"><span>{range}</span><span className="v">{value}</span></div>
    <div className="track-wrap"><div className="track"><div className="fill" style={{ width: pct + '%' }} /><div className="thumb" style={{ left: pct + '%' }} /></div></div>
  </>);
}

function Conn({ ico, cls, name, desc, dot, defaultOn }) {
  const [on, setOn] = useState(!!defaultOn);
  return (
    <div className={'conn' + (on ? ' on' : '')}>
      <div className={'conn-ico ' + cls}><Ic name={ico} size={13} /></div>
      <div className="conn-info">
        <div className="conn-name">{dot && <span className="dot" style={dot === true ? null : { background: dot }} />}{name}</div>
        <div className="conn-desc">{desc}</div>
      </div>
      <Switch on={on} onToggle={() => setOn(v => !v)} />
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
                      <div className="ci"><Ic name="share-2" size={13} />Share</div>
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

/* ── Header slots ────────────────────────────────────────────── */
function ChatTitle() {
  return (
    <div className="chat-title">
      <div className="crumb"><Ic name="message-circle" size={10} />Chat<Ic name="chevron-right" size={10} /><span>Today, 11:24</span></div>
      <div className="title"><span className="name">MCP tool calling basics</span><span className="edit-icon"><Ic name="edit-3" size={12} /></span></div>
    </div>
  );
}

function HeaderActions() {
  return (<>
    <button className="head-btn"><Ic name="share-2" size={13} />Share</button>
    <button className="head-btn"><Ic name="download" size={13} />Export</button>
    <button className="cicon-btn"><Ic name="more-horizontal" size={15} /></button>
  </>);
}

/* ── Main: conversation + composer ───────────────────────────── */
function MetaActs() {
  return (
    <div className="meta-acts">
      <button title="Copy"><Ic name="copy" size={13} /></button>
      <button title="Regenerate"><Ic name="refresh-cw" size={13} /></button>
      <button title="Branch"><Ic name="git-branch" size={13} /></button>
      <button title="Good"><Ic name="thumbs-up" size={13} /></button>
    </div>
  );
}

function ChatMain({ compact, alwaysMeta, onOpenTab }) {
  const { openRail } = useShell();
  const taRef = useRef(null);
  const openTab = name => { onOpenTab(name); openRail(); };
  const onKey = e => {
    if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') { e.preventDefault(); const ta = e.target; if (ta.value.trim()) { ta.value = ''; ta.style.height = ''; } }
  };
  const onInput = e => { const t = e.target; t.style.height = 'auto'; t.style.height = Math.min(t.scrollHeight, 160) + 'px'; };
  const metaCls = 'meta-strip' + (alwaysMeta ? ' always' : '');

  return (
    <div className="chat-main">
      <div className="conv">
        <div className="conv-inner" style={compact ? { gap: 14 } : null}>

          <div className="msg" style={compact ? { gap: 8 } : null}>
            <div className="m-avatar av-user">YO</div>
            <div className="bubble">
              <div className="nm">You <span className="msg-time">11:24</span></div>
              <div className="body"><div className="user-body">Explain how tool calls work with the <span className="ic">filesystem</span> MCP server, and show me a concrete read example.</div></div>
            </div>
          </div>

          <div className="msg" style={compact ? { gap: 8 } : null}>
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
              <div className={metaCls}>
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

          <div className="msg" style={compact ? { gap: 8 } : null}>
            <div className="m-avatar av-user">YO</div>
            <div className="bubble">
              <div className="nm">You <span className="msg-time">11:28</span></div>
              <div className="body"><div className="user-body">Now switch to Claude Sonnet and compare its reasoning.</div></div>
            </div>
          </div>

          <div className="msg" style={compact ? { gap: 8 } : null}>
            <div className="m-avatar av-ai"><Ic name="sparkles" size={14} /></div>
            <div className="bubble" tabIndex={0}>
              <div className="nm">Bodhi <span className="model-tag api">claude-sonnet-4.5</span><span className="msg-time">11:29 · via API</span></div>
              <div className="body">
                <p>Switched. Compared to <span className="ic">llama-3.1-8b</span>, Sonnet reasons about tool selection more aggressively — it's more willing to chain calls and self-correct mid-turn, which is why tool-iteration budgets matter more on capable models.</p>
                <p>Two practical differences: (a) Sonnet rarely needs the explicit schema reminder in the system prompt, (b) it emits tool arguments with tighter types, so downstream validation fails less often.</p>
              </div>
              <div className={metaCls}>
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
            <button className="comp-chip" title="Switch model"><span className="dot" /><span className="kw">llama-3.1-8b</span><Ic name="chevron-down" size={13} /></button>
            <button className="comp-chip" title="Context usage: 15%">
              <svg className="ctx-ring" viewBox="0 0 15 15"><circle className="bg" cx="7.5" cy="7.5" r="5.5" /><circle className="fg" cx="7.5" cy="7.5" r="5.5" strokeDasharray="34.56" strokeDashoffset="29.4" /></svg>
              <b>15%</b><span style={{ color: 'hsl(var(--muted-foreground))' }}>· 1.2k / 8k</span>
            </button>
            <button className="comp-chip" title="Active connectors" onClick={() => openTab('connectors')}><Ic name="plug" size={13} /><b>4</b>&nbsp;tools</button>
            <button className="comp-chip" title="Artifacts" onClick={() => openTab('artifacts')}><Ic name="layers" size={13} /><b>2</b>&nbsp;artifacts</button>
            <button className="comp-chip" title="Attach file"><Ic name="paperclip" size={13} /></button>

            <button className="chip-icon" title="llama-3.1-8b" style={{ width: 'auto', padding: '0 8px', gap: 5, fontSize: 11, fontFamily: 'var(--font-mono)', fontWeight: 500 }}>
              <span style={{ width: 6, height: 6, borderRadius: '50%', background: '#27A34F', display: 'inline-block', flex: 'none' }} />
              llama-3.1-8b<Ic name="chevron-down" size={10} />
            </button>
            <button className="chip-icon" title="4 tools active" onClick={() => openTab('connectors')}><Ic name="plug" size={14} /><span className="cb">4</span></button>
            <button className="chip-icon" title="2 artifacts" onClick={() => openTab('artifacts')}><Ic name="layers" size={14} /><span className="cb">2</span></button>
            <button className="chip-icon" title="Attach file"><Ic name="paperclip" size={14} /></button>

            <button className="send"><span className="send-lbl">Send</span><Ic name="arrow-up" size={13} /></button>
          </div>
        </div>
      </div>
    </div>
  );
}

/* ── Rail: parameters / connectors / artifacts ───────────────── */
const TAB_TITLES = { parameters: 'Parameters', connectors: 'Connectors', artifacts: 'Artifacts' };

function RightPanel({ tab, setTab }) {
  const { collapseRail } = useShell();
  return (<>
    <div className="r-tabs">
      <button className={'r-tab' + (tab === 'parameters' ? ' active' : '')} onClick={() => setTab('parameters')}><Ic name="sliders-horizontal" size={13} />Parameters <span className="badge">4</span></button>
      <button className={'r-tab' + (tab === 'connectors' ? ' active' : '')} onClick={() => setTab('connectors')}><Ic name="plug" size={13} />Connectors <span className="badge">6</span></button>
      <button className={'r-tab' + (tab === 'artifacts' ? ' active' : '')} onClick={() => setTab('artifacts')}><Ic name="layers" size={13} />Artifacts <span className="badge">2</span></button>
    </div>
    <div className="r-top">
      <div className="t">{TAB_TITLES[tab]} <span>· this chat</span></div>
      <div className="btns">
        <button title="Reset to defaults"><Ic name="rotate-ccw" size={13} /></button>
        <button title="Close" onClick={() => collapseRail && collapseRail()}><Ic name="x" size={13} /></button>
      </div>
    </div>
    <div className="r-scroll">
      {tab === 'parameters' && (
        <div className="r-pane active">
          <Collapsible title="Edited · overrides" count="4" defaultOpen edited>
            <Param name="Temperature" help defaultOn><Slider range="0.00 – 2.00" value="1.20" pct={60} /></Param>
            <Param name="Top P" help defaultOn><Slider range="0.00 – 1.00" value="0.90" pct={90} /></Param>
            <Param name="Max tokens" help defaultOn><input className="param-input" type="number" defaultValue={4096} /></Param>
            <Param name="Max tool iterations" help defaultOn><input className="param-input" type="number" defaultValue={5} /></Param>
          </Collapsible>
          <Collapsible title="Sampling" count="5">
            <Param name="Top K"><input className="param-input" type="number" placeholder="e.g. 40" /></Param>
            <Param name="Frequency penalty"><Slider range="−2.00 – 2.00" value="0.00" pct={50} /></Param>
            <Param name="Presence penalty"><Slider range="−2.00 – 2.00" value="0.00" pct={50} /></Param>
            <Param name="Seed"><input className="param-input" type="number" placeholder="e.g. 42" /></Param>
            <Param name="Stop sequences"><input className="param-input" placeholder="One per line" /></Param>
          </Collapsible>
          <Collapsible title="Behavior" count="3">
            <Param name="Stream response" defaultOn />
            <Param name="System prompt"><input className="param-input" placeholder="You are a helpful assistant…" /></Param>
            <Param name="JSON mode" />
          </Collapsible>
        </div>
      )}
      {tab === 'connectors' && (
        <div className="r-pane active">
          <Collapsible title="MCP servers" count="3" defaultOpen>
            <Conn ico="folder" cls="ci1" name="filesystem" desc="12 tools · local" dot defaultOn />
            <Conn ico="git-branch" cls="ci1" name="git" desc="8 tools · local" dot defaultOn />
            <Conn ico="database" cls="ci1" name="postgres" desc="connecting…" dot="#F39013" />
          </Collapsible>
          <Collapsible title="Built-in tools" count="4" defaultOpen>
            <Conn ico="globe" cls="ci2" name="Web search" desc="Exa · 1,500 queries/mo" defaultOn />
            <Conn ico="terminal" cls="ci2" name="Code interpreter" desc="Python sandbox · off" />
            <Conn ico="image" cls="ci2" name="Image generation" desc="DALL·E 3 · requires API key" />
            <Conn ico="brain" cls="ci2" name="Memory" desc="Long-term recall across chats" />
          </Collapsible>
          <Collapsible title="App connectors" count="5">
            <Conn ico="book" cls="ci4" name="Notion" desc="Not connected" />
            <Conn ico="message-square" cls="ci4" name="Slack" desc="Not connected" />
            <Conn ico="hard-drive" cls="ci4" name="Google Drive" desc="Not connected" />
            <Conn ico="github" cls="ci4" name="GitHub" desc="Not connected" />
          </Collapsible>
        </div>
      )}
      {tab === 'artifacts' && (
        <div className="r-pane active">
          <div style={{ fontSize: 11, color: 'hsl(var(--muted-foreground))', marginBottom: 10 }}>Generated in this conversation</div>
          <div className="art">
            <div className="art-thumb" style={{ background: '#0B0D1A', color: '#BEFA91' }}>{'{ }'}</div>
            <div className="art-info"><div className="art-name">tool_request.json</div><div className="art-meta">code · 2.1 KB</div></div>
            <div className="art-turn">turn 2</div>
          </div>
          <div className="art">
            <div className="art-thumb" style={{ background: 'rgba(39,163,79,.14)', color: '#1F7D3F' }}><Ic name="plug" size={14} /></div>
            <div className="art-info"><div className="art-name">filesystem.read_file</div><div className="art-meta">result · 2.1 KB</div></div>
            <div className="art-turn">turn 2</div>
          </div>
        </div>
      )}
    </div>
  </>);
}

/* ── Root ────────────────────────────────────────────────────── */
function ChatApp() {
  const [tab, setTab] = useState('parameters');
  const compact = false;
  const alwaysMeta = false;

  return (<>
    <AppShell
      section="chat" resizeKey="chat"
      sidebarWidth={260} railWidth={360}
      breadcrumb={<ChatTitle />}
      headerActions={<HeaderActions />}
      sidebar={<ChatSidebar />}
      contentClass="flush" mainScroll={false} railScroll={false}
      rail={<RightPanel tab={tab} setTab={setTab} />}
    >
      <ChatMain compact={compact} alwaysMeta={alwaysMeta} onOpenTab={setTab} />
    </AppShell>
  </>);
}

ReactDOM.createRoot(document.getElementById('root')).render(<ChatApp />);
