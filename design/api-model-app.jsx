/* ═══════════════════════════════════════════
   CREATE NEW API MODEL — React App v3
   - No sidebar
   - Alias name + chips inline
   - Inline validation (no static hint)
   - Quant download status; no Pull Now
   - System prompt above request params
   - Click-to-add split panels
═══════════════════════════════════════════ */

const API_TWEAK_DEFAULTS = /*EDITMODE-BEGIN*/{
  "theme": "light"
} /*EDITMODE-END*/;

/* ── Quant options ─────────────────────── */
const API_QUANTS = [
{ name: 'Q4_K_M', size: '4.9 GB', bpw: '4.85 bpw', rec: true, status: 'downloaded' },
{ name: 'Q3_K_S', size: '3.7 GB', bpw: '3.50 bpw', rec: false, status: 'downloaded' },
{ name: 'Q5_K', size: '5.7 GB', bpw: '5.69 bpw', rec: false, status: 'partial' },
{ name: 'Q8_0', size: '8.5 GB', bpw: '8.50 bpw', rec: false, status: 'remote' },
{ name: 'F16', size: '16.0 GB', bpw: '16.00 bpw', rec: false, status: 'remote' }];


const STATUS_LABEL = {
  downloaded: 'Downloaded',
  partial: 'Partial',
  remote: 'Not downloaded'
};

/* ── Presets ───────────────────────────── */
const API_PRESETS = [
{ id: 'default', label: 'Default', flags: '--ctx-size 4096\n--flash-attn auto\n--parallel 4\n' },
{ id: 'coding', label: 'Coding', flags: '--ctx-size 8192\n--flash-attn auto\n--parallel 2\n--cache-prompt true\n' },
{ id: 'non-coder', label: 'Non Coder', flags: '--ctx-size 2048\n--parallel 2\n' },
{ id: 'smart', label: 'Smart', flags: '--ctx-size 16384\n--flash-attn auto\n--parallel 8\n--rope-scaling yarn\n' },
{ id: 'translate', label: 'Translate', flags: '--ctx-size 4096\n--parallel 4\n--repeat-penalty 1.1\n' },
{ id: 'rag', label: 'RAG', flags: '--ctx-size 32768\n--flash-attn auto\n--cache-prompt true\n--parallel 6\n' },
{ id: 'kwa-deep', label: 'KWA Deep', flags: '--ctx-size 47536\n--flash-attn auto\n--parallel 10\n--cont-batching true\n--cache-type-k q8_0\n--rope-scaling yarn\n--cache-prompt true\n--grp-attn-n 8\n' }];


/* ── Runtime flags catalogue ───────────── */
const RUNTIME_FLAGS = [
{ key: '--ctx-size', type: 'int', range: '512 – 131072', desc: 'Context window size in tokens.' },
{ key: '--flash-attn', type: 'enum', range: 'auto | true | false', desc: 'Flash attention — faster on supported hardware.' },
{ key: '--parallel', type: 'int', range: '1 – 64', desc: 'Number of parallel request slots.' },
{ key: '--cont-batching', type: 'bool', range: 'true | false', desc: 'Continuous batching for better throughput.' },
{ key: '--cache-type-k', type: 'enum', range: 'f16 | q8_0 | q4_0', desc: 'KV cache quant type for keys. Saves VRAM.' },
{ key: '--cache-type-v', type: 'enum', range: 'f16 | q8_0 | q4_0', desc: 'KV cache quant type for values.' },
{ key: '--rope-scaling', type: 'enum', range: 'none | linear | yarn', desc: 'RoPE scaling for context extension.' },
{ key: '--cache-prompt', type: 'bool', range: 'true | false', desc: 'Cache system prompt KV across requests.' },
{ key: '--grp-attn-n', type: 'int', range: '1 – 16', desc: 'Group attention factor for YaRN.' },
{ key: '--n-predict', type: 'int', range: '-1 – 32768', desc: 'Max tokens to generate. -1 = unlimited.' },
{ key: '--n-batch', type: 'int', range: '1 – 2048', desc: 'Logical batch size for token evaluation.' },
{ key: '--ubatch-size', type: 'int', range: '1 – 2048', desc: 'Physical batch size for prompt processing.' },
{ key: '--keep', type: 'int', range: '0 – ctx-size', desc: 'Tokens kept from initial prompt on reset.' },
{ key: '--mmap', type: 'bool', range: 'true | false', desc: 'Memory-map model file for faster load.' },
{ key: '--mlock', type: 'bool', range: 'true | false', desc: 'Lock model in RAM to prevent swapping.' },
{ key: '--split-mode', type: 'enum', range: 'none | layer | row', desc: 'How to split model across multiple GPUs.' },
{ key: '--n-gpu-layers', type: 'int', range: '0 – max', desc: 'Layers to offload to GPU. -1 = all.' },
{ key: '--main-gpu', type: 'int', range: '0 – N', desc: 'Primary GPU index for tensor ops.' },
{ key: '--repeat-penalty', type: 'float', range: '0.0 – 2.0', desc: 'Penalise repeated tokens. 1.0 = off.' },
{ key: '--top-k', type: 'int', range: '0 – vocab', desc: 'Top-K sampling. 0 = disabled.' },
{ key: '--top-p', type: 'float', range: '0.0 – 1.0', desc: 'Nucleus (top-P) sampling threshold.' },
{ key: '--min-p', type: 'float', range: '0.0 – 1.0', desc: 'Minimum probability for sampling.' },
{ key: '--temperature', type: 'float', range: '0.0 – 2.0', desc: 'Sampling temperature. Lower = deterministic.' },
{ key: '--seed', type: 'int', range: '-1 | 0 – 2³²', desc: 'Random seed. -1 = random.' },
{ key: '--mirostat', type: 'int', range: '0 | 1 | 2', desc: 'Mirostat sampling version.' },
{ key: '--no-warmup', type: 'flag', range: '—', desc: 'Skip warm-up on server start.' },
{ key: '--chat-template', type: 'string', range: 'chatml | llama2 | …', desc: 'Override auto-detected chat template.' }];


/* ── OpenAI compat request params ─────── */
const REQUEST_PARAMS = [
{ key: 'temperature', type: 'float', range: '0.0 – 2.0', desc: 'Sampling temperature for this model.' },
{ key: 'top_p', type: 'float', range: '0.0 – 1.0', desc: 'Nucleus sampling probability mass.' },
{ key: 'max_tokens', type: 'int', range: '1 – ctx-size', desc: 'Maximum tokens in the completion.' },
{ key: 'seed', type: 'int', range: '-1 | int', desc: 'Reproducibility seed. -1 = random.' },
{ key: 'frequency_penalty', type: 'float', range: '-2.0 – 2.0', desc: 'Penalise tokens by frequency so far.' },
{ key: 'presence_penalty', type: 'float', range: '-2.0 – 2.0', desc: 'Penalise tokens that appeared at all.' },
{ key: 'n', type: 'int', range: '1 – 10', desc: 'Completions to generate per prompt.' },
{ key: 'stop', type: 'string', range: 'str | str[]', desc: 'Stop sequences — halt generation on match.' },
{ key: 'response_format', type: 'enum', range: 'text | json_object', desc: 'Force output format.' },
{ key: 'tool_choice', type: 'enum', range: 'none | auto | required', desc: 'Tool/function calling behaviour.' },
{ key: 'stream', type: 'bool', range: 'true | false', desc: 'Stream partial tokens as SSE.' },
{ key: 'logprobs', type: 'bool', range: 'true | false', desc: 'Return log probabilities per token.' },
{ key: 'top_logprobs', type: 'int', range: '0 – 20', desc: 'Top token logprobs to return.' },
{ key: 'suffix', type: 'string', range: '<string>', desc: 'Text appended after completion (FIM).' },
{ key: 'user', type: 'string', range: '<string>', desc: 'End-user ID for abuse tracking.' }];


/* ── Breakpoint hook ───────────────────── */
function useBreakpoint() {
  const get = () => {
    const w = typeof window !== 'undefined' ? window.innerWidth : 1280;
    return w < 640 ? 'mobile' : w < 1024 ? 'tablet' : 'desktop';
  };
  const [bp, setBp] = React.useState(get);
  React.useEffect(() => {
    const h = () => setBp(get());
    window.addEventListener('resize', h);
    return () => window.removeEventListener('resize', h);
  }, []);
  return bp;
}

/* ── Icon ──────────────────────────────── */
function Icon({ name, size = 13, style = {} }) {
  const ref = React.useRef(null);
  React.useEffect(() => {
    if (!ref.current) return;
    ref.current.innerHTML = '';
    const el = document.createElement('i');
    el.setAttribute('data-lucide', name);
    ref.current.appendChild(el);
    lucide.createIcons({ nodes: [el] });
  }, [name]);
  return (
    <span ref={ref} style={{ display: 'inline-flex', width: size, height: size, alignItems: 'center', justifyContent: 'center', flexShrink: 0, ...style }} />);

}

/* ── Inline validation message ─────────── */
function AliasValidation({ value }) {
  if (!value) return (
    <div className="nm-validation nm-validation-hint">
      <Icon name="info" size={11} /> Lowercase, digits, and dashes only.
    </div>);

  const valid = /^[a-z0-9-]+$/.test(value);
  if (valid) return (
    <div className="nm-validation nm-validation-ok">
      <Icon name="check-circle" size={11} /> Looks good.
    </div>);

  return (
    <div className="nm-validation nm-validation-err">
      <Icon name="alert-circle" size={11} /> Only lowercase letters, digits, and dashes allowed.
    </div>);

}

/* ── Click-to-add flag/param panel ─────── */
function FlagPanel({ catalogue, textareaValue, onAdd, label, mode }) {
  const addedKeys = React.useMemo(() => {
    const lines = textareaValue.split('\n');
    const keys = new Set();
    lines.forEach((l) => {
      const t = l.trim();
      if (mode === 'params') {
        const m = t.match(/^([\w_]+)=/);
        if (m) keys.add(m[1]);
      } else {
        const m = t.match(/^(--[\w-]+)/);
        if (m) keys.add(m[1]);
      }
    });
    return keys;
  }, [textareaValue, mode]);

  return (
    <div>
      <div className="nm-split-label">{label}</div>
      <div className="nm-flag-list">
        {catalogue.map((f) => {
          const isAdded = addedKeys.has(f.key);
          return (
            <div
              key={f.key}
              className={`nm-flag-item${isAdded ? ' added' : ''}`}
              onClick={() => !isAdded && onAdd(f)}
              title={isAdded ? 'Already added' : `Click to add ${f.key}`}>
              
              <div className="nm-flag-row">
                <span className="nm-flag-name">{f.key}</span>
                <span className="nm-flag-type">{f.type}</span>
                {isAdded && <span className="nm-flag-added-badge">added</span>}
              </div>
              <div className="nm-flag-desc">{f.desc}</div>
              {f.range && f.range !== '—' &&
              <div className="nm-flag-vals">{f.range}</div>
              }
            </div>);

        })}
      </div>
    </div>);

}

/* ── Main App ──────────────────────────── */
function ApiModelApp() {
  const [tweaks, setTweak] = useTweaks(API_TWEAK_DEFAULTS);

  /* form state */
  const [aliasName, setAliasName] = React.useState('qwen-api');
  const [repo, setRepo] = React.useState('Qwen/Qwen3-8B-GGUF');
  const [snapshot, setSnapshot] = React.useState('(GGUF_PREVIEW)');
  const [selectedQ, setSelectedQ] = React.useState(0);
  const [activePreset, setActivePreset] = React.useState('kwa-deep');

  /* textareas */
  const [runtimeText, setRuntimeText] = React.useState(
    API_PRESETS.find((p) => p.id === 'kwa-deep').flags
  );
  const [reqText, setReqText] = React.useState('temperature=0.7\ntop_p=1.0\nn=1\n');
  const [sysPrompt, setSysPrompt] = React.useState('You are a helpful assistant.');

  /* sync theme */
  React.useEffect(() => {
    document.documentElement.setAttribute('data-theme', tweaks.theme);
  }, [tweaks.theme]);

  /* preset click */
  const applyPreset = (preset) => {
    setActivePreset(preset.id);
    setRuntimeText(preset.flags);
  };

  /* add flag to runtime textarea */
  const addFlag = (flag) => {
    const defaultVal = flag.type === 'bool' ? 'true' :
    flag.type === 'flag' ? '' :
    flag.type === 'int' ? '0' :
    flag.type === 'float' ? '0.0' :
    '<value>';
    const line = flag.type === 'flag' ? `${flag.key}\n` : `${flag.key} ${defaultVal}\n`;
    setRuntimeText((t) => t + line);
  };

  /* add param to req textarea */
  const addParam = (param) => {
    const defaultVal = param.type === 'bool' ? 'false' :
    param.type === 'int' ? '0' :
    param.type === 'float' ? '0.0' :
    '<value>';
    setReqText((t) => t + `${param.key}=${defaultVal}\n`);
  };

  const selQ = API_QUANTS[selectedQ];

  return (
    <div className="nm-app-shell">
      <BodhiSidebar section="models" subPage="new-local-model" />
      <div className="nm-content">
      {/* ═══ TOP BAR ═══ */}
      <header className="nm-top-bar">
        <nav className="nm-breadcrumb">
          <a href="Bodhi Models.html">Bodhi</a>
          <span className="nm-bc-sep">/</span>
          <a href="Bodhi Models.html">Models</a>
          <span className="nm-bc-sep">/</span>
          <span className="nm-bc-curr">New API alias</span>
        </nav>
        <div className="nm-spacer" />
        <div className="nm-top-actions">
          <button className="nm-btn nm-btn-cancel">Cancel</button>
          <button className="nm-btn nm-btn-save">Save &amp; test</button>
          <button className="nm-btn nm-btn-create">Create alias</button>
        </div>
      </header>

      {/* ═══ PAGE LAYOUT ═══ */}
      <div className="nm-page-wrap">
        <main className="nm-main-col">
          <h1 className="nm-page-title">New Local model alias</h1>
          <p className="nm-page-sub">
            Configure a named alias for a local GGUF model. Runtime flags control the llama.cpp server; request defaults apply to every OpenAI-compat API call.
          </p>

          {/* ─── 1. Identity ─── */}
          <div className="nm-field-group">
            <label className="nm-field-label">
              Alias name <span className="nm-req">*</span>
            </label>
            {/* name input + chips on same row */}
            <div className="nm-alias-row">
              <input
                className="nm-input"
                value={aliasName}
                onChange={(e) => setAliasName(e.target.value)}
                placeholder="e.g. qwen-api" />
              
              <div className="nm-tag-chips">
                {['text', 'inst ver', 'I18n'].map((t, i) =>
                <span key={i} className="nm-chip">{t}</span>
                )}
              </div>
            </div>
            <AliasValidation value={aliasName} />
          </div>

          <div className="nm-divider" />

          {/* ─── 2. Model file ─── */}
          <div className="nm-field-row">
            <div className="nm-field-group" style={{ marginBottom: 0 }}>
              <label className="nm-field-label">Repo</label>
              <select className="nm-select" value={repo} onChange={(e) => setRepo(e.target.value)}>
                <option>Qwen/Qwen3-8B-GGUF</option>
                <option>meta-llama/Llama-3-8B-GGUF</option>
                <option>mistralai/Mistral-7B-GGUF</option>
              </select>
            </div>
            <div className="nm-field-group" style={{ marginBottom: 0 }}>
              <label className="nm-field-label">Snapshot</label>
              <select className="nm-select" value={snapshot} onChange={(e) => setSnapshot(e.target.value)}>
                <option>(GGUF_PREVIEW)</option>
                <option>main</option>
                <option>v0.2</option>
              </select>
            </div>
          </div>

          {/* Quant table */}
          <div style={{ marginTop: 12 }}>
            <div style={{ fontSize: '9.5px', fontWeight: 700, textTransform: 'uppercase', letterSpacing: '.07em', color: 'hsl(var(--muted-foreground))', marginBottom: 5 }}>
              Quantisation — selects file
            </div>
            <div className="nm-table-scroll">
              <div className="nm-table-wrap">
                <table className="nm-table">
                  <thead>
                    <tr>
                      <th className="nm-th" style={{ width: 28 }}></th>
                      <th className="nm-th">Quant</th>
                      <th className="nm-th">Size</th>
                      <th className="nm-th">BPW</th>
                      <th className="nm-th">Status</th>
                      <th className="nm-th" style={{ width: 88 }}></th>
                    </tr>
                  </thead>
                  <tbody>
                    {API_QUANTS.map((q, i) =>
                    <tr
                      key={i}
                      className={selectedQ === i ? 'nm-tr-sel' : ''}
                      style={{ cursor: 'pointer', transition: 'background 80ms' }}
                      onClick={() => setSelectedQ(i)}
                      onMouseEnter={(e) => {if (selectedQ !== i) e.currentTarget.style.background = 'hsl(var(--muted))';}}
                      onMouseLeave={(e) => {if (selectedQ !== i) e.currentTarget.style.background = '';}}>
                      
                        <td className="nm-td">
                          <div className={`nm-qradio${selectedQ === i ? ' nm-qradio-on' : ''}`}>
                            {selectedQ === i && <div className="nm-qradio-dot" />}
                          </div>
                        </td>
                        <td className="nm-td nm-mono" style={{ fontWeight: selectedQ === i ? 600 : 400 }}>{q.name}</td>
                        <td className="nm-td nm-mono">{q.size}</td>
                        <td className="nm-td nm-mono" style={{ color: 'hsl(var(--muted-foreground))' }}>{q.bpw}</td>
                        <td className="nm-td">
                          <span className={`nm-status-dot nm-status-${q.status}`}>
                            {STATUS_LABEL[q.status]}
                          </span>
                        </td>
                        <td className="nm-td">{q.rec && <span className="nm-rec-badge">recommended</span>}</td>
                      </tr>
                    )}
                  </tbody>
                </table>
              </div>
            </div>

            {/* Soft info note — no Pull Now button */}
            {selQ.status !== 'downloaded' &&
            <div className="nm-quant-note">
                <Icon name="info" size={12} />
                <span>
                  {selQ.status === 'partial' ?
                `${selQ.name} is partially downloaded — will resume automatically on save.` :
                `${selQ.name} (${selQ.size}) is not yet downloaded — will download automatically after save.`}
                </span>
              </div>
            }
            {selQ.status === 'downloaded' &&
            <div className="nm-quant-note">
                <Icon name="check-circle" size={12} style={{ color: 'var(--c-leaf-text)' }} />
                <span style={{ color: 'var(--c-leaf-text)' }}>
                  {selQ.name} is already downloaded locally.
                </span>
              </div>
            }
          </div>

          <div className="nm-divider" />

          {/* ─── 3. Preset & Runtime args ─── */}
          <div className="nm-field-group" style={{ marginBottom: 8 }}>
            <label className="nm-field-label">Preset</label>
            <div className="nm-preset-scroll">
              <div className="nm-preset-row">
                {API_PRESETS.map((p) =>
                <button
                  key={p.id}
                  className={`nm-preset-pill${activePreset === p.id ? ' nm-preset-pill-active' : ''}`}
                  onClick={() => applyPreset(p)}>
                  
                    {activePreset === p.id && <Icon name="check" size={10} />}
                    {p.label}
                  </button>
                )}
              </div>
            </div>
          </div>

          {/* Runtime split panel */}
          <div className="nm-split">
            <div>
              <div className="nm-ta-bar">
                <div className="nm-split-label" style={{ marginBottom: 0 }}>Active runtime flags</div>
                <button className="nm-ta-btn" onClick={() => {
                  const p = API_PRESETS.find((p) => p.id === activePreset);
                  if (p) setRuntimeText(p.flags);
                }}>
                  <Icon name="rotate-ccw" size={9} /> Reset
                </button>
                <button className="nm-ta-btn" onClick={() => navigator.clipboard?.writeText(runtimeText)}>
                  <Icon name="copy" size={9} /> Copy
                </button>
                {activePreset && <span className="nm-preset-active-label">{activePreset}</span>}
              </div>
              <textarea
                className="nm-flags-textarea"
                value={runtimeText}
                onChange={(e) => setRuntimeText(e.target.value)}
                spellCheck={false} />
              
              <div className="nm-hint">One flag per line. Click a flag on the right to append it.</div>
            </div>
            <FlagPanel
              catalogue={RUNTIME_FLAGS}
              textareaValue={runtimeText}
              onAdd={addFlag}
              mode="flags"
              label="Available flags — click to add" />
            
          </div>

          <div className="nm-divider" />

          {/* ─── 4. Request Defaults ─── */}
          {/* System prompt FIRST */}
          <div className="nm-field-group" style={{ marginBottom: 10 }}>
            <label className="nm-field-label">
              System prompt
              <span style={{ fontSize: 10, fontWeight: 400, color: 'hsl(var(--muted-foreground))' }}>— applied to every request</span>
            </label>
            <textarea
              className="nm-flags-textarea"
              style={{ minHeight: 64, fontFamily: 'var(--font-sans)', fontSize: 12.5, lineHeight: '1.55' }}
              value={sysPrompt}
              onChange={(e) => setSysPrompt(e.target.value)} />
            
          </div>

          {/* Request params split panel */}
          <div className="nm-field-group" style={{ marginBottom: 6 }}>
            <label className="nm-field-label">
              Request parameters
              <span style={{ fontSize: 10, fontWeight: 400, color: 'hsl(var(--muted-foreground))' }}>— OpenAI compat, key=value</span>
            </label>
          </div>
          <div className="nm-split">
            <div>
              <div className="nm-ta-bar">
                <div className="nm-split-label" style={{ marginBottom: 0 }}>Active parameters</div>
                <button className="nm-ta-btn" onClick={() => setReqText('temperature=0.7\ntop_p=1.0\nn=1\n')}>
                  <Icon name="rotate-ccw" size={9} /> Reset
                </button>
                <button className="nm-ta-btn" onClick={() => navigator.clipboard?.writeText(reqText)}>
                  <Icon name="copy" size={9} /> Copy
                </button>
              </div>
              <textarea
                className="nm-flags-textarea"
                value={reqText}
                onChange={(e) => setReqText(e.target.value)}
                spellCheck={false} />
              
              <div className="nm-hint">Format: <span className="nm-mono">key=value</span>. Click a param on the right to append it.</div>
            </div>
            <FlagPanel
              catalogue={REQUEST_PARAMS}
              textareaValue={reqText}
              onAdd={addParam}
              mode="params"
              label="Available parameters — click to add" />
            
          </div>

        </main>
      </div>

      {/* ═══ MOBILE BOTTOM BAR ═══ */}
      <div className="nm-mobile-bar">
        <button className="nm-btn nm-btn-cancel">Cancel</button>
        <button className="nm-btn nm-btn-save">Save &amp; test</button>
        <button className="nm-btn nm-btn-create">Create alias</button>
      </div>
      </div>{/* end nm-content */}

      {/* ═══ TWEAKS ═══ */}
      <TweaksPanel>
        <TweakSection title="Theme">
          <TweakRadio
            value={tweaks.theme}
            options={[{ label: 'Light', value: 'light' }, { label: 'Dark', value: 'dark' }]}
            onChange={(v) => setTweak('theme', v)} />
          
        </TweakSection>
      </TweaksPanel>
    </div>);

}

const apiRoot = ReactDOM.createRoot(document.getElementById('root'));
apiRoot.render(<ApiModelApp />);