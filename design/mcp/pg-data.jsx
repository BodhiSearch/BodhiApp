/* ═══════════════════════════════════════════════════════════════
   BODHI MCP — PLAYGROUND DATA  ·  capability mock surface
   mcp/pg-data.jsx   (load AFTER mcp-catalog.jsx, BEFORE pg-shared)

   The catalog (mcp-catalog.jsx) owns servers, instances, auth and the
   TOOL specs. This file adds the rest of the MCP capability surface the
   re-imagined Playground browses — PROMPTS, RESOURCES and RESOURCE
   TEMPLATES — plus the per-instance lookup the left rail uses.

   Everything is mock but shaped exactly like the real MCP protocol:
     • prompt   → name · arguments[] · build(vals) → messages[]
     • resource → uri · mimeType · read() → contents
     • template → uriTemplate · arguments[] · resolve(vals) → resource
   Published to window so the page app reads them as globals.
═══════════════════════════════════════════════════════════════ */

/* ── PROMPTS ─────────────────────────────────────────────────── */
const SERVER_PROMPTS = {
  deepwiki: [
    { name: 'explain_repo', title: 'Explain a repository', icon: 'book-open',
      desc: 'Get a plain-language tour of what a GitHub project does and how it is put together.',
      args: [{ name: 'repoName', label: 'Repository', required: true, desc: 'owner/repo on GitHub', placeholder: 'facebook/react' },
             { name: 'audience', label: 'Explain it for', required: false, desc: 'Who is reading — tunes the depth', placeholder: 'a new contributor' }],
      build: v => [
        { role: 'user', content: `Give me a clear, friendly overview of the GitHub repository ${v.repoName || 'facebook/react'}. Explain what it does, the main building blocks, and how they fit together${v.audience ? ` — written for ${v.audience}` : ''}. Avoid jargon where you can.` },
      ] },
    { name: 'compare_repos', title: 'Compare two repositories', icon: 'git-compare',
      desc: 'Side-by-side comparison of two projects — strengths, trade-offs and when to pick each.',
      args: [{ name: 'repoA', label: 'First repository', required: true, desc: 'owner/repo', placeholder: 'vercel/next.js' },
             { name: 'repoB', label: 'Second repository', required: true, desc: 'owner/repo', placeholder: 'remix-run/remix' }],
      build: v => [
        { role: 'user', content: `Compare ${v.repoA || 'vercel/next.js'} and ${v.repoB || 'remix-run/remix'}. Cover what each is best at, the main trade-offs, and a short recommendation on when to choose one over the other.` },
      ] },
  ],
  notion: [
    { name: 'weekly_summary', title: 'Summarize my week', icon: 'calendar-range',
      desc: 'Pull the highlights from this week\u2019s pages into a tidy summary you can share.',
      args: [{ name: 'workspace_area', label: 'Area', required: false, desc: 'Limit to a section of the workspace', placeholder: 'Engineering' }],
      build: v => [
        { role: 'user', content: `Summarize the most important updates from this week\u2019s Notion pages${v.workspace_area ? ` in the ${v.workspace_area} area` : ''}. Group by theme, keep it to a few bullets, and call out anything that needs a decision.` },
      ] },
    { name: 'draft_doc', title: 'Draft a document', icon: 'file-pen-line',
      desc: 'Start a first draft on any topic, in the right shape for the audience you choose.',
      args: [{ name: 'topic', label: 'Topic', required: true, desc: 'What the document is about', placeholder: 'Q3 launch plan' },
             { name: 'audience', label: 'Audience', required: false, desc: 'Who will read it', placeholder: 'leadership' }],
      build: v => [
        { role: 'user', content: `Draft a clear, well-structured document about "${v.topic || 'Q3 launch plan'}"${v.audience ? ` for ${v.audience}` : ''}. Include a short summary, the key sections, and next steps.` },
      ] },
  ],
  exa: [
    { name: 'research_brief', title: 'Research brief', icon: 'telescope',
      desc: 'Search the web and assemble a sourced brief on a topic, ready to skim.',
      args: [{ name: 'topic', label: 'Topic', required: true, desc: 'What to research', placeholder: 'state of AI agents 2026' },
             { name: 'depth', label: 'Depth', required: false, desc: 'quick scan or deep dive', placeholder: 'deep dive' }],
      build: v => [
        { role: 'user', content: `Research "${v.topic || 'state of AI agents 2026'}" on the web and write a ${v.depth || 'concise'} brief. Cite sources inline, lead with the key findings, and flag where sources disagree.` },
      ] },
  ],
};

/* ── RESOURCES ───────────────────────────────────────────────── */
const SERVER_RESOURCES = {
  deepwiki: [
    { uri: 'wiki://facebook/react/overview', name: 'React — Overview', icon: 'file-text', mimeType: 'text/markdown',
      desc: 'The top-level documentation page for facebook/react.',
      contents: '# React\n\nReact is a JavaScript library for building user interfaces. It lets you compose complex UIs from small, isolated pieces of code called \u201ccomponents.\u201d\n\n## Core ideas\n- **Declarative** \u2014 describe what the UI should look like for any given state.\n- **Component-based** \u2014 build encapsulated components that manage their own state.\n- **Learn once, write anywhere** \u2014 the same model powers web and native.' },
    { uri: 'wiki://vercel/next.js/routing', name: 'Next.js — Routing', icon: 'file-text', mimeType: 'text/markdown',
      desc: 'How the App Router maps folders to URLs.',
      contents: '# Routing\n\nNext.js uses a file-system based router. Folders define routes; a `page.tsx` makes a route segment publicly accessible. Nested folders create nested routes, and special files (`layout`, `loading`, `error`) wrap the segment.' },
  ],
  notion: [
    { uri: 'notion://page/q1-project-notes', name: 'Q1 Project Notes', icon: 'file-text', mimeType: 'application/json',
      desc: 'A page from your Notion workspace.',
      contents: { id: 'abc123', title: 'Q1 Project Notes', lastEdited: '2026-06-21T14:00:00Z', owner: 'Yogesh', blocks: 42, tags: ['planning', 'q1'] } },
    { uri: 'notion://database/tasks', name: 'Tasks', icon: 'database', mimeType: 'application/json',
      desc: 'Your Tasks database, with current counts by status.',
      contents: { id: 'db-tasks', title: 'Tasks', rows: 128, statuses: { todo: 31, 'in-progress': 12, done: 85 } } },
  ],
  exa: [
    { uri: 'exa://recent-searches', name: 'Recent searches', icon: 'history', mimeType: 'application/json',
      desc: 'The last few queries run through this instance.',
      contents: [{ query: 'AI agent benchmarks 2026', when: '3h ago' }, { query: 'vector database comparison', when: 'yesterday' }] },
  ],
};

/* ── RESOURCE TEMPLATES ──────────────────────────────────────── */
const SERVER_TEMPLATES = {
  deepwiki: [
    { uriTemplate: 'wiki://{owner}/{repo}/{topic}', name: 'Wiki page', icon: 'file-search', mimeType: 'text/markdown',
      desc: 'Open any documentation page for any repository by filling in the blanks.',
      args: [{ name: 'owner', label: 'Owner', required: true, placeholder: 'facebook' },
             { name: 'repo', label: 'Repository', required: true, placeholder: 'react' },
             { name: 'topic', label: 'Topic', required: true, placeholder: 'hooks' }],
      resolve: v => ({ uri: `wiki://${v.owner}/${v.repo}/${v.topic}`, mimeType: 'text/markdown',
        contents: `# ${v.topic}\n\nDocumentation for **${v.topic}** in ${v.owner}/${v.repo}. This page is resolved on demand from the live repository docs.` }) },
  ],
  notion: [
    { uriTemplate: 'notion://page/{pageId}', name: 'Page by ID', icon: 'file-search', mimeType: 'application/json',
      desc: 'Fetch any page when you already know its Notion ID.',
      args: [{ name: 'pageId', label: 'Page ID', required: true, placeholder: 'abc123def456' }],
      resolve: v => ({ uri: `notion://page/${v.pageId}`, mimeType: 'application/json',
        contents: { id: v.pageId, object: 'page', title: 'Resolved page', url: `https://notion.so/${v.pageId}` } }) },
  ],
  exa: [
    { uriTemplate: 'exa://search/{query}', name: 'Saved search', icon: 'file-search', mimeType: 'application/json',
      desc: 'Turn any query into a re-runnable resource.',
      args: [{ name: 'query', label: 'Query', required: true, placeholder: 'open source LLMs' }],
      resolve: v => ({ uri: `exa://search/${encodeURIComponent(v.query || '')}`, mimeType: 'application/json',
        contents: { query: v.query, results: 10, refreshedeable: true } }) },
  ],
};

const promptsFor   = id => SERVER_PROMPTS[id]   || [];
const resourcesFor = id => SERVER_RESOURCES[id] || [];
const templatesFor = id => SERVER_TEMPLATES[id] || [];

/* Capability counts for a server id — drives the rail badges + Overview. */
function capabilityCounts(serverId) {
  return {
    tools: (typeof toolsFor === 'function' ? toolsFor(serverId) : []).length,
    prompts: promptsFor(serverId).length,
    resources: resourcesFor(serverId).length,
    templates: templatesFor(serverId).length,
  };
}

/* ── INSTANCE LOOKUP ─────────────────────────────────────────────
   Flatten CATALOG.userInstances into the list the left-rail combobox
   shows. Each row carries enough to resolve server + capabilities. */
function playgroundInstances() {
  const out = [];
  (CATALOG || []).forEach(s => {
    (s.userInstances || []).forEach(i => {
      out.push({
        instId: i.id, instName: i.name, status: i.status,
        authType: i.authType, authName: i.authName, time: i.time,
        serverId: s.id, server: s,
      });
    });
  });
  return out;
}
function findInstance(instId, serverId) {
  const all = playgroundInstances();
  return all.find(i => instId && i.instId === instId)
      || all.find(i => serverId && i.serverId === serverId)
      || null;
}

Object.assign(window, {
  SERVER_PROMPTS, SERVER_RESOURCES, SERVER_TEMPLATES,
  promptsFor, resourcesFor, templatesFor, capabilityCounts,
  playgroundInstances, findInstance,
});
