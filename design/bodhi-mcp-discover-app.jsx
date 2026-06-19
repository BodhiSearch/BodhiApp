/* ═══════════════════════════════════════════════════
   BODHI MCP — DISCOVER (3-column, on AppShell)
   bodhi-mcp-discover-app.jsx  (load after bodhi-app-shell.jsx + tweaks-panel.jsx)
═══════════════════════════════════════════════════ */
const { useState, useEffect, useMemo } = React;
const Ic = ShellIcon;

/* ══ DATA ══ */
const INITIAL_SERVERS = [
  { id:'notion', name:'Notion', publisher:'Notion Labs', verified:true, icon:'N', iconBg:'#000', iconColor:'#fff',
    category:'Productivity', catClass:'tag-lotus',
    desc:'Search, read and write pages & databases across your Notion workspace.',
    tools:7, installs:'7.4k', auth:['oauth'], url:'https://mcp.notion.com/mcp', transport:'streamable-http',
    adminApproved:true, disabled:false,
    userInstances:[{id:'inst-notion-1',name:'notion',status:'connected',authType:'oauth',time:'yesterday'}],
    approvalRequests:[],
    toolList:[
      {name:'notion-search',desc:'Perform a search across Notion — "internal" Search api.'},
      {name:'notion-fetch',desc:'Retrieves details about a Notion entity (page, database, block).'},
      {name:'notion-create-pages',desc:'Overview: creates one or more Notion pages with given properties.'},
      {name:'notion-update-page',desc:'Overview: update a Notion page properties.'},
      {name:'notion-delete',desc:'Archive or permanently delete a Notion block or page.'},
    ],
    stats:{installs:'7.4k',calls:'212k/wk',uptime:'98.1%',p50:'420ms'},
    meta:{license:'MIT',repo:'notion/mcp'},
    authConfigs:[{type:'oauth',name:'oauth-default',detail:'Dynamic registration · mcp.notion.com'}] },
  { id:'linear', name:'Linear', publisher:'Linear', verified:true, icon:'L', iconBg:'#5E6AD2', iconColor:'#fff',
    category:'Productivity', catClass:'tag-lotus',
    desc:'Manage issues, projects, cycles, and comments across your Linear workspace.',
    tools:3, installs:'3.1k', auth:['oauth'], url:'https://mcp.linear.app/mcp', transport:'streamable-http',
    adminApproved:true, disabled:false, userInstances:[], approvalRequests:[],
    toolList:[
      {name:'linear-create-issue',desc:'Create a new issue in a Linear team.'},
      {name:'linear-search',desc:'Search issues, projects and cycles.'},
      {name:'linear-update-issue',desc:'Update properties of an existing issue.'},
    ],
    stats:{installs:'3.1k',calls:'44k/wk',uptime:'99.7%',p50:'310ms'},
    meta:{license:'Apache-2.0',repo:'linear/mcp-server'},
    authConfigs:[{type:'oauth',name:'oauth-default',detail:'Dynamic registration · mcp.linear.app'}] },
  { id:'gmail', name:'Gmail', publisher:'Google', verified:true, icon:'G', iconBg:'#EA4335', iconColor:'#fff',
    category:'Comms', catClass:'tag-indigo',
    desc:'Send, draft, reply, forward, and bulk-modify messages and threads in Gmail.',
    tools:null, installs:'46k', auth:['oauth'], url:'https://mcp.google.com/gmail', transport:'streamable-http',
    adminApproved:false, disabled:false, userInstances:[],
    approvalRequests:[{email:'alice@company.com',time:'2h ago',initials:'A'},{email:'bob.smith@org.io',time:'5h ago',initials:'B'}],
    toolList:[
      {name:'gmail-send',desc:'Send an email from the authenticated account.'},
      {name:'gmail-search',desc:'Search Gmail threads and messages.'},
      {name:'gmail-draft',desc:'Create a draft message.'},
    ],
    stats:{installs:'46k',calls:'1.2M/wk',uptime:'99.9%',p50:'290ms'},
    meta:{license:'Proprietary',repo:'google/workspace-mcp'}, authConfigs:[] },
  { id:'slack', name:'Slack', publisher:'Slack', verified:true, icon:'S', iconBg:'#4A154B', iconColor:'#fff',
    category:'Comms', catClass:'tag-indigo',
    desc:'Channel-based messaging: post, search, react across your Slack workspace.',
    tools:null, installs:'14k', auth:['oauth'], url:'https://mcp.slack.com/mcp', transport:'streamable-http',
    adminApproved:true, disabled:false,
    userInstances:[{id:'inst-slack-1',name:'slack',status:'pending',authType:'oauth',time:'1d ago'}], approvalRequests:[],
    toolList:[
      {name:'slack-post-message',desc:'Post a message to a Slack channel.'},
      {name:'slack-search',desc:'Search messages and files in Slack.'},
    ],
    stats:{installs:'14k',calls:'380k/wk',uptime:'98.5%',p50:'350ms'},
    meta:{license:'Proprietary',repo:'slack/mcp-server'},
    authConfigs:[{type:'oauth',name:'oauth-default',detail:'Dynamic registration · mcp.slack.com'}] },
  { id:'exa', name:'Exa Search', publisher:'Exa Labs', verified:true, icon:'E', iconBg:'#1C1C1C', iconColor:'#fff',
    category:'Search & Web', catClass:'tag-saffron',
    desc:'Fast, intelligent web search and crawling — Exa-code context tool for coding.',
    tools:3, installs:'60k', auth:['key'], url:'https://mcp.exa.ai/mcp', transport:'streamable-http',
    adminApproved:true, disabled:false,
    userInstances:[{id:'inst-exa-1',name:'exa',status:'connected',authType:'key',time:'3h ago'}], approvalRequests:[],
    toolList:[
      {name:'exa-search',desc:'Perform a semantic web search using Exa.'},
      {name:'exa-get-contents',desc:'Retrieve contents of specific URLs.'},
      {name:'exa-find-similar',desc:'Find pages similar to a given URL.'},
    ],
    stats:{installs:'60k',calls:'890k/wk',uptime:'99.4%',p50:'210ms'},
    meta:{license:'MIT',repo:'exa-labs/exa-mcp-server'},
    authConfigs:[{type:'key',name:'apikey-default',detail:'Header: x-api-key'}] },
  { id:'github', name:'GitHub', publisher:'GitHub', verified:true, icon:'G', iconBg:'#24292e', iconColor:'#fff',
    category:'Dev Tools', catClass:'tag-indigo',
    desc:'Manage repos, issues, PRs, workflows, and Actions — official MCP server.',
    tools:null, installs:'5.2k', auth:['oauth'], url:'https://mcp.github.com/mcp', transport:'streamable-http',
    adminApproved:true, disabled:false, userInstances:[], approvalRequests:[],
    toolList:[
      {name:'github-create-issue',desc:'Open an issue in a GitHub repository.'},
      {name:'github-list-prs',desc:'List pull requests for a repository.'},
      {name:'github-search-code',desc:'Search code across GitHub repos.'},
    ],
    stats:{installs:'5.2k',calls:'71k/wk',uptime:'99.8%',p50:'280ms'},
    meta:{license:'MIT',repo:'github/github-mcp-server'},
    authConfigs:[{type:'oauth',name:'oauth-default',detail:'Dynamic registration · mcp.github.com'}] },
  { id:'playwright', name:'Playwright', publisher:'Microsoft', verified:true, icon:'▷', iconBg:'#2B2B2B', iconColor:'#45BA4B',
    category:'Browser', catClass:'tag-teal',
    desc:'Browser automation: click, fill, screenshot, assert across Chrome/FF/Safari.',
    tools:null, installs:'3.9k', auth:['key'], url:'https://mcp.playwright.dev/mcp', transport:'stdio',
    adminApproved:false, disabled:false, userInstances:[], approvalRequests:[],
    toolList:[
      {name:'playwright-navigate',desc:'Navigate to a URL in a browser.'},
      {name:'playwright-click',desc:'Click an element by selector.'},
      {name:'playwright-screenshot',desc:'Capture a screenshot of the current page.'},
    ],
    stats:{installs:'3.9k',calls:'52k/wk',uptime:'97.2%',p50:'1.1s'},
    meta:{license:'Apache-2.0',repo:'microsoft/playwright-mcp'}, authConfigs:[] },
  { id:'gsheets', name:'Google Sheets', publisher:'Google', verified:true, icon:'⊞', iconBg:'#1E8E3E', iconColor:'#fff',
    category:'Data', catClass:'tag-leaf',
    desc:'Read, write and format spreadsheet data; manage sheets and collaborate.',
    tools:null, installs:'55k', auth:['oauth'], url:'https://mcp.google.com/sheets', transport:'streamable-http',
    adminApproved:false, disabled:false, userInstances:[], approvalRequests:[{email:'carol@team.co',time:'1d ago',initials:'C'}],
    toolList:[
      {name:'sheets-read-range',desc:'Read values from a range in a Google Sheet.'},
      {name:'sheets-write-range',desc:'Write values to a range in a Google Sheet.'},
    ],
    stats:{installs:'55k',calls:'790k/wk',uptime:'99.9%',p50:'330ms'},
    meta:{license:'Proprietary',repo:'google/workspace-mcp'}, authConfigs:[] },
  { id:'supabase', name:'Supabase', publisher:'Supabase', verified:true, icon:'▲', iconBg:'#3ECF8E', iconColor:'#000',
    category:'Data', catClass:'tag-leaf',
    desc:'Search Supabase docs, troubleshoot errors, and manage projects & schema.',
    tools:null, installs:'6.6k', auth:['key'], url:'https://mcp.supabase.com/mcp', transport:'streamable-http',
    adminApproved:true, disabled:false, userInstances:[], approvalRequests:[],
    toolList:[
      {name:'supabase-query',desc:'Run a SQL query against the project database.'},
      {name:'supabase-list-tables',desc:'List all tables in the project schema.'},
    ],
    stats:{installs:'6.6k',calls:'88k/wk',uptime:'99.1%',p50:'480ms'},
    meta:{license:'Apache-2.0',repo:'supabase/mcp-server-supabase'},
    authConfigs:[{type:'key',name:'apikey-default',detail:'Header: Authorization'}] },
  { id:'context7', name:'Context7', publisher:'Upstash', verified:true, icon:'○', iconBg:'#6366F1', iconColor:'#fff',
    category:'Memory', catClass:'tag-indigo',
    desc:'Fetch up-to-date, version-specific docs and code examples into your prompts.',
    tools:null, installs:'12.8k', auth:['none'], url:'https://mcp.context7.com/mcp', transport:'streamable-http',
    adminApproved:false, disabled:true, userInstances:[], approvalRequests:[],
    toolList:[
      {name:'resolve-library-id',desc:'Resolve a library name to a Context7 library ID.'},
      {name:'get-library-docs',desc:'Fetch docs and usage examples for a library.'},
    ],
    stats:{installs:'12.8k',calls:'155k/wk',uptime:'99.3%',p50:'190ms'},
    meta:{license:'MIT',repo:'upstash/context7'},
    authConfigs:[{type:'none',name:'public',detail:'No authentication required'}] },
  { id:'deepwiki', name:'DeepWiki', publisher:'Dexa', verified:true, icon:'D', iconBg:'#6C47FF', iconColor:'#fff',
    category:'Dev Tools', catClass:'tag-indigo',
    desc:'Ask any question about GitHub repos — instant answers from code and docs.',
    tools:3, installs:'3.3k', auth:['none'], url:'https://mcp.deepwiki.com/mcp', transport:'streamable-http',
    adminApproved:true, disabled:false,
    userInstances:[{id:'inst-dw-1',name:'deepwiki',status:'connected',authType:'none',time:'2d ago'}], approvalRequests:[],
    toolList:[
      {name:'read_wiki_structure',desc:'Get a list of documentation topics for a GitHub repository.'},
      {name:'read_wiki_contents',desc:'View documentation about a GitHub repository or a specific topic.'},
      {name:'ask_question',desc:'Ask any question about a GitHub repository.'},
    ],
    stats:{installs:'3.3k',calls:'38k/wk',uptime:'98.8%',p50:'310ms'},
    meta:{license:'MIT',repo:'dexa/deepwiki-mcp'},
    authConfigs:[{type:'none',name:'public',detail:'No authentication required'}] },
];

/* ══ HELPERS ══ */
const AUTH_META = {
  oauth: { icon: 'lock', cls: 'auth-oauth', label: 'OAuth', iconBg: 'var(--c-indigo-bg)', iconColor: 'var(--c-indigo-text)' },
  key:   { icon: 'key',  cls: 'auth-key',   label: 'API Key', iconBg: 'var(--c-saffron-bg)', iconColor: 'var(--c-saffron-text)' },
  none:  { icon: 'unlock', cls: 'auth-none', label: 'Public', iconBg: 'var(--c-leaf-bg)', iconColor: 'var(--c-leaf-text)' },
};
function AuthBadges({ auths }) {
  return auths.map(a => {
    const m = AUTH_META[a] || AUTH_META.none;
    return <span key={a} className={'auth-badge ' + m.cls}><Ic name={m.icon} size={10} />{a === 'key' ? 'API Key' : m.label}</span>;
  });
}
const CatBadge = ({ s }) => <span className={'tag ' + s.catClass} style={{ fontSize: 10.5, padding: '2px 8px' }}>{s.category}</span>;

function statusClass(s) {
  if (s.disabled) return 'status-disabled';
  if (s.userInstances.some(i => i.status === 'connected')) return 'status-connected';
  if (s.userInstances.some(i => i.status === 'pending')) return 'status-pending';
  return '';
}
function userStatusSummary(s) {
  if (s.disabled) return 'disabled';
  if (s.userInstances.some(i => i.status === 'connected')) return 'connected';
  if (s.userInstances.some(i => i.status === 'pending')) return 'pending';
  if (s.adminApproved) return 'approved';
  return 'not';
}
function StatusLine({ s }) {
  const st = userStatusSummary(s);
  if (st === 'disabled') return <span className="card-status-line" style={{ color: 'hsl(var(--muted-foreground))' }}><Ic name="ban" size={11} />Admin disabled</span>;
  if (st === 'connected') return <span className="card-status-line" style={{ color: 'var(--c-connected-text)' }}><Ic name="circle-check" size={11} />{s.userInstances.filter(i => i.status === 'connected').length} instance(s)</span>;
  if (st === 'pending') return <span className="card-status-line" style={{ color: 'var(--c-pending-text)' }}><Ic name="clock" size={11} />Pending</span>;
  if (st === 'approved') return <span className="card-status-line status-approved"><Ic name="shield-check" size={11} />admin-approved</span>;
  return <span className="card-status-line"><Ic name="minus-circle" size={11} />not yet in this app</span>;
}

function goToPlayground(instId, instName, serverId) {
  window.location.href = `Bodhi MCP Playground.html?instance=${instId}&name=${encodeURIComponent(instName)}&server=${serverId}`;
}
function goToNewMCP(serverId, authType) {
  window.location.href = `Bodhi MCP New Instance.html?server=${serverId}&auth=${authType}`;
}

/* ══ CTA button (card/list foot) ══ */
function Cta({ s, role, onOpen }) {
  if (s.disabled) return <button className="cta cta-unavail" disabled>Unavailable</button>;
  const st = userStatusSummary(s);
  const stop = (e, tab) => { e.stopPropagation(); onOpen(s.id, tab); };
  if (role === 'admin') {
    const cnt = s.approvalRequests.length;
    const label = !s.adminApproved ? `Configure${cnt ? ` (${cnt})` : ''} · approve` : 'Configure';
    return <button className="cta cta-configure" onClick={e => stop(e, 'configure')}><Ic name="settings-2" size={11} />{label}</button>;
  }
  if (st === 'connected' || st === 'pending') return <button className="cta cta-view" onClick={e => stop(e, 'connect')}><Ic name="plug" size={11} />My instances</button>;
  if (st === 'approved') return <button className="cta cta-add" onClick={e => stop(e, 'connect')}><Ic name="plus" size={11} />Connect</button>;
  return <button className="cta cta-submit" onClick={e => e.stopPropagation()}><Ic name="send" size={11} />Request Approval</button>;
}

/* ══ Card / Row ══ */
function McpCard({ s, role, active, onOpen }) {
  return (
    <div className={`l-card mcp-card ${statusClass(s)}${active ? ' active' : ''}`} onClick={() => onOpen(s.id)}>
      <div className="card-head">
        <div className="card-icon" style={{ background: s.iconBg }}><span style={{ color: s.iconColor, fontSize: 15, fontWeight: 800 }}>{s.icon}</span></div>
        <div className="card-title-block">
          <div className="card-name">{s.name}</div>
          <div className="card-publisher">{s.publisher}{s.verified && <Ic name="badge-check" size={10} color="var(--c-indigo-text)" />}</div>
        </div>
        <CatBadge s={s} />
      </div>
      <div className="card-desc">{s.desc}</div>
      <div className="card-meta">
        <span className="card-meta-item"><Ic name="wrench" size={11} />{s.tools != null ? s.tools : '?'} tools</span>
        <span className="card-meta-item"><Ic name="download" size={11} />{s.installs}</span>
        <AuthBadges auths={s.auth} />
        {role === 'admin' && s.approvalRequests.length > 0 && (
          <span className="auth-badge" style={{ background: 'var(--c-saffron-bg)', borderColor: 'var(--c-saffron-bd)', color: 'var(--c-saffron-text)' }}>
            <Ic name="users" size={10} />{s.approvalRequests.length} request{s.approvalRequests.length > 1 ? 's' : ''}
          </span>
        )}
      </div>
      {s.userInstances.length > 0 && (
        <div className="card-instances">
          {s.userInstances.map(inst => (
            <div className="card-instance-row" key={inst.id}>
              <span className={'inst-dot ' + inst.status}></span>
              <span className="inst-name">{inst.name}</span>
              <span className="inst-time">{inst.time}</span>
              {inst.status === 'connected' && (
                <button className="inst-play-btn" title="Open playground" onClick={e => { e.stopPropagation(); goToPlayground(inst.id, inst.name, s.id); }}><Ic name="play" size={11} /></button>
              )}
            </div>
          ))}
        </div>
      )}
      <div className="card-foot"><StatusLine s={s} /><Cta s={s} role={role} onOpen={onOpen} /></div>
    </div>
  );
}

function McpRow({ s, role, active, onOpen }) {
  return (
    <ListRow className={statusClass(s)} active={active} onSelect={() => onOpen(s.id)} label={`Open ${s.name}`}>
      <div className="row-icon"><div className="row-icon-box" style={{ background: s.iconBg, color: s.iconColor, borderColor: s.iconBg }}>{s.icon}</div></div>
      <div className="row-body">
        <div className="row-name">{s.name}</div>
        <div className="row-pub">{s.publisher}{s.verified && <Ic name="badge-check" size={10} color="var(--c-indigo-text)" />}</div>
      </div>
      <div className="row-cat"><CatBadge s={s} /></div>
      <div className="row-tools"><div className="row-tools-val">{s.tools != null ? s.tools : '?'}</div><div className="row-tools-lbl">tools</div></div>
      <div className="row-auth"><AuthBadges auths={s.auth} /></div>
      <div className="row-stat"><StatusLine s={s} /></div>
      <div className="row-act"><Cta s={s} role={role} onOpen={onOpen} /></div>
    </ListRow>
  );
}

/* ══ Sidebar filters (decorative + collapse-aware) ══ */
function DiscoverSidebar() {
  return (
    <>
      <ShellFilterGroup icon="shapes" label="Category" clearable chips={[
        { label: 'All', defaultOn: true }, { label: 'Productivity', color: 'lotus' }, { label: 'Dev Tools', color: 'indigo' },
        { label: 'Search & Web', color: 'saffron' }, { label: 'Browser' }, { label: 'Data', color: 'leaf' },
        { label: 'AI & Content', color: 'teal' }, { label: 'Memory' }, { label: 'Comms' }]} />
      <ShellFilterGroup icon="key-round" label="Auth Type" chips={[
        { label: 'Any', defaultOn: true }, { label: 'OAuth', color: 'indigo' }, { label: 'API Key', color: 'saffron' }, { label: 'No auth' }]} />
      <ShellFilterGroup icon="activity" label="My Status" chips={[
        { label: 'All', defaultOn: true }, { label: 'Connected', color: 'leaf' }, { label: 'Approved', color: 'indigo' },
        { label: 'Pending', color: 'saffron' }, { label: 'Not added' }]} />
      <ShellFilterGroup icon="badge-check" label="Publisher" chips={[
        { label: 'Verified ✓' }, { label: 'Official' }, { label: 'Community' }]} />
    </>
  );
}

/* ══ Detail panel (rail) ══ */
function SpecRow({ k, v, small }) {
  return <div className="spec-row"><span className="spec-k">{k}</span><span className="spec-v" style={small ? { fontSize: 11, wordBreak: 'break-all' } : null}>{v}</span></div>;
}

function FootButton({ s, role, setTab }) {
  if (s.disabled) return <button className="btn-full btn-disabled" disabled><Ic name="ban" size={14} />Unavailable</button>;
  const st = userStatusSummary(s);
  if (role === 'admin') return <button className="btn-full btn-indigo" onClick={() => setTab('configure')}><Ic name="settings-2" size={14} /> Configure Server</button>;
  if (st === 'connected' || st === 'pending') return <button className="btn-full btn-leaf" onClick={() => setTab('connect')}><Ic name="plug" size={14} /> Manage Instances</button>;
  if (st === 'approved') return <button className="btn-full btn-lotus" onClick={() => setTab('connect')}><Ic name="plus" size={14} /> Connect to this server</button>;
  return <button className="btn-full btn-lotus"><Ic name="send" size={14} /> Request Approval</button>;
}

/* Compact rail header — sits on the shared 56px header gridline */
function DiscoverRailHeader({ s, setActiveId }) {
  return (
    <div className="rail-head">
      <div className="rail-head-icon" style={{ background: s.iconBg }}><span style={{ color: s.iconColor }}>{s.icon}</span></div>
      <div className="rail-head-body">
        <div className="rail-head-title">{s.name}</div>
        <div className="rail-head-pub">{s.publisher}{s.verified && <Ic name="badge-check" size={10} color="var(--c-indigo-text)" />}<span style={{ margin: '0 2px', opacity: .4 }}>·</span>{s.category}</div>
      </div>
      <button className="panel-close" onClick={() => setActiveId(null)}><Ic name="x" size={14} /></button>
    </div>
  );
}

function DetailPanel({ s, role, tab, setTab, onConfig, actions }) {
  const st = userStatusSummary(s);
  const isAdmin = role === 'admin';
  const hasReqs = s.approvalRequests.length > 0;

  let statusPill;
  if (s.disabled) statusPill = <span className="auth-badge" style={{ background: 'hsl(var(--muted))', borderColor: 'hsl(var(--border))', color: 'hsl(var(--muted-foreground))' }}><Ic name="ban" size={10} />Admin Disabled</span>;
  else if (st === 'connected') statusPill = <span className="auth-badge" style={{ background: 'var(--c-connected-bg)', borderColor: 'var(--c-connected-bd)', color: 'var(--c-connected-text)' }}><Ic name="circle-check" size={10} />Connected</span>;
  else if (st === 'pending') statusPill = <span className="auth-badge" style={{ background: 'var(--c-pending-bg)', borderColor: 'var(--c-pending-bd)', color: 'var(--c-pending-text)' }}><Ic name="clock" size={10} />Request pending</span>;
  else if (st === 'approved') statusPill = <span className="auth-badge" style={{ background: 'var(--c-indigo-bg)', borderColor: 'var(--c-indigo-bd)', color: 'var(--c-indigo-text)' }}><Ic name="shield-check" size={10} />Admin approved</span>;
  else if (!s.adminApproved && isAdmin) statusPill = <span className="auth-badge" style={{ background: 'var(--c-saffron-bg)', borderColor: 'var(--c-saffron-bd)', color: 'var(--c-saffron-text)' }}><Ic name="alert-circle" size={10} />Not configured</span>;
  else statusPill = <span className="auth-badge auth-none"><Ic name="minus-circle" size={10} />Not in this app</span>;

  const tabs = ['about', 'capabilities', 'connection'];
  if (!isAdmin) tabs.push('connect');
  if (isAdmin) tabs.push('configure');
  tabs.push('metadata');
  const tabLabel = { about: 'About', capabilities: 'Capabilities', connection: 'Connection', connect: 'Connect', configure: 'Configure', metadata: 'Metadata' };

  return (
    <div className="mcp-detail">
      <div className="panel-status-row">
        {statusPill}
        <AuthBadges auths={s.auth} />
        <span className="auth-badge auth-none" style={{ marginLeft: 'auto' }}><Ic name="download" size={10} />{s.installs}</span>
      </div>
      <div className="panel-tabs">
        {tabs.map(t => (
          <button key={t} className={'ptab' + (t === 'configure' ? ' admin-tab' : '') + (tab === t ? ' on' : '')} onClick={() => setTab(t)}>
            {tabLabel[t]}{t === 'configure' && hasReqs && <span className="ptab-dot"></span>}
          </button>
        ))}
      </div>
      <div className="panel-body">
        {tab === 'about' && (<>
          <div className="p-section"><div className="p-sec-lbl">Description</div><div style={{ fontSize: 13, lineHeight: 1.6, color: 'hsl(var(--muted-foreground))' }}>{s.desc}</div></div>
          <div className="p-section"><div className="p-sec-lbl">Tools ({s.toolList.length}{s.tools == null ? '+' : ''})</div>
            <div className="tool-list">{s.toolList.map(t => <div className="tool-item" key={t.name}><div className="tool-name">{t.name}</div><div className="tool-desc">{t.desc}</div></div>)}</div>
          </div>
          <div className="p-section"><div className="p-sec-lbl">Stats (30d)</div>
            <div className="stat-grid">{Object.entries(s.stats).map(([k, v]) => <div className="stat-box" key={k}><div className="stat-val">{v}</div><div className="stat-lbl">{k}</div></div>)}</div>
          </div>
        </>)}

        {tab === 'capabilities' && (<>
          <div className="p-section"><div className="p-sec-lbl">Exposed Tools</div>
            <div className="cap-chips">{s.toolList.map(t => <span key={t.name} className="tag tag-indigo" style={{ fontSize: 12, padding: '3px 9px', fontFamily: 'var(--font-mono)' }}>{t.name}</span>)}</div>
          </div>
          <div className="p-section"><div className="p-sec-lbl">Transport</div>
            <div className="spec-table"><SpecRow k="protocol" v={s.transport} /><SpecRow k="streaming" v={s.transport === 'streamable-http' ? 'Yes' : 'No'} /></div>
          </div>
        </>)}

        {tab === 'connection' && (<>
          <div className="p-section"><div className="p-sec-lbl">Endpoint</div>
            <div className="spec-table"><SpecRow k="URL" v={s.url} small /><SpecRow k="Transport" v={s.transport} /></div>
          </div>
          <div className="p-section"><div className="p-sec-lbl">Auth methods configured</div>
            {s.authConfigs.length ? s.authConfigs.map((ac, i) => {
              const m = AUTH_META[ac.type];
              return (
                <div className="auth-method-row" style={{ cursor: 'default' }} key={i}>
                  <div className="auth-method-icon" style={{ background: m.iconBg }}><Ic name={m.icon} size={13} color={m.iconColor} /></div>
                  <div className="auth-method-body"><div className="auth-method-name">{ac.name}</div><div className="auth-method-detail">{ac.detail}</div></div>
                </div>
              );
            }) : <div style={{ fontSize: 12.5, color: 'hsl(var(--muted-foreground))', padding: '8px 0' }}>{isAdmin ? 'No auth configured yet. Use the Configure tab to add one.' : 'No authentication configured by admin.'}</div>}
          </div>
        </>)}

        {tab === 'connect' && (<>
          {s.userInstances.length > 0 && (
            <div className="p-section"><div className="p-sec-lbl">My Instances</div>
              {s.userInstances.map(inst => (
                <div className="user-instance-row" key={inst.id}>
                  <span className={'ui-dot ' + inst.status}></span>
                  <div className="ui-body"><div className="ui-name">{inst.name}</div>
                    <div className="ui-meta"><span>{inst.authType === 'oauth' ? 'OAuth' : inst.authType === 'key' ? 'API Key' : 'Public'}</span><span>·</span><span>{inst.time}</span></div>
                  </div>
                  <div className="ui-actions">
                    {inst.status === 'connected'
                      ? <button className="ui-play-btn" onClick={() => goToPlayground(inst.id, inst.name, s.id)}><Ic name="play" size={11} />Playground</button>
                      : <span style={{ fontSize: 11.5, color: 'var(--c-pending-text)', fontWeight: 600 }}>Pending</span>}
                    <button className="ui-del-btn" title="Delete instance" onClick={() => actions.deleteInstance(s.id, inst.id)}><Ic name="trash-2" size={11} /></button>
                  </div>
                </div>
              ))}
            </div>
          )}
          <div className="p-section">
            <div className="p-sec-lbl">Connect with…</div>
            {s.adminApproved && s.authConfigs.map(ac => {
              const m = AUTH_META[ac.type];
              const label = ac.type === 'oauth' ? 'Connect with OAuth' : ac.type === 'key' ? 'Connect with API Key' : 'Connect (Public)';
              const desc = ac.type === 'oauth' ? 'Authorize via OAuth redirect' : ac.type === 'key' ? 'Provide your API key' : 'No authentication needed';
              return (
                <div className="auth-connect-row" key={ac.name} onClick={() => goToNewMCP(s.id, ac.type)}>
                  <div className="acr-icon" style={{ background: m.iconBg }}><Ic name={m.icon} size={13} color={m.iconColor} /></div>
                  <div className="acr-body"><div className="acr-name">{label}</div><div className="acr-desc">{desc}</div></div>
                  <div className="acr-arrow"><Ic name="chevron-right" size={13} /></div>
                </div>
              );
            })}
            {s.adminApproved && !s.authConfigs.some(ac => ac.type === 'none') && (
              <div className="auth-connect-row" onClick={() => goToNewMCP(s.id, 'none')}>
                <div className="acr-icon" style={{ background: 'var(--c-leaf-bg)' }}><Ic name="unlock" size={13} color="var(--c-leaf-text)" /></div>
                <div className="acr-body"><div className="acr-name">Connect (Public)</div><div className="acr-desc">No authentication required</div></div>
                <div className="acr-arrow"><Ic name="chevron-right" size={13} /></div>
              </div>
            )}
            {!s.adminApproved && <div style={{ fontSize: 12.5, color: 'hsl(var(--muted-foreground))', padding: '8px 0' }}>This server is not yet approved by your admin.</div>}
          </div>
        </>)}

        {tab === 'configure' && (<>
          {hasReqs && (<>
            <div className="p-section">
              <div className="p-sec-lbl" style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                <span>Approval Requests</span>
                <span style={{ background: 'var(--c-saffron-bg)', border: '1px solid var(--c-saffron-bd)', color: 'var(--c-saffron-text)', fontSize: 10, fontWeight: 700, padding: '1px 6px', borderRadius: 99 }}>{s.approvalRequests.length}</span>
              </div>
              {s.approvalRequests.map(req => (
                <div className="approval-request-row" key={req.email}>
                  <div className="arr-avatar">{req.initials}</div>
                  <div className="arr-body"><div className="arr-email">{req.email}</div><div className="arr-time">{req.time}</div></div>
                  <div className="arr-actions">
                    <button className="arr-approve" onClick={() => actions.approveRequest(s.id, req.email)}>Approve</button>
                    <button className="arr-reject" onClick={() => actions.rejectRequest(s.id, req.email)}>Reject</button>
                  </div>
                </div>
              ))}
            </div>
            <div className="form-divider"></div>
          </>)}
          <div className="p-section">
            <div className="p-sec-lbl">Server Status</div>
            <div className="form-toggle-row">
              <div><div className="form-toggle-label">Enabled globally</div><div style={{ fontSize: 11.5, color: 'hsl(var(--muted-foreground))', marginTop: 2 }}>Allow users to connect to this server</div></div>
              <div className={'sw' + (!s.disabled ? ' on' : '')} onClick={() => actions.toggleDisabled(s.id)}></div>
            </div>
            {!s.adminApproved ? (
              <div className="form-toggle-row" style={{ marginTop: 8 }}>
                <div><div className="form-toggle-label">Admin approved</div><div style={{ fontSize: 11.5, color: 'hsl(var(--muted-foreground))', marginTop: 2 }}>Make visible and connectable to users</div></div>
                <div className={'sw' + (s.adminApproved ? ' on' : '')} onClick={() => actions.toggleApproved(s.id)}></div>
              </div>
            ) : (
              <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginTop: 8, padding: '8px 10px', background: 'var(--c-leaf-bg)', border: '1px solid var(--c-leaf-bd)', borderRadius: 8, fontSize: 12.5, fontWeight: 600, color: 'var(--c-leaf-text)' }}>
                <Ic name="shield-check" size={13} /> Approved & active
              </div>
            )}
          </div>
          <div className="p-section">
            <div className="p-sec-lbl" style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <span>Authentication Methods</span>
              <button style={{ fontSize: 11.5, fontWeight: 600, color: 'var(--c-indigo-text)', background: 'none', border: 'none', display: 'flex', alignItems: 'center', gap: 3 }} onClick={() => onConfig(s, 'add')}><Ic name="plus" size={11} /> Add</button>
            </div>
            {s.authConfigs.length ? s.authConfigs.map((ac, i) => {
              const m = AUTH_META[ac.type];
              return (
                <div className="auth-method-row" key={i}>
                  <div className="auth-method-icon" style={{ background: m.iconBg }}><Ic name={m.icon} size={13} color={m.iconColor} /></div>
                  <div className="auth-method-body"><div className="auth-method-name">{ac.name}</div><div className="auth-method-detail">{ac.detail}</div></div>
                  <div className="auth-method-actions">
                    <button className="auth-method-btn" title="Edit" onClick={() => onConfig(s, i)}><Ic name="pencil" size={12} /></button>
                    <button className="auth-method-btn danger" title="Delete" onClick={() => actions.deleteAuthConfig(s.id, i)}><Ic name="trash-2" size={12} /></button>
                  </div>
                </div>
              );
            }) : <div style={{ fontSize: 12.5, color: 'hsl(var(--muted-foreground))', padding: 12, border: '1px dashed hsl(var(--border))', borderRadius: 8, textAlign: 'center' }}>No auth configured. Click <strong>Add</strong> to set up OAuth, API Key, or leave as Public.</div>}
          </div>
        </>)}

        {tab === 'metadata' && (
          <div className="p-section"><div className="p-sec-lbl">Metadata</div>
            <div className="spec-table">
              <SpecRow k="license" v={s.meta.license} /><SpecRow k="repo" v={s.meta.repo} /><SpecRow k="publisher" v={s.publisher} />
              <SpecRow k="verified" v={s.verified ? '✓ Yes' : 'No'} /><SpecRow k="transport" v={s.transport} />
            </div>
          </div>
        )}
      </div>
      <div className="panel-foot">
        {tab === 'configure'
          ? <><button className="btn-full btn-indigo" onClick={() => onConfig(s, 'add')}><Ic name="plus" size={14} /> Add Auth Method</button>
              <button className="btn-full btn-ghost" onClick={() => onConfig(s, 'edit-server')}><Ic name="settings-2" size={14} /> Edit Server Details</button></>
          : <FootButton s={s} role={role} setTab={setTab} />}
      </div>
    </div>
  );
}

/* ══ Config slide-over ══ */
function ConfigDrawer({ state, onClose, onSave }) {
  const open = Boolean(state);
  const s = state ? state.server : null;
  const mode = state ? state.mode : null; // 'add' | number | 'edit-server'
  const existing = (s && typeof mode === 'number') ? s.authConfigs[mode] : null;
  const [authType, setAuthType] = useState('oauth');
  const [authName, setAuthName] = useState('');

  useEffect(() => {
    if (!s) return;
    setAuthType(existing ? existing.type : 'oauth');
    setAuthName(existing ? existing.name : '');
  }, [state]);

  const isEditServer = mode === 'edit-server';
  const title = !s ? '' : isEditServer ? `Edit: ${s.name}` : typeof mode === 'number' ? `Edit Auth: ${existing.name}` : `Add Auth for ${s.name}`;
  const sub = isEditServer ? 'Update server URL, name, and settings' : typeof mode === 'number' ? 'Update authentication configuration' : 'Configure how users connect to this server';

  return (
    <div className="config-overlay" style={{ pointerEvents: open ? 'auto' : 'none' }}>
      <div className={'config-scrim' + (open ? ' visible' : '')} onClick={onClose}></div>
      <div className={'config-drawer' + (open ? ' open' : '')}>
        {s && (<>
          <div className="config-drawer-head">
            <div className="config-drawer-icon" style={{ background: s.iconBg }}><span style={{ color: s.iconColor, fontSize: 16, fontWeight: 800 }}>{s.icon}</span></div>
            <div className="config-drawer-title"><h2>{title}</h2><p>{sub}</p></div>
            <button className="icon-btn" onClick={onClose}><Ic name="x" size={14} /></button>
          </div>
          <div className="config-drawer-body">
            {isEditServer ? (<>
              <div className="form-field"><div className="form-label">URL <span className="req">*</span></div><input className="form-input" defaultValue={s.url} placeholder="https://mcp.example.com/mcp" /><div className="form-hint">The MCP server endpoint URL</div></div>
              <div className="form-field"><div className="form-label">Name <span className="req">*</span></div><input className="form-input" defaultValue={s.name} /></div>
              <div className="form-field"><div className="form-label">Description <span className="hint">(optional)</span></div><textarea className="form-textarea" defaultValue={s.desc}></textarea></div>
            </>) : (<>
              <div className="form-field">
                <div className="form-label">Auth Type <span className="req">*</span></div>
                <select className="form-select" value={authType} onChange={e => setAuthType(e.target.value)}>
                  <option value="oauth">OAuth 2.0</option>
                  <option value="key">Header / Query Key</option>
                  <option value="none">Public (No auth)</option>
                </select>
              </div>
              <div className="form-field"><div className="form-label">Config Name <span className="req">*</span></div><input className="form-input" value={authName} onChange={e => setAuthName(e.target.value)} placeholder="e.g. oauth-default" /><div className="form-hint">Internal identifier for this auth config</div></div>
              {authType === 'oauth' && (<>
                <div className="form-divider"></div>
                <div className="form-section-head">OAuth Configuration</div>
                <div className="form-field"><div className="form-label">Authorization Endpoint</div><input className="form-input" placeholder="https://example.com/authorize" defaultValue={existing?.authEndpoint || ''} /></div>
                <div className="form-field"><div className="form-label">Token Endpoint</div><input className="form-input" placeholder="https://example.com/token" defaultValue={existing?.tokenEndpoint || ''} /></div>
                <div className="form-field"><div className="form-label">Scopes <span className="hint">(optional)</span></div><input className="form-input" placeholder="read write" defaultValue={existing?.scopes || ''} /></div>
              </>)}
              {authType === 'key' && (<>
                <div className="form-divider"></div>
                <div className="form-section-head">Key / Value Configuration</div>
                <div className="form-field"><div className="form-label">Inject Via</div><select className="form-select"><option>Header</option><option>Query Parameter</option></select></div>
                <div className="form-field"><div className="form-label">Key name <span className="req">*</span></div><input className="form-input" placeholder="e.g. x-api-key" defaultValue={existing?.keyName || ''} /></div>
              </>)}
              {authType === 'none' && (
                <><div className="form-divider"></div>
                <div style={{ padding: 12, background: 'var(--c-leaf-bg)', border: '1px solid var(--c-leaf-bd)', borderRadius: 8, fontSize: 13, color: 'var(--c-leaf-text)' }}>
                  <strong>Public access</strong> — no authentication required. All users with access to this app can connect without providing any credentials.
                </div></>
              )}
            </>)}
          </div>
          <div className="config-drawer-foot">
            <button className="btn-cta-primary" onClick={() => onSave(s.id, isEditServer ? null : { type: authType, name: authName })}><Ic name="check" size={14} /> {isEditServer ? 'Save Changes' : 'Save Auth'}</button>
            <button className="btn-cta-secondary" onClick={onClose}>Cancel</button>
          </div>
        </>)}
      </div>
    </div>
  );
}

/* ══ Root ══ */
function DiscoverApp() {
  const [servers, setServers] = useState(INITIAL_SERVERS);
  const [view, setView] = useState('list');
  const [stab, setStab] = useState('all');
  const [search, setSearch] = useState('');
  const [role, setRole] = useState('user');
  const [activeId, setActiveId] = useState(null);
  const [tab, setTab] = useState('about');
  const [configState, setConfigState] = useState(null);

  useEffect(() => {
    if (!window.matchMedia('(max-width:767px)').matches) setActiveId(INITIAL_SERVERS[0].id);
  }, []);

  const activeServer = servers.find(s => s.id === activeId) || null;
  const totalApprovals = servers.reduce((n, s) => n + s.approvalRequests.length, 0);

  const updateServer = (id, fn) => setServers(prev => prev.map(s => s.id === id ? fn({ ...s }) : s));

  const actions = {
    deleteInstance: (id, instId) => updateServer(id, s => ({ ...s, userInstances: s.userInstances.filter(i => i.id !== instId) })),
    deleteAuthConfig: (id, idx) => updateServer(id, s => ({ ...s, authConfigs: s.authConfigs.filter((_, i) => i !== idx) })),
    approveRequest: (id, email) => updateServer(id, s => ({ ...s, approvalRequests: s.approvalRequests.filter(r => r.email !== email), adminApproved: true })),
    rejectRequest: (id, email) => updateServer(id, s => ({ ...s, approvalRequests: s.approvalRequests.filter(r => r.email !== email) })),
    toggleDisabled: id => updateServer(id, s => ({ ...s, disabled: !s.disabled })),
    toggleApproved: id => updateServer(id, s => ({ ...s, adminApproved: !s.adminApproved })),
  };

  const matchesFilter = s => {
    if (stab === 'mine' && s.userInstances.length === 0) return false;
    if (stab === 'connected' && s.userInstances.every(i => i.status !== 'connected')) return false;
    if (stab === 'approved' && !s.adminApproved) return false;
    if (stab === 'pending' && s.userInstances.every(i => i.status !== 'pending')) return false;
    if (stab === 'not' && s.userInstances.some(i => i.status === 'connected' || i.status === 'pending')) return false;
    if (stab === 'approval_req' && !s.approvalRequests.length) return false;
    if (search) {
      const q = search.toLowerCase();
      if (!s.name.toLowerCase().includes(q) && !s.publisher.toLowerCase().includes(q) && !s.category.toLowerCase().includes(q)) return false;
    }
    return true;
  };
  const visible = servers.filter(matchesFilter);

  const STABS = [
    { id: 'all', label: 'Explore' },
    { id: 'mine', label: 'My Instances', catCls: 'c-leaf' },
    { id: 'approved', label: 'Admin-approved', catCls: 'c-indigo' },
    { id: 'connected', label: 'Connected', catCls: 'c-leaf' },
    { id: 'not', label: 'Not connected' },
    { id: 'pending', label: 'Pending', catCls: 'c-saffron' },
  ];

  const headerActions = (
    <div className="role-badge" onClick={() => setRole(r => r === 'user' ? 'admin' : 'user')} title="Click to switch role">
      <Ic name="user-circle" size={11} /> Role: {role === 'admin' ? 'Admin' : 'User'} <Ic name="chevron-down" size={11} />
    </div>
  );

  return (
    <>
      <AppShell
        section="mcp" subPage="discover" resizeKey="mcp"
        railWidth={380} railMin={320} railMax={540}
        breadcrumb={[{ label: 'Bodhi', href: 'Bodhi Chat.html' }, { label: 'MCP', href: 'Bodhi MCP Discover v2.html' }, { label: 'All MCPs', current: true }]}
        headerActions={headerActions}
        sidebar={<DiscoverSidebar />}
        contentClass="flush" mainScroll={false} railScroll={false}
        railHeader={activeServer ? <DiscoverRailHeader s={activeServer} setActiveId={setActiveId} /> : undefined}
        rail={activeServer ? <DetailPanel s={activeServer} role={role} tab={tab} setTab={setTab}
          onConfig={(s, m) => setConfigState({ server: s, mode: m })} actions={actions} /> : null}
      >
        <MainArea
          visible={visible} view={view} setView={setView} stab={stab} setStab={setStab} search={search} setSearch={setSearch}
          role={role} activeId={activeId} STABS={STABS} totalApprovals={totalApprovals}
          onOpen={(id, t) => { setActiveId(id); setTab(t || 'about'); }} />
      </AppShell>

      <ConfigDrawer state={configState} onClose={() => setConfigState(null)}
        onSave={(id, ac) => {
          if (ac && ac.name) {
            updateServer(id, s => {
              if (s.authConfigs.find(c => c.name === ac.name)) return s;
              const detail = ac.type === 'oauth' ? 'Dynamic registration' : ac.type === 'key' ? 'Header key' : 'Public access';
              return { ...s, authConfigs: [...s.authConfigs, { type: ac.type, name: ac.name, detail }], adminApproved: true };
            });
          }
          setConfigState(null);
        }} />
    </>
  );
}

/* main content — needs useShell for openRail, so it's its own component inside AppShell.
   Built on the shared list-page primitives: <ListToolbar> (category pills left +
   collapsible search) over a single .l-scroll region holding the card grid or the
   edge-to-edge <ListView>. */
function MainArea({ visible, view, setView, stab, setStab, search, setSearch, role, activeId, STABS, totalApprovals, onOpen }) {
  const { openRail } = useShell();
  useListKeyNav();
  const open = (id, t) => { onOpen(id, t); openRail(); };

  const cats = STABS.map(t => ({ id: t.id, label: t.label, cls: t.catCls }));
  if (role === 'admin' && totalApprovals > 0) {
    cats.push({ id: 'approval_req', label: 'Approval Requests', cls: 'c-saffron', badge: totalApprovals });
  }

  const listHead = (
    <>
      <div className="lh-icon"></div>
      <div className="lh-name l-lh">Server</div>
      <div className="lh-cat l-lh">Category</div>
      <div className="lh-tools l-lh">Tools</div>
      <div className="lh-auth l-lh">Auth</div>
      <div className="lh-stat l-lh">Status</div>
      <div className="lh-act"></div>
    </>
  );

  return (
    <div className="l-page">
      <ListToolbar
        categories={cats} category={stab} onCategory={setStab}
        search={search} onSearch={setSearch}
        searchPlaceholder="Search MCP servers by name, publisher, or tag…"
        actions={
          <button className={'l-iconbtn' + (view === 'cards' ? ' on' : '')}
                  title={view === 'cards' ? 'Switch to list view' : 'Switch to card view'}
                  onClick={() => setView(v => v === 'cards' ? 'list' : 'cards')}>
            <Ic name={view === 'cards' ? 'list' : 'layout-grid'} size={15} />
          </button>
        } />
      <div className="l-scroll">
        {visible.length === 0 ? (
          <div className="l-empty"><Ic name="search-x" size={30} /><div className="l-empty-t">No servers match</div><div className="l-empty-s">Try adjusting filters or search</div></div>
        ) : view === 'cards' ? (
          <div className="l-cardgrid">{visible.map(s => <McpCard key={s.id} s={s} role={role} active={activeId === s.id} onOpen={open} />)}</div>
        ) : (
          <ListView head={listHead}>
            {visible.map(s => <McpRow key={s.id} s={s} role={role} active={activeId === s.id} onOpen={open} />)}
          </ListView>
        )}
      </div>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<DiscoverApp />);
