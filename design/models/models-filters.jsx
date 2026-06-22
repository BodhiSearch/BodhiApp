/* ═══════════════════════════════════════════════════════════════
   Bodhi Models — sidebar filters
   models/models-filters.jsx   (load after models-base.jsx)

   Per-mode filter definitions and the ModelsSidebar that renders them
   into the shell's `sidebar` slot. The local-explore sidebar also
   stitches in the publisher / browse / context groups that live in
   bodhi-models-local.jsx (only loaded on the Local page).

   Exports: MODE_CFG, FILTERS, PLACEHOLDER, ModelsSidebar
═══════════════════════════════════════════════════════════════ */
const { LOCAL_MODELS } = window.MODELS_DATA;

/* ── Mode config (one fixed mode per page) ────────────────────── */
const MODE_CFG = {
  'my-models': { subPage: 'my-models',     label: 'My Models' },
  'local':     { subPage: 'explore-local', label: 'Explore · Local Models' },
  'api':       { subPage: 'explore-api',   label: 'Explore · API Providers' }
};

/* ── Search placeholders (per mode) ───────────────────────────── */
const PLACEHOLDER = {
  'my-models': 'Search by alias, repo, filename, base URL — ⌘K',
  'local': 'Search HuggingFace repos — ⌘K',
  'api': 'Search API providers — ⌘K'
};

const FILTERS = {
  'my-models': [
  { icon: 'shapes', label: 'Type', chips: [
    { label: 'Local', color: 'saffron' }, { label: 'Alias', color: 'lotus' },
    { label: 'API', color: 'indigo' }, { label: 'Router', color: 'teal' }] },
  { icon: 'sparkles', label: 'Capability', chips: ['chat', 'tool-use', 'vision', 'embeddings', 'reasoning'].map((l) => ({ label: l })) },
  { icon: 'ruler', label: 'Size', note: '(local files)', range: { min: 0, max: 16, step: 1, unit: ' GB', defaultMin: 0, defaultMax: 16 } },
  { icon: 'plug', label: 'API Format', note: '(API only)', chips: ['OpenAI', 'Responses', 'Anthropic', 'Gemini'].map((l) => ({ label: l })) }],

  'local': [
  { icon: 'compass', label: 'Browse', chips: [{ label: '↗ Trending' }, { label: '✦ New' }] },
  { icon: 'list-checks', label: 'Task', chips: [
    { label: 'Text Generation', color: 'indigo', defaultOn: true }, { label: 'Image-Text-to-Text', color: 'lotus' }] },
  { icon: 'target', label: 'Specialisation', clearable: true, chips: [
    { label: 'All', defaultOn: true }, { label: 'Coding', color: 'saffron' },
    { label: 'Reasoning' }, { label: 'Vision' }] },
  { icon: 'hash', label: 'Tag', note: '(advanced)', clearable: true, chips: [
    { label: 'tool-use' }, { label: 'conversational' }, { label: 'thinking' }, { label: 'moe' }, { label: 'embedding' }] },
  { icon: 'languages', label: 'Language', clearable: true, chips: [
    { label: 'en', color: 'leaf' }, { label: 'zh', color: 'leaf' }, { label: 'es', color: 'leaf' }, { label: 'fr', color: 'leaf' }, { label: 'de', color: 'leaf' }, { label: 'ja', color: 'leaf' }, { label: 'ko', color: 'leaf' }] },
  { icon: 'scale', label: 'License', clearable: true, chips: [
    { label: 'Apache-2', color: 'indigo', defaultOn: true }, { label: 'MIT' }, { label: 'Llama' }, { label: 'Gemma' }, { label: 'DeepSeek' }] }],

  'api': [
  { icon: 'activity', label: 'Status', single: true, chips: [
    { label: 'Connected', color: 'leaf' }] },
  { icon: 'sparkles', label: 'Capability', chips: [
    { label: 'tool-use', color: 'indigo' }, { label: 'vision' }, { label: 'reasoning' }, { label: 'structured' }] },
  { icon: 'dollar-sign', label: 'Pricing', note: '($/Mtok)', range: { min: 0, max: 20, step: 0.5, unit: '/M', prefix: '$', defaultMin: 0, defaultMax: 20 } },
  { icon: 'plug', label: 'API Format', chips: [
    { label: 'OpenAI', color: 'indigo' }, { label: 'Responses' }, { label: 'Anthropic' }, { label: 'Gemini' }, { label: 'Other' }] }]

};

function ModelsSidebar({ mode, orgFilters, onPickOrg, onRemoveOrg, onClearOrgs, sort, onBrowse, apiConnectedOnly, onToggleApiConnected }) {
  const { collapsed } = useShell();
  if (mode !== 'local') {
    return (
      <>
        {FILTERS[mode].map((g) =>
        g.label === 'Status' && g.single ?
        <ShellFilterGroup key={mode + '-' + g.label} icon={g.icon} label={g.label} chips={g.chips} single clearable
          value={apiConnectedOnly ? 'Connected' : null} onSelect={() => onToggleApiConnected && onToggleApiConnected()} /> :
        <ShellFilterGroup key={mode + '-' + g.label} icon={g.icon} label={g.label}
        note={g.note} clearable={g.clearable} chips={g.chips} range={g.range} />
        )}
      </>);
  }
  return (
    <>
      {FILTERS.local.map((g) =>
      <React.Fragment key={'local-' + g.label}>
        {g.label === 'Browse'
          ? <BrowseGroup sort={sort} onBrowse={onBrowse} />
          : <ShellFilterGroup icon={g.icon} label={g.label} note={g.note} clearable={g.clearable} chips={g.chips} range={g.range} />}
        {g.label === 'Tag' &&
          <PublisherGroup orgFilters={orgFilters} onPick={onPickOrg} onRemove={onRemoveOrg} />}
      </React.Fragment>
      )}
    </>);
}

Object.assign(window, { MODE_CFG, FILTERS, PLACEHOLDER, ModelsSidebar });
