/* ═══════════════════════════════════════════════════
   Bodhi MCP Discover — data + status helpers
   mcp/mcp-discover-data.jsx   (load 1st of the discover modules)

   The mock server catalog (INITIAL_SERVERS) plus the small shared
   helpers every other discover module leans on: auth badges, category
   badge, per-user status derivation, navigation helpers, and the CTA
   button. Published to window for mcp-discover-cards / -detail /
   -config / the root app.
═══════════════════════════════════════════════════ */
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

Object.assign(window, {
  Ic, INITIAL_SERVERS, AUTH_META, AuthBadges, CatBadge,
  statusClass, userStatusSummary, StatusLine, goToPlayground, goToNewMCP, Cta,
});
