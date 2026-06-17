/* ═══════════════════════════════════════════
   CREATE LOCAL MODEL — React App
   local-model-app.jsx

   Layout (shell · 52px topbar · centered container · footer) comes from
   bodhi-form.css (.bf-*). Page-specific UI (quant table, presets, split
   panels, flag list) is styled by local-model.css (.nm-*).
   Sidebar auto-wires resize via bodhi-sidebar-react.jsx.
═══════════════════════════════════════════ */

/* ── Quant options ─────────────────────── */
const LM_QUANTS = [
  { name: 'Q4_K_M', size: '4.9 GB', bpw: '4.85 bpw', rec: true,  status: 'downloaded' },
  { name: 'Q3_K_S', size: '3.7 GB', bpw: '3.50 bpw', rec: false, status: 'downloaded' },
  { name: 'Q5_K',   size: '5.7 GB', bpw: '5.69 bpw', rec: false, status: 'partial' },
  { name: 'Q8_0',   size: '8.5 GB', bpw: '8.50 bpw', rec: false, status: 'remote' },
  { name: 'F16',    size: '16.0 GB', bpw: '16.00 bpw', rec: false, status: 'remote' },
];

const STATUS_LABEL = {
  downloaded: 'Downloaded',
  partial: 'Partial',
  remote: 'Not downloaded',
};

/* ── Presets ───────────────────────────── */
const LM_PRESETS = [
  { id: 'default',   label: 'Default',   flags: '--ctx-size 4096\n--flash-attn auto\n--parallel 4\n' },
  { id: 'coding',    label: 'Coding',    flags: '--ctx-size 8192\n--flash-attn auto\n--parallel 2\n--cache-prompt true\n' },
  { id: 'non-coder', label: 'Non Coder', flags: '--ctx-size 2048\n--parallel 2\n' },
  { id: 'smart',     label: 'Smart',     flags: '--ctx-size 16384\n--flash-attn auto\n--parallel 8\n--rope-scaling yarn\n' },
  { id: 'translate', label: 'Translate', flags: '--ctx-size 4096\n--parallel 4\n--repeat-penalty 1.1\n' },
  { id: 'rag',       label: 'RAG',       flags: '--ctx-size 32768\n--flash-attn auto\n--cache-prompt true\n--parallel 6\n' },
  { id: 'kwa-deep',  label: 'KWA Deep',  flags: '--ctx-size 47536\n--flash-attn auto\n--parallel 10\n--cont-batching true\n--cache-type-k q8_0\n--rope-scaling yarn\n--cache-prompt true\n--grp-attn-n 8\n' },
];

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
  { key: '--chat-template', type: 'string', range: 'chatml | llama2 | …', desc: 'Override auto-detected chat template.' },
];

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
  { key: 'user', type: 'string', range: '<string>', desc: 'End-user ID for abuse tracking.' },
];

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
    <span ref={ref} style={{ display: 'inline-flex', width: size, height: size, alignItems: 'center', justifyContent: 'center', flexShrink: 0, ...style }} />
  );
}

/* ── Inline validation message ─────────── */
function AliasValidation({ value }) {
  if (!value) return (
    <div className="nm-validation nm-validation-hint">
      <Icon name="info" size={12} /> Lowercase, digits, and dashes only.
    </div>
  );
  const valid = /^[a-z0-9-]+$/.test(value);
  if (valid) return (
    <div className="nm-validation nm-validation-ok">
      <Icon name="check-circle" size={12} /> Looks good.
    </div>
  );
  return (
    <div className="nm-validation nm-validation-err">
      <Icon name="alert-circle" size={12} /> Only lowercase letters, digits, and dashes allowed.
    </div>
  );
}

/* ── Combobox — typeable input with suggestion dropdown ─────────
   Behaves like a select (click chevron / arrow keys / highlight)
   but never enforces selection: any typed value is accepted.
──────────────────────────────────────────────────────────────── */
function Combobox({ value, onChange, options, placeholder, mono }) {
  const [open, setOpen] = React.useState(false);
  const [active, setActive] = React.useState(-1);
  const [typed, setTyped] = React.useState(false); // filter only after the user types
  const wrapRef = React.useRef(null);
  const inputRef = React.useRef(null);
  const listRef = React.useRef(null);

  const filtered = React.useMemo(() => {
    const q = value.trim().toLowerCase();
    if (!typed || !q) return options;
    return options.filter((o) => o.value.toLowerCase().includes(q));
  }, [value, options, typed]);

  // close on outside click
  React.useEffect(() => {
    if (!open) return;
    const onDoc = (e) => { if (wrapRef.current && !wrapRef.current.contains(e.target)) setOpen(false); };
    document.addEventListener('mousedown', onDoc);
    return () => document.removeEventListener('mousedown', onDoc);
  }, [open]);

  // keep active option in view
  React.useEffect(() => {
    if (!open || active < 0 || !listRef.current) return;
    const el = listRef.current.children[active];
    if (el) el.scrollIntoView({ block: 'nearest' });
  }, [active, open]);

  const openMenu = () => { setOpen(true); setActive(-1); };
  const commit = (v) => { onChange(v); setOpen(false); setTyped(false); setActive(-1); };

  const onKey = (e) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (!open) { openMenu(); return; }
      setActive((a) => Math.min(a + 1, filtered.length - 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (!open) return;
      setActive((a) => Math.max(a - 1, 0));
    } else if (e.key === 'Enter') {
      if (open && active >= 0 && filtered[active]) { e.preventDefault(); commit(filtered[active].value); }
    } else if (e.key === 'Escape') {
      setOpen(false); setActive(-1);
    }
  };

  return (
    <div className="nm-combo" ref={wrapRef}>
      <div className={`nm-combo-control${open ? ' nm-combo-open' : ''}`}>
        <input
          ref={inputRef}
          className={`nm-combo-input${mono ? ' nm-mono' : ''}`}
          value={value}
          placeholder={placeholder}
          spellCheck={false}
          autoComplete="off"
          onChange={(e) => { onChange(e.target.value); setTyped(true); setActive(-1); if (!open) setOpen(true); }}
          onFocus={openMenu}
          onKeyDown={onKey} />
        <button
          type="button"
          className="nm-combo-chevron"
          tabIndex={-1}
          aria-label="Toggle suggestions"
          onMouseDown={(e) => { e.preventDefault(); }}
          onClick={() => { if (open) { setOpen(false); } else { setTyped(false); openMenu(); inputRef.current?.focus(); } }}>
          <Icon name="chevron-down" size={15} />
        </button>
      </div>
      {open && filtered.length > 0 &&
        <div className="nm-combo-menu" ref={listRef} role="listbox">
          {filtered.map((o, i) => {
            const selected = o.value === value;
            return (
              <div
                key={o.value}
                role="option"
                aria-selected={selected}
                className={`nm-combo-opt${i === active ? ' nm-combo-opt-active' : ''}${selected ? ' nm-combo-opt-sel' : ''}`}
                onMouseEnter={() => setActive(i)}
                onMouseDown={(e) => { e.preventDefault(); commit(o.value); }}>
                <span className="nm-combo-opt-main nm-mono">{o.value}</span>
                {o.note && <span className="nm-combo-opt-note">{o.note}</span>}
                {selected && <Icon name="check" size={13} style={{ marginLeft: 'auto', color: 'var(--c-lotus-text)' }} />}
              </div>
            );
          })}
        </div>}
    </div>
  );
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
                <div className="nm-flag-vals">{f.range}</div>}
            </div>
          );
        })}
      </div>
    </div>
  );
}

/* ── Main App ──────────────────────────── */
function LocalModelApp() {
  /* form state */
  const [aliasName, setAliasName] = React.useState('qwen-api');
  const [repo, setRepo] = React.useState('Qwen/Qwen3-8B-GGUF');
  const [snapshot, setSnapshot] = React.useState('main');
  const [selectedQ, setSelectedQ] = React.useState(0);
  const [activePreset, setActivePreset] = React.useState('kwa-deep');

  /* quant fetch state — 'ready' | 'loading' */
  const [quantState, setQuantState] = React.useState('ready');
  const fetchTimer = React.useRef(null);
  const firstFetch = React.useRef(true);

  /* re-fetch quantisations whenever repo or snapshot changes (debounced) */
  React.useEffect(() => {
    if (firstFetch.current) { firstFetch.current = false; return; }
    if (!repo.trim()) { setQuantState('ready'); return; }
    setQuantState('loading');
    clearTimeout(fetchTimer.current);
    fetchTimer.current = setTimeout(() => setQuantState('ready'), 1100);
    return () => clearTimeout(fetchTimer.current);
  }, [repo, snapshot]);

  /* textareas */
  const [runtimeText, setRuntimeText] = React.useState(
    LM_PRESETS.find((p) => p.id === 'kwa-deep').flags
  );
  const [reqText, setReqText] = React.useState('temperature=0.7\ntop_p=1.0\nn=1\n');
  const [sysPrompt, setSysPrompt] = React.useState('You are a helpful assistant.');

  const applyPreset = (preset) => {
    setActivePreset(preset.id);
    setRuntimeText(preset.flags);
  };

  const addFlag = (flag) => {
    const defaultVal = flag.type === 'bool' ? 'true' :
      flag.type === 'flag' ? '' :
      flag.type === 'int' ? '0' :
      flag.type === 'float' ? '0.0' : '<value>';
    const line = flag.type === 'flag' ? `${flag.key}\n` : `${flag.key} ${defaultVal}\n`;
    setRuntimeText((t) => t + line);
  };

  const addParam = (param) => {
    const defaultVal = param.type === 'bool' ? 'false' :
      param.type === 'int' ? '0' :
      param.type === 'float' ? '0.0' : '<value>';
    setReqText((t) => t + `${param.key}=${defaultVal}\n`);
  };

  const selQ = LM_QUANTS[selectedQ];

  return (
    <>
    <AppShell
      section="models" subPage="new-local-model" resizeKey="createmodel"
      breadcrumb={[
        { label: 'Bodhi', href: 'Bodhi Chat.html' },
        { label: 'Models', href: 'Bodhi Models.html' },
        { label: 'New Local Model', current: true },
      ]}
      contentClass="flush" mainScroll={false}
    >
        {/* ═══ SCROLL · centered container ═══ */}
        <div className="bf-scroll">
          <div className="bf-container">
            <div className="bf-page-head">
              <h1 className="bf-page-title">Create New Local Model</h1>
              <p className="bf-page-sub">
                Configure a named alias for a local GGUF model. Runtime flags control the llama.cpp server; request defaults apply to every OpenAI-compatible API call.
              </p>
            </div>

            <div className="bf-card">
              <div className="bf-card-body">

                {/* ── Identity ── */}
                <div className="bf-section">
                  <div className="bf-section-title">Identity</div>
                  <div className="bf-field">
                    <label className="bf-label">
                      <span className="bf-label-text">Alias name</span>
                      <span className="bf-req">*</span>
                    </label>
                    <div className="nm-alias-row">
                      <input
                        className="bf-input"
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
                </div>

                <div className="bf-divider"></div>

                {/* ── Model file ── */}
                <div className="bf-section">
                  <div className="bf-section-title">Model file</div>
                  <div className="bf-field-row">
                    <div className="bf-field">
                      <label className="bf-label"><span className="bf-label-text">Repo</span></label>
                      <Combobox
                        value={repo}
                        onChange={setRepo}
                        placeholder="org/repo"
                        mono
                        options={[
                          { value: 'Qwen/Qwen3-8B-GGUF', note: 'suggested' },
                          { value: 'meta-llama/Llama-3-8B-GGUF' },
                          { value: 'mistralai/Mistral-7B-GGUF' },
                        ]} />
                      <div className="bf-hint">Suggestions shown — or type any <span className="nm-mono">&lt;org&gt;/&lt;repo&gt;</span> to download.</div>
                    </div>
                    <div className="bf-field">
                      <label className="bf-label"><span className="bf-label-text">Snapshot</span></label>
                      <Combobox
                        value={snapshot}
                        onChange={setSnapshot}
                        placeholder="main"
                        mono
                        options={[
                          { value: 'main', note: 'default' },
                          { value: '(GGUF_PREVIEW)' },
                          { value: 'v0.2' },
                        ]} />
                      <div className="bf-hint">Defaults to <span className="nm-mono">main</span> — or paste a commit SHA / branch.</div>
                    </div>
                  </div>

                  {/* Quant table */}
                  <div className="bf-field" style={{ marginBottom: 0 }}>
                    <div className="nm-quant-label">Quantisation — selects file</div>
                    {quantState === 'loading' ? (
                      <div className="nm-quant-loading">
                        <div className="nm-spinner" />
                        <div className="nm-quant-loading-text">
                          Fetching quantisations
                          <span className="nm-quant-loading-sub">
                            Reading <span className="nm-mono">{repo || 'repo'}</span> @ <span className="nm-mono">{snapshot || 'main'}</span>
                          </span>
                        </div>
                      </div>
                    ) : (
                    <React.Fragment>
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
                              <th className="nm-th" style={{ width: 96 }}></th>
                            </tr>
                          </thead>
                          <tbody>
                            {LM_QUANTS.map((q, i) =>
                              <tr
                                key={i}
                                className={selectedQ === i ? 'nm-tr-sel' : ''}
                                style={{ cursor: 'pointer' }}
                                onClick={() => setSelectedQ(i)}>
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

                    {selQ.status !== 'downloaded' &&
                      <div className="nm-quant-note">
                        <Icon name="info" size={13} />
                        <span>
                          {selQ.status === 'partial' ?
                            `${selQ.name} is partially downloaded — will resume automatically on save.` :
                            `${selQ.name} (${selQ.size}) is not yet downloaded — will download automatically after save.`}
                        </span>
                      </div>}
                    {selQ.status === 'downloaded' &&
                      <div className="nm-quant-note">
                        <Icon name="check-circle" size={13} style={{ color: 'var(--c-leaf-text)' }} />
                        <span style={{ color: 'var(--c-leaf-text)' }}>
                          {selQ.name} is already downloaded locally.
                        </span>
                      </div>}
                    </React.Fragment>
                    )}
                  </div>
                </div>

                <div className="bf-divider"></div>

                {/* ── Preset & runtime flags ── */}
                <div className="bf-section">
                  <div className="bf-section-title">Preset &amp; runtime flags</div>
                  <div className="bf-field">
                    <label className="bf-label"><span className="bf-label-text">Preset</span></label>
                    <div className="nm-preset-scroll">
                      <div className="nm-preset-row">
                        {LM_PRESETS.map((p) =>
                          <button
                            key={p.id}
                            className={`nm-preset-pill${activePreset === p.id ? ' nm-preset-pill-active' : ''}`}
                            onClick={() => applyPreset(p)}>
                            {activePreset === p.id && <Icon name="check" size={11} />}
                            {p.label}
                          </button>
                        )}
                      </div>
                    </div>
                  </div>

                  <div className="nm-split">
                    <div>
                      <div className="nm-ta-bar">
                        <div className="nm-split-label" style={{ marginBottom: 0 }}>Active runtime flags</div>
                        <button className="nm-ta-btn" onClick={() => {
                          const p = LM_PRESETS.find((p) => p.id === activePreset);
                          if (p) setRuntimeText(p.flags);
                        }}>
                          <Icon name="rotate-ccw" size={10} /> Reset
                        </button>
                        <button className="nm-ta-btn" onClick={() => navigator.clipboard?.writeText(runtimeText)}>
                          <Icon name="copy" size={10} /> Copy
                        </button>
                        {activePreset && <span className="nm-preset-active-label">{activePreset}</span>}
                      </div>
                      <textarea
                        className="nm-flags-textarea"
                        value={runtimeText}
                        onChange={(e) => setRuntimeText(e.target.value)}
                        spellCheck={false} />
                      <div className="bf-hint">One flag per line. Click a flag on the right to append it.</div>
                    </div>
                    <FlagPanel
                      catalogue={RUNTIME_FLAGS}
                      textareaValue={runtimeText}
                      onAdd={addFlag}
                      mode="flags"
                      label="Available flags — click to add" />
                  </div>
                </div>

                <div className="bf-divider"></div>

                {/* ── Request defaults ── */}
                <div className="bf-section">
                  <div className="bf-section-title">Request defaults</div>

                  <div className="bf-field">
                    <label className="bf-label">
                      <span className="bf-label-text">System prompt</span>
                      <span className="bf-optional">— applied to every request</span>
                    </label>
                    <textarea
                      className="bf-textarea"
                      style={{ minHeight: 72 }}
                      value={sysPrompt}
                      onChange={(e) => setSysPrompt(e.target.value)} />
                  </div>

                  <div className="bf-field" style={{ marginBottom: 0 }}>
                    <label className="bf-label">
                      <span className="bf-label-text">Request parameters</span>
                      <span className="bf-optional">— OpenAI compat, key=value</span>
                    </label>
                    <div className="nm-split">
                      <div>
                        <div className="nm-ta-bar">
                          <div className="nm-split-label" style={{ marginBottom: 0 }}>Active parameters</div>
                          <button className="nm-ta-btn" onClick={() => setReqText('temperature=0.7\ntop_p=1.0\nn=1\n')}>
                            <Icon name="rotate-ccw" size={10} /> Reset
                          </button>
                          <button className="nm-ta-btn" onClick={() => navigator.clipboard?.writeText(reqText)}>
                            <Icon name="copy" size={10} /> Copy
                          </button>
                        </div>
                        <textarea
                          className="nm-flags-textarea"
                          value={reqText}
                          onChange={(e) => setReqText(e.target.value)}
                          spellCheck={false} />
                        <div className="bf-hint">Format: <span className="nm-mono">key=value</span>. Click a param on the right to append it.</div>
                      </div>
                      <FlagPanel
                        catalogue={REQUEST_PARAMS}
                        textareaValue={reqText}
                        onAdd={addParam}
                        mode="params"
                        label="Available parameters — click to add" />
                    </div>
                  </div>
                </div>
              </div>{/* end card-body */}

              {/* ═══ FOOTER — the ONLY place actions live ═══ */}
              <div className="bf-footer">
                <button className="bf-btn bf-btn-secondary">
                  <Icon name="plug-zap" size={13} />
                  Save &amp; test
                </button>
                <div className="bf-footer-spacer"></div>
                <button className="bf-btn bf-btn-ghost">Cancel</button>
                <button className="bf-btn bf-btn-primary">Create alias</button>
              </div>
            </div>{/* end card */}
          </div>
        </div>
    </AppShell>
    </>
  );
}

const lmRoot = ReactDOM.createRoot(document.getElementById('root'));
lmRoot.render(<LocalModelApp />);
