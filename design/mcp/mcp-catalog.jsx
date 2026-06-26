/* ═══════════════════════════════════════════════════════════════
   BODHI MCP — SHARED CATALOG  ·  single source of truth
   mcp/mcp-catalog.jsx   (load FIRST of the MCP modules, after the shell)

   ONE server catalog for every MCP page. Each page derives the view it
   needs from CATALOG instead of hand-maintaining its own copy:

     • Explore / My MCPs  → CATALOG (full objects, instances + auth)
     • Configure server   → KNOWN_SERVERS   (registered servers, by id)
     • New / Edit instance→ CONNECTABLE_SERVERS (registered & enabled)
     • Playground         → toolsFor(id)    (tool specs per server)

   Also the home of the auth vocabulary (AUTH_META), badges, status
   helpers, and cross-page navigation. Everything is published to
   window so the per-page app scripts can read it as globals.

   Domain model (mirrors the backend):
     server ─1:*→ auth-mechanism ─1:*→ instance
       • server   : base url + transport. `registered` = saved in our DB;
                    `disabled` = turned off for the whole workspace.
       • auth     : public (always available, no DB row), oauth
                    (dcr | pre-registered), or header/query key. A server
                    may expose several, even several of one type.
       • instance : a user's personal connection to a server + mechanism.
═══════════════════════════════════════════════════════════════ */
const Ic = ShellIcon;

/* ══ CATALOG ════════════════════════════════════════════════════
   requestStatus (only meaningful when !registered, for non-admins):
     'none' · 'pending' · 'rejected'
   instance.status: 'connected' | 'pending' (oauth not yet authorized)
   authConfigs carry the UNION of fields every page needs:
     oauth → regType · authEndpoint · tokenEndpoint · scopes
     key   → injectVia · keyName · keyPlaceholder
   (all share type · name · detail) */
const CATALOG = [
  { id:'notion', name:'Notion', publisher:'Notion Labs', verified:true, icon:'N', iconBg:'#000', iconColor:'#fff',
    category:'Productivity', catClass:'tag-lotus',
    desc:'Search, read and write pages & databases across your Notion workspace.',
    url:'https://mcp.notion.com/mcp', transport:'streamable-http',
    registered:true, disabled:false, requestStatus:'none', auth:['oauth'],
    userInstances:[{ id:'inst-notion-1', name:'notion', status:'connected', authType:'oauth', authName:'oauth-default', time:'yesterday' }],
    authConfigs:[
      { type:'oauth', name:'oauth-default', regType:'dcr', detail:'Dynamic registration · mcp.notion.com',
        authEndpoint:'https://mcp.notion.com/authorize', tokenEndpoint:'https://mcp.notion.com/token', scopes:'' }] },

  { id:'linear', name:'Linear', publisher:'Linear', verified:true, icon:'L', iconBg:'#5E6AD2', iconColor:'#fff',
    category:'Productivity', catClass:'tag-lotus',
    desc:'Manage issues, projects, cycles, and comments across your Linear workspace.',
    url:'https://mcp.linear.app/mcp', transport:'streamable-http',
    registered:true, disabled:false, requestStatus:'none', auth:['oauth'],
    userInstances:[],
    authConfigs:[
      { type:'oauth', name:'oauth-default', regType:'dcr', detail:'Dynamic registration · mcp.linear.app',
        authEndpoint:'https://mcp.linear.app/authorize', tokenEndpoint:'https://mcp.linear.app/token', scopes:'' }] },

  { id:'github', name:'GitHub', publisher:'GitHub', verified:true, icon:'G', iconBg:'#24292e', iconColor:'#fff',
    category:'Dev Tools', catClass:'tag-indigo',
    desc:'Manage repos, issues, PRs, workflows, and Actions — official MCP server.',
    url:'https://mcp.github.com/mcp', transport:'streamable-http',
    registered:true, disabled:false, requestStatus:'none', auth:['oauth','key'],
    userInstances:[],
    authConfigs:[
      { type:'oauth', name:'oauth-default', regType:'dcr', detail:'Dynamic registration · mcp.github.com',
        authEndpoint:'https://mcp.github.com/authorize', tokenEndpoint:'https://mcp.github.com/token', scopes:'repo read:org' },
      { type:'key', name:'pat-header', injectVia:'header', keyName:'Authorization', detail:'Header: Authorization', keyPlaceholder:'Bearer ghp_…' }] },

  { id:'exa', name:'Exa Search', publisher:'Exa Labs', verified:true, icon:'E', iconBg:'#1C1C1C', iconColor:'#fff',
    category:'Search & Web', catClass:'tag-saffron',
    desc:'Fast, intelligent web search and crawling — Exa-code context tool for coding.',
    url:'https://mcp.exa.ai/mcp', transport:'streamable-http',
    registered:true, disabled:false, requestStatus:'none', auth:['key'],
    userInstances:[{ id:'inst-exa-1', name:'exa', status:'connected', authType:'key', authName:'apikey-default', time:'3h ago' }],
    authConfigs:[
      { type:'key', name:'apikey-default', injectVia:'header', keyName:'x-api-key', detail:'Header: x-api-key', keyPlaceholder:'exa-sk-…' },
      { type:'key', name:'apikey-readonly', injectVia:'query', keyName:'apiKey', detail:'Query: apiKey (read-only)', keyPlaceholder:'exa-ro-…' }] },

  { id:'supabase', name:'Supabase', publisher:'Supabase', verified:true, icon:'▲', iconBg:'#3ECF8E', iconColor:'#000',
    category:'Data', catClass:'tag-leaf',
    desc:'Search Supabase docs, troubleshoot errors, and manage projects & schema.',
    url:'https://mcp.supabase.com/mcp', transport:'streamable-http',
    registered:true, disabled:false, requestStatus:'none', auth:['key'],
    userInstances:[],
    authConfigs:[
      { type:'key', name:'apikey-default', injectVia:'header', keyName:'Authorization', detail:'Header: Authorization', keyPlaceholder:'Bearer sbp_…' }] },

  { id:'deepwiki', name:'DeepWiki', publisher:'Dexa', verified:true, icon:'D', iconBg:'#6C47FF', iconColor:'#fff',
    category:'Dev Tools', catClass:'tag-indigo',
    desc:'Ask any question about GitHub repos — instant answers from code and docs.',
    url:'https://mcp.deepwiki.com/mcp', transport:'streamable-http',
    registered:true, disabled:false, requestStatus:'none', auth:['none'],
    userInstances:[{ id:'inst-dw-1', name:'deepwiki', status:'connected', authType:'none', authName:'public', time:'2d ago' }],
    authConfigs:[] },

  { id:'slack', name:'Slack', publisher:'Slack', verified:true, icon:'S', iconBg:'#4A154B', iconColor:'#fff',
    category:'Comms', catClass:'tag-indigo',
    desc:'Channel-based messaging: post, search, react across your Slack workspace.',
    url:'https://mcp.slack.com/mcp', transport:'streamable-http',
    registered:true, disabled:false, requestStatus:'none', auth:['oauth'],
    userInstances:[{ id:'inst-slack-1', name:'slack', status:'pending', authType:'oauth', authName:'oauth-default', time:'1d ago' }],
    authConfigs:[
      { type:'oauth', name:'oauth-default', regType:'dcr', detail:'Dynamic registration · mcp.slack.com',
        authEndpoint:'https://mcp.slack.com/authorize', tokenEndpoint:'https://mcp.slack.com/token', scopes:'' }] },

  { id:'context7', name:'Context7', publisher:'Upstash', verified:true, icon:'○', iconBg:'#6366F1', iconColor:'#fff',
    category:'Memory', catClass:'tag-indigo',
    desc:'Fetch up-to-date, version-specific docs and code examples into your prompts.',
    url:'https://mcp.context7.com/mcp', transport:'streamable-http',
    registered:true, disabled:true, requestStatus:'none', auth:['none'],
    userInstances:[], authConfigs:[] },

  { id:'gmail', name:'Gmail', publisher:'Google', verified:true, icon:'G', iconBg:'#EA4335', iconColor:'#fff',
    category:'Comms', catClass:'tag-indigo',
    desc:'Send, draft, reply, forward, and bulk-modify messages and threads in Gmail.',
    url:'https://mcp.google.com/gmail', transport:'streamable-http',
    registered:false, disabled:false, requestStatus:'pending', auth:['oauth'],
    userInstances:[], authConfigs:[] },

  { id:'gsheets', name:'Google Sheets', publisher:'Google', verified:true, icon:'⊞', iconBg:'#1E8E3E', iconColor:'#fff',
    category:'Data', catClass:'tag-leaf',
    desc:'Read, write and format spreadsheet data; manage sheets and collaborate.',
    url:'https://mcp.google.com/sheets', transport:'streamable-http',
    registered:false, disabled:false, requestStatus:'rejected', auth:['oauth'],
    userInstances:[], authConfigs:[] },

  { id:'playwright', name:'Playwright', publisher:'Microsoft', verified:true, icon:'▷', iconBg:'#2B2B2B', iconColor:'#45BA4B',
    category:'Browser', catClass:'tag-teal',
    desc:'Browser automation: click, fill, screenshot, assert across Chrome/FF/Safari.',
    url:'https://mcp.playwright.dev/mcp', transport:'streamable-http',
    registered:false, disabled:false, requestStatus:'none', auth:['key'],
    userInstances:[], authConfigs:[] },
];

/* INITIAL_SERVERS — alias kept for the Explore / My MCPs app. */
const INITIAL_SERVERS = CATALOG;
const getServer = id => CATALOG.find(s => s.id === id) || null;

/* Registered servers, keyed by id — the Configure-server page reads this. */
const KNOWN_SERVERS = Object.fromEntries(
  CATALOG.filter(s => s.registered).map(s => [s.id, {
    name: s.name, url: s.url, desc: s.desc, enabled: !s.disabled,
    authConfigs: s.authConfigs.map(c => ({ type: c.type, name: c.name, detail: c.detail })),
  }])
);

/* Servers a user can actually create an instance against (registered & on). */
const CONNECTABLE_SERVERS = CATALOG.filter(s => s.registered && !s.disabled).map(s => ({
  id: s.id, name: s.name, publisher: s.publisher, icon: s.icon, iconBg: s.iconBg, iconColor: s.iconColor,
  url: s.url, authConfigs: s.authConfigs,
}));

/* ══ AUTH VOCABULARY ════════════════════════════════════════════ */
const AUTH_META = {
  oauth: { icon:'lock',   cls:'auth-oauth', label:'OAuth',   iconBg:'var(--c-indigo-bg)',  iconColor:'var(--c-indigo-text)' },
  key:   { icon:'key',    cls:'auth-key',   label:'API Key', iconBg:'var(--c-saffron-bg)', iconColor:'var(--c-saffron-text)' },
  none:  { icon:'unlock', cls:'auth-none',  label:'Public',  iconBg:'var(--c-leaf-bg)',    iconColor:'var(--c-leaf-text)' },
};
const TRANSPORT_LABEL = { 'streamable-http':'Streamable HTTP', 'sse':'SSE (deprecated)', 'stdio':'stdio' };
const PUBLIC_AC = { type:'none', name:'public', detail:'No authentication required', builtin:true };

/* One badge (by type) and a row of badges (from an array of type strings). */
function AuthBadge({ type }) {
  const m = AUTH_META[type] || AUTH_META.none;
  return <span className={'auth-badge ' + m.cls}><Ic name={m.icon} size={10} />{m.label}</span>;
}
function AuthBadges({ auths }) {
  return (auths || []).map(a => <AuthBadge key={a} type={a} />);
}
const CatBadge = ({ s }) => <span className={'tag ' + s.catClass} style={{ fontSize: 10.5, padding: '2px 8px' }}>{s.category}</span>;

/* ══ AUTH MECHANISM LISTS ═══════════════════════════════════════
   Public is always available (no DB row). Two orderings, by UI intent:
     availableAuth  → Public FIRST  (Explore rail "Connect with" list)
     connectMechs   → Public LAST   (New-instance auth dropdown) */
function availableAuth(s) {
  const explicit = (s.authConfigs || []).filter(c => c.type !== 'none');
  return [{ ...PUBLIC_AC }, ...explicit];
}
function connectMechs(s) {
  if (!s) return [];
  const explicit = (s.authConfigs || []).filter(c => c.type !== 'none');
  return [...explicit, { ...PUBLIC_AC }];
}

/* ══ STATUS HELPERS ═════════════════════════════════════════════ */
const connectedInstances = s => s.userInstances.filter(i => i.status === 'connected');

/* left-border accent on list rows */
function statusClass(s) {
  if (s.disabled) return 'status-disabled';
  if (s.registered && connectedInstances(s).length) return 'status-connected';
  if (!s.registered && s.requestStatus === 'pending') return 'status-pending';
  return '';
}

/* Status indicator (NOT an action) — shown on rows. Actions live in the rail. */
function StatusLine({ s, role }) {
  const isAdmin = role === 'admin';
  if (s.disabled) return <span className="card-status-line" style={{ color: 'hsl(var(--muted-foreground))' }}><Ic name="ban" size={11} />Disabled by admin</span>;
  if (!s.registered) {
    if (s.requestStatus === 'pending') return <span className="card-status-line" style={{ color: 'var(--c-pending-text)' }}><Ic name="clock" size={11} />Approval pending</span>;
    if (s.requestStatus === 'rejected') return <span className="card-status-line" style={{ color: 'var(--c-saffron-text)' }}><Ic name="x-circle" size={11} />Request declined</span>;
    return <span className="card-status-line"><Ic name={isAdmin ? 'settings-2' : 'minus-circle'} size={11} />{isAdmin ? 'Not configured' : 'Not in this workspace'}</span>;
  }
  const conn = connectedInstances(s).length;
  if (conn) return <span className="card-status-line" style={{ color: 'var(--c-connected-text)' }}><Ic name="circle-check" size={11} />{conn} instance{conn > 1 ? 's' : ''}</span>;
  if (s.userInstances.some(i => i.status === 'pending')) return <span className="card-status-line" style={{ color: 'var(--c-pending-text)' }}><Ic name="clock" size={11} />Authorizing…</span>;
  return <span className="card-status-line status-approved"><Ic name="plug" size={11} />Available</span>;
}

const slugify = str => str.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');

/* ══ PLAYGROUND TOOL SPECS ══════════════════════════════════════
   Tool surface per server (mock). toolsFor(id) falls back to a generic
   set so the playground always has something to show. */
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
const toolsFor = id => SERVER_TOOLS[id] || DEFAULT_TOOLS;

/* ══ NAVIGATION ═════════════════════════════════════════════════ */
function goToPlayground(instId, instName, serverId) {
  window.location.href = `Bodhi MCP Playground.html?instance=${instId}&name=${encodeURIComponent(instName)}&server=${serverId}`;
}
function goToNewInstance(serverId, authName) {
  window.location.href = `Bodhi MCP New Instance.html?server=${serverId}&auth=${encodeURIComponent(authName || 'public')}`;
}
function goToEditInstance(inst, serverId) {
  const p = new URLSearchParams({ instance: inst.id, edit: '1', server: serverId, name: inst.name, auth: inst.authName || 'public' });
  window.location.href = 'Bodhi MCP New Instance.html?' + p.toString();
}
function goToNewServer(params) {
  const q = new URLSearchParams(params || {}).toString();
  window.location.href = 'Bodhi MCP New Server.html' + (q ? '?' + q : '');
}
function goToViewServer(serverId) {
  window.location.href = 'Bodhi MCP Server.html?server=' + serverId;
}

Object.assign(window, {
  Ic, CATALOG, INITIAL_SERVERS, getServer, KNOWN_SERVERS, CONNECTABLE_SERVERS,
  AUTH_META, TRANSPORT_LABEL, PUBLIC_AC, AuthBadge, AuthBadges, CatBadge,
  availableAuth, connectMechs, connectedInstances, statusClass, StatusLine, slugify,
  SERVER_TOOLS, DEFAULT_TOOLS, toolsFor,
  goToPlayground, goToNewInstance, goToEditInstance, goToNewServer, goToViewServer,
});
