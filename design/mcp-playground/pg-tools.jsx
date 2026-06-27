/* ═══════════════════════════════════════════════════════════════
   BODHI MCP — PLAYGROUND TOOL MODEL  ·  rich tools + behaviour hints
   mcp/pg-tools.jsx   (load AFTER mcp-catalog.jsx, BEFORE pg-data.jsx)

   Phase 1 makes Tools the centrepiece. This file adds, on top of the
   catalog's base tool specs:

     • behaviour HINTS  — the MCP annotation row (readOnly / destructive /
       idempotent / openWorld) re-expressed as friendly, reassuring labels
       with the protocol term tucked into a tooltip.
     • a richer RESULT model — a tool returns an ordered list of content
       blocks (markdown/plain/pre text, image, resource link, embedded
       resource) plus optional structured data, so the readable renderer
       can show whatever an MCP sends back the way a human wants to read it.
     • the EVERYTHING reference server's tools (exercise every result kind
       and every annotation), and light ENRICHment (titles + annotations +
       a few rich results) layered over the real servers.

   Pure data + helpers — published to window; consumed by pg-views.
═══════════════════════════════════════════════════════════════ */

/* ── behaviour hints (friendly label · protocol term in the tooltip) ── */
const HINT_ORDER = ['readOnlyHint', 'destructiveHint', 'idempotentHint', 'openWorldHint'];
function hintFor(key, val) {
  switch (key) {
    case 'readOnlyHint': return val
      ? { key, label: 'Read-only',      tone: 'safe', icon: 'eye',          term: 'readOnlyHint: true',     tip: 'Only reads data — it won’t change anything.' }
      : { key, label: 'Makes changes',  tone: 'warn', icon: 'pencil-line',  term: 'readOnlyHint: false',    tip: 'Can create, update or change data.' };
    case 'destructiveHint': return val
      ? { key, label: 'Can delete',     tone: 'danger', icon: 'trash-2',     term: 'destructiveHint: true',  tip: 'May remove or overwrite existing data.' }
      : { key, label: 'Non-destructive', tone: 'safe',  icon: 'shield-check', term: 'destructiveHint: false', tip: 'Additive only — nothing gets removed.' };
    case 'idempotentHint': return val
      ? { key, label: 'Safe to repeat', tone: 'safe', icon: 'repeat',        term: 'idempotentHint: true',   tip: 'Running it again with the same input has the same effect.' }
      : { key, label: 'Repeats add up', tone: 'warn', icon: 'repeat-2',      term: 'idempotentHint: false',  tip: 'Each run may have an additional effect.' };
    case 'openWorldHint': return val
      ? { key, label: 'Reaches out',    tone: 'info', icon: 'globe',         term: 'openWorldHint: true',    tip: 'Talks to systems beyond this workspace (e.g. the internet).' }
      : { key, label: 'Stays in workspace', tone: 'muted', icon: 'lock',     term: 'openWorldHint: false',   tip: 'Only touches this workspace’s own data.' };
  }
  return null;
}
function hintsForTool(tool) {
  const a = tool && tool.annotations;
  if (!a) return [];
  return HINT_ORDER.filter(k => k in a).map(k => hintFor(k, a[k])).filter(Boolean);
}

/* ── result model ─────────────────────────────────────────────────
   A tool's readable result is { content:[block…], structuredContent?, isError? }.
   block.type ∈ text | image | resource_link | resource
     text   → { text, format?: 'markdown'(default) | 'plain' | 'pre' }
     image  → { name, mimeType, w?, h?, alt?, tint? }   (placeholder tile)
     link   → { uri, name, description?, mimeType? }     (jumps to Resources)
     resource → { uri, mimeType, text }                  (embedded inline) */
function toolResultModel(tool) {
  if (tool.result) return tool.result;
  let parsed; try { parsed = JSON.parse(tool.mockResponse); } catch (e) { parsed = tool.mockResponse; }
  if (typeof parsed === 'string') return { content: [{ type: 'text', text: parsed }] };
  if (Array.isArray(parsed) && parsed.every(b => b && b.type === 'text'))
    return { content: parsed.map(b => ({ type: 'text', text: b.text })) };
  if (parsed && parsed.type === 'text') return { content: [{ type: 'text', text: parsed.text }] };
  return { content: [], structuredContent: parsed };
}
/* the wire-shape envelope shown under Raw (mirrors a real tools/call result) */
function blockToWire(b) {
  if (b.type === 'text') return { type: 'text', text: b.text };
  if (b.type === 'image') return { type: 'image', mimeType: b.mimeType || 'image/png', data: b.data || 'iVBORw0KGgoAAAANSUhEUg…(base64)' };
  if (b.type === 'resource_link') return { type: 'resource_link', uri: b.uri, name: b.name, description: b.description, mimeType: b.mimeType };
  if (b.type === 'resource') return { type: 'resource', resource: { uri: b.uri, mimeType: b.mimeType, text: b.text } };
  return b;
}
function toolResultEnvelope(model) {
  const out = { content: (model.content || []).map(blockToWire) };
  if (model.structuredContent !== undefined) out.structuredContent = model.structuredContent;
  if (model.isError) out.isError = true;
  return out;
}

/* ── EVERYTHING reference server — one tool per result kind / annotation ── */
const EVERYTHING_TOOLS = [
  { name: 'read_document', title: 'Read a document', desc: 'Read a file from the reference workspace and return it as formatted Markdown.',
    params: [{ name: 'path', type: 'string', required: true, desc: 'Path of the file to read', placeholder: 'guides/getting-started.md' }],
    annotations: { readOnlyHint: true, idempotentHint: true, openWorldHint: false },
    result: { content: [{ type: 'text', format: 'markdown', text:
'# Getting started\n\nWelcome to the **Everything** reference server. It exists to show what a connected MCP can do — without you writing any code.\n\n## What you can try\n- **Tools** — actions you can run, like this one\n- **Resources** — data you can read\n- **Prompts** — ready-made requests\n\n> Tip: turn on **Developer** in the header to see the exact request and raw response.\n\n```js\nconst result = await mcp.call("read_document", { path });\n```\n\n| Capability | What it is |\n| --- | --- |\n| Tools | Actions you can run |\n| Resources | Data you can read |\n| Prompts | Ready-made requests |\n\nLearn more at [the docs](https://modelcontextprotocol.io).' }] } },

  { name: 'tail_logs', title: 'View recent logs', desc: 'Return the last lines from the server log as plain text.',
    params: [{ name: 'lines', type: 'number', required: false, desc: 'How many lines to return (default 12)', placeholder: '12' }],
    annotations: { readOnlyHint: true, openWorldHint: false },
    result: { content: [{ type: 'text', format: 'pre', text:
'2026-06-27 13:24:01  INFO   server started, listening on :3001\n2026-06-27 13:24:02  INFO   loaded 7 tools, 4 prompts, 6 resources\n2026-06-27 13:24:09  DEBUG  tools/list  (12ms)\n2026-06-27 13:25:14  INFO   tools/call read_document path="guides/getting-started.md"  (31ms)\n2026-06-27 13:25:14  DEBUG  resources/read demo://docs/architecture.md  (8ms)\n2026-06-27 13:26:40  WARN   rate limit at 80% for client "playground"\n2026-06-27 13:26:41  INFO   tools/call tail_logs  (4ms)' }] } },

  { name: 'list_models', title: 'List available models', desc: 'Return the catalog of models this server knows about, as structured data.',
    params: [],
    annotations: { readOnlyHint: true, idempotentHint: true, openWorldHint: false },
    result: { content: [{ type: 'text', text: 'Found **4 models** in the catalog.' }],
      structuredContent: { models: [
        { id: 'llama-3.1-8b', family: 'Llama', context: 128000, local: true },
        { id: 'qwen-2.5-7b',  family: 'Qwen',  context: 32768,  local: true },
        { id: 'gpt-4o-mini',  family: 'GPT',   context: 128000, local: false },
        { id: 'claude-3.5',   family: 'Claude', context: 200000, local: false },
      ] } } },

  { name: 'make_thumbnail', title: 'Make a thumbnail', desc: 'Render a small preview image for a web page and return it inline.',
    params: [{ name: 'url', type: 'string', required: true, desc: 'Page to render', placeholder: 'https://modelcontextprotocol.io' }],
    annotations: { readOnlyHint: true, openWorldHint: true },
    result: { content: [
      { type: 'text', text: 'Here’s a preview of that page:' },
      { type: 'image', name: 'thumbnail.png', mimeType: 'image/png', w: 280, h: 168, alt: 'Rendered page preview', tint: 'indigo' },
    ] } },

  { name: 'find_resources', title: 'Find related resources', desc: 'Search the workspace and return links to resources that match.',
    params: [{ name: 'query', type: 'string', required: true, desc: 'What to look for', placeholder: 'architecture' }],
    annotations: { readOnlyHint: true, openWorldHint: false },
    result: { content: [
      { type: 'text', text: 'Found **3 resources** that match your search:' },
      { type: 'resource_link', uri: 'demo://docs/architecture.md', name: 'architecture.md', mimeType: 'text/markdown', description: 'System architecture overview' },
      { type: 'resource_link', uri: 'demo://docs/how-it-works.md', name: 'how-it-works.md', mimeType: 'text/markdown', description: 'How the server processes a request' },
      { type: 'resource_link', uri: 'demo://data/structure.json', name: 'structure.json', mimeType: 'application/json', description: 'Project structure as JSON' },
    ] } },

  { name: 'create_note', title: 'Create a note', desc: 'Create a new note in the workspace and return a link to it.',
    params: [{ name: 'title', type: 'string', required: true, desc: 'Note title', placeholder: 'Ideas for launch' },
             { name: 'body', type: 'string', required: false, desc: 'Optional starting text', placeholder: 'First thoughts…' }],
    annotations: { readOnlyHint: false, destructiveHint: false, idempotentHint: false, openWorldHint: false },
    result: { content: [
      { type: 'text', text: 'Note created.' },
      { type: 'resource_link', uri: 'demo://notes/note-8842', name: 'Ideas for launch', mimeType: 'application/json', description: 'Open the note you just created' },
    ], structuredContent: { note: { id: 'note-8842', title: 'Ideas for launch', created: '2026-06-27T13:30:00Z', words: 0 } } } },

  { name: 'delete_note', title: 'Delete a note', desc: 'Permanently delete a note by its id.',
    params: [{ name: 'id', type: 'string', required: true, desc: 'Note id to delete', placeholder: 'note-8842' }],
    annotations: { readOnlyHint: false, destructiveHint: true, idempotentHint: true, openWorldHint: false },
    result: { content: [{ type: 'text', text: 'Note **note-8842** was deleted. This can’t be undone.' }] } },
];

/* ── enrichment for the real servers (titles + annotations + a few results) ── */
const TOOL_ENRICH = {
  deepwiki: {
    read_wiki_structure: { title: 'Browse documentation', annotations: { readOnlyHint: true, idempotentHint: true, openWorldHint: true } },
    read_wiki_contents:  { title: 'Read a doc page',      annotations: { readOnlyHint: true, idempotentHint: true, openWorldHint: true } },
    ask_question:        { title: 'Ask about a repo',     annotations: { readOnlyHint: true, openWorldHint: true },
      result: { content: [{ type: 'text', format: 'markdown', text:
'The React **reconciler** decides what to change in the UI by *diffing* the new element tree against the previous one.\n\n- It walks the tree and marks the work to do\n- Work is split into small units (**fibers**) so rendering can pause and resume\n- A final **commit** phase applies all the DOM changes in one pass\n\nThis is what makes updates feel fast even on large screens.' }] } },
  },
  notion: {
    'notion-search': { title: 'Search Notion', annotations: { readOnlyHint: true, openWorldHint: false },
      result: { content: [{ type: 'text', text: 'Found **2 results** in your workspace.' }],
        structuredContent: { results: [
          { id: 'abc123', title: 'Q1 Project Notes', type: 'page', edited: '2 days ago' },
          { id: 'def456', title: 'Meeting Notes',    type: 'page', edited: '5 days ago' },
        ] } } },
    'notion-fetch':        { title: 'Open a page',     annotations: { readOnlyHint: true, openWorldHint: false } },
    'notion-create-pages': { title: 'Create pages',    annotations: { readOnlyHint: false, destructiveHint: false, idempotentHint: false, openWorldHint: false },
      result: { content: [
        { type: 'text', text: 'Created **My new page**.' },
        { type: 'resource_link', uri: 'notion://page/new-page-id', name: 'My new page', mimeType: 'application/json', description: 'Open the page you just created' },
      ], structuredContent: { id: 'new-page-id', url: 'https://notion.so/My-new-page', created: '2026-06-27T14:00:00Z' } } },
    'notion-update-page':  { title: 'Update a page',   annotations: { readOnlyHint: false, destructiveHint: false, idempotentHint: true, openWorldHint: false } },
    'notion-delete':       { title: 'Delete or archive', annotations: { readOnlyHint: false, destructiveHint: true, idempotentHint: true, openWorldHint: false } },
  },
  exa: {
    'exa-search': { title: 'Search the web', annotations: { readOnlyHint: true, openWorldHint: true },
      result: { content: [{ type: 'text', text: 'Top results for your search:' }],
        structuredContent: { results: [
          { title: 'GPT-5 Benchmark Results',     url: 'https://arxiv.org/abs/2405.0001', score: 0.97 },
          { title: 'Open LLM Leaderboard 2026',   url: 'https://huggingface.co/blog/evals', score: 0.94 },
        ] } } },
    'exa-get-contents': { title: 'Read page contents', annotations: { readOnlyHint: true, openWorldHint: true } },
    'exa-find-similar': { title: 'Find similar pages', annotations: { readOnlyHint: true, openWorldHint: true } },
  },
};

/* ── the playground's enriched tool list for a server ── */
function playgroundToolsFor(serverId) {
  if (serverId === 'everything') return EVERYTHING_TOOLS;
  const base = (typeof toolsFor === 'function') ? toolsFor(serverId) : [];
  const en = TOOL_ENRICH[serverId] || {};
  return base.map(t => {
    const e = en[t.name];
    if (!e) return t;
    return { ...t, ...e, annotations: e.annotations || t.annotations };
  });
}

Object.assign(window, {
  HINT_ORDER, hintFor, hintsForTool,
  toolResultModel, toolResultEnvelope, blockToWire,
  EVERYTHING_TOOLS, TOOL_ENRICH, playgroundToolsFor,
});
