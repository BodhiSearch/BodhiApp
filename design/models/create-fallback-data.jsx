/* ═══════════════════════════════════════════════════
   Create Model Router — data + badge primitives
   models/create-fallback-data.jsx   (load 1st of the cfm modules)

   Mock alias catalog + shared atoms (icon helper, type badge,
   provider badge). Everything is stashed on window.CFM so the other
   create-fallback-*.jsx modules can pull what they need with a single
   destructure at the top of each file.
═══════════════════════════════════════════════════ */
const CFM = (window.CFM = window.CFM || {});

/* ── Available aliases ────────────────────────────
   In the real app this comes from the API; mocked here.
   For api-models the `fwdMode` field mirrors how the
   API-model alias was configured ("selected" or "all"),
   determining whether the model field below is a
   constrained dropdown or a free-text autocomplete.
─────────────────────────────────────────────────── */
const AVAILABLE_ALIASES = [
  { id:'afrideva/Llama-68M-Chat:Q8_0',     type:'local-file',  display:'afrideva/Llama-68M-Chat:Q8_0',     size:'0.07 GB' },
  { id:'Qwen/Qwen3-Coder-32B:Q4_K_M',      type:'local-file',  display:'Qwen/Qwen3-Coder-32B:Q4_K_M',      size:'18.5 GB' },
  { id:'meta-llama/Llama-3.3-70B:Q4_K_M',  type:'local-file',  display:'meta-llama/Llama-3.3-70B:Q4_K_M',  size:'35.0 GB' },
  { id:'my-qwen-coder',                    type:'model-alias', display:'my-qwen-coder', backing:'Qwen/Qwen3-Coder-32B:Q4_K_M' },
  { id:'01kp50czqbcgnhnwtnv7jq2s',         type:'api-model',   display:'01kp50czqbcgnhnwtnv7jq2s',  provider:'ANTHROPIC',       fwdMode:'selected', models:['claude-sonnet-4-5'] },
  { id:'01kp506g2crx8pgqtp4ts1jfh7',       type:'api-model',   display:'01kp506g2crx8pgqtp4ts1jfh7', provider:'ANTHROPIC_OAUTH', fwdMode:'selected', models:['claude-opus-4'] },
  { id:'openai-gpt-main',                  type:'api-model',   display:'openai-gpt-main',           provider:'OPENAI',          fwdMode:'selected', models:['gpt-5','gpt-4o','gpt-4o-mini'] },
  { id:'openrouter-all',                   type:'api-model',   display:'openrouter-all',            provider:'OPENROUTER',      fwdMode:'all',      models:[] },
];

/* Broader autocomplete pool for api-models with fwdMode:'all' */
const ALL_KNOWN_MODELS = [
  'gpt-4o','gpt-4o-mini','gpt-4-turbo','gpt-3.5-turbo','o1','o1-mini','o3','o3-mini',
  'claude-opus-4','claude-sonnet-4-5','claude-haiku-3-5','claude-3-5-sonnet','claude-3-opus',
  'gemini-2.0-flash','gemini-1.5-pro','gemini-1.5-flash',
  'llama-3.3-70b-versatile','mixtral-8x7b-32768','gemma2-9b-it',
  'deepseek-v3','deepseek-r1',
  'qwen-plus','qwen-turbo',
];

/* Type-badge style table */
const TYPE_CFG = {
  'local-file':  { bg:'var(--c-saffron-bg)', bd:'var(--c-saffron-bd)', text:'var(--c-saffron-text)', icon:'hard-drive', label:'Local File' },
  'model-alias': { bg:'var(--c-lotus-bg)',   bd:'var(--c-lotus-bd)',   text:'var(--c-lotus-text)',   icon:'tag',        label:'Model Alias' },
  'api-model':   { bg:'var(--c-indigo-bg)',  bd:'var(--c-indigo-bd)',  text:'var(--c-indigo-text)',  icon:'at-sign',    label:'API Model' },
};

/* ── Lucide icon helper ─────────────────── */
function Icon({ name, size = 13, style = {} }) {
  const ref = React.useRef(null);
  React.useEffect(() => {
    if (!ref.current) return;
    ref.current.innerHTML = '';
    const el = document.createElement('i');
    el.setAttribute('data-lucide', name);
    ref.current.appendChild(el);
    if (typeof lucide !== 'undefined') lucide.createIcons({ nodes: [el] });
  }, [name]);
  return (
    <span
      ref={ref}
      style={{
        display:'inline-flex', width:size, height:size,
        alignItems:'center', justifyContent:'center', flexShrink:0,
        ...style,
      }}
    />
  );
}

/* ── Type badge ─────────────────────────── */
function TypeBadge({ type, small = false }) {
  const cfg = TYPE_CFG[type];
  if (!cfg) return null;
  const padding = small ? '1px 5px' : '2px 7px';
  const fontSize = small ? 9.5 : 10;
  const iconSize = small ? 8 : 9;
  return (
    <span style={{
      display:'inline-flex', alignItems:'center', gap:4,
      padding, borderRadius:99,
      fontSize, fontWeight:600,
      background:cfg.bg, border:`1px solid ${cfg.bd}`, color:cfg.text,
      whiteSpace:'nowrap',
    }}>
      <Icon name={cfg.icon} size={iconSize} />
      {cfg.label}
    </span>
  );
}

/* ── Provider badge ─────────────────────── */
function ProviderBadge({ provider }) {
  if (!provider) return null;
  return (
    <span style={{
      fontSize:10, fontWeight:700, padding:'2px 6px', borderRadius:4,
      background:'var(--c-leaf-bg)', color:'var(--c-leaf-text)',
      border:'1px solid var(--c-leaf-bd)', whiteSpace:'nowrap',
      letterSpacing:.02,
    }}>{provider}</span>
  );
}

Object.assign(CFM, { AVAILABLE_ALIASES, ALL_KNOWN_MODELS, TYPE_CFG, Icon, TypeBadge, ProviderBadge });
