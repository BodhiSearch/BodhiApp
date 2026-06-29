/* ═══════════════════════════════════════════════════
   BODHI APP SETTINGS — Settings page (on AppShell)
   bodhi-app-settings-app.jsx  (load after bodhi-app-shell.jsx + tweaks-panel.jsx)
═══════════════════════════════════════════════════ */
const { useState, useEffect, useRef, useMemo } = React;
const Ic = ShellIcon;

/* ── Section groups ── */
const SECTIONS = [
  { id: 'app',    icon: 'settings-2', name: 'App Config',    label: 'App Config',     headerName: 'App Config',                          headerDesc: 'Core application settings and paths' },
  { id: 'model',  icon: 'database',   name: 'Model Files',   label: 'Model Files',    headerName: 'Model Files Configuration',           headerDesc: 'Model file storage and configuration' },
  { id: 'llama',  icon: 'terminal',   name: 'Llama.cpp Exec',label: 'Llama.cpp Exec', headerName: 'Llama.cpp Executable Configuration',  headerDesc: 'Llama.cpp execution settings' },
  { id: 'server', icon: 'server',     name: 'Server Config', label: 'Server Config',  headerName: 'Server Configuration',                headerDesc: 'Server connection and networking settings' },
];

/* ── Settings data ── */
const SETTINGS = [
  { key: 'BODHI_HOME', section: 'app', sectionLabel: 'App Configuration',
    desc: 'The home directory for Bodhi application. All app data, configs and logs are stored here.',
    current: '/Users/amir36/.bodhi-dev-makefile', defaultVal: '/Users/amir36/.bodhi-dev-makefile',
    type: 'string', source: 'environment', requiresRestart: false },
  { key: 'HF_HOME', section: 'model', sectionLabel: 'Model Files Configuration',
    desc: 'Home directory for Hugging Face model files. Models downloaded from HuggingFace are cached here.',
    current: '/Users/amir36/.cache/huggingface', defaultVal: '/Users/amir36/.cache/huggingface',
    type: 'string', source: 'default', requiresRestart: false },
  { key: 'BODHI_EXEC_LOOKUP_PATH', section: 'llama', sectionLabel: 'Llama.cpp Executable Configuration',
    desc: 'Path to look for Llama.cpp executables. The app will search this directory for llama-server and other executables.',
    current: '/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/bodhi/llama-bins', defaultVal: '/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/bodhi/llama-bins',
    type: 'string', source: 'default', requiresRestart: true },
  { key: 'BODHI_EXEC_TARGET', section: 'llama', sectionLabel: 'Llama.cpp Executable Configuration',
    desc: 'Target platform for llama.cpp executable.',
    current: 'aarch64-apple-darwin', defaultVal: 'aarch64-apple-darwin',
    type: 'string', source: 'default', requiresRestart: true },
  { key: 'BODHI_EXEC_VARIANT', section: 'llama', sectionLabel: 'Llama.cpp Executable Configuration',
    desc: 'Optimized hardware-specific variant of llama.cpp to use.',
    current: 'metal', defaultVal: 'metal',
    type: 'option', source: 'default', requiresRestart: true, options: ['metal', 'cpu', 'cuda', 'vulkan'] },
  { key: 'BODHI_EXEC_NAME', section: 'llama', sectionLabel: 'Llama.cpp Executable Configuration',
    desc: 'Name of the llama.cpp executable to invoke.',
    current: 'llama-server', defaultVal: 'llama-server',
    type: 'string', source: 'default', requiresRestart: false },
  { key: 'BODHI_EXEC_VARIANTS', section: 'llama', sectionLabel: 'Llama.cpp Executable Configuration',
    desc: 'Available llama.cpp variants for this platform.',
    current: 'metal,cpu', defaultVal: 'metal,cpu',
    type: 'string', source: 'default', requiresRestart: false },
  { key: 'BODHI_LLAMACPP_ARGS', section: 'llama', sectionLabel: 'Llama.cpp Executable Configuration',
    desc: 'Common arguments passed to all llama.cpp server instances.',
    current: '--jinja --no-webui', defaultVal: '--jinja --no-webui',
    type: 'string', source: 'default', requiresRestart: false },
  { key: 'BODHI_KEEP_ALIVE_SECS', section: 'llama', sectionLabel: 'Llama.cpp Executable Configuration',
    desc: 'Keep alive timeout for llama-server (in seconds). Range: 300 (5 mins) to 86400 (1 day).',
    current: '600', defaultVal: '300',
    type: 'number', source: 'modified', requiresRestart: false },
  { key: 'BODHI_SCHEME', section: 'server', sectionLabel: 'Server Configuration',
    desc: 'Scheme used for server connection.',
    current: 'http', defaultVal: 'http',
    type: 'option', source: 'default', requiresRestart: true, options: ['http', 'https'] },
  { key: 'BODHI_HOST', section: 'server', sectionLabel: 'Server Configuration',
    desc: 'Host address for the server.',
    current: '0.0.0.0', defaultVal: '0.0.0.0',
    type: 'string', source: 'default', requiresRestart: true },
  { key: 'BODHI_PORT', section: 'server', sectionLabel: 'Server Configuration',
    desc: 'Port number for the server.',
    current: '21135', defaultVal: '1135',
    type: 'number', source: 'command_line', requiresRestart: true },
];

function highlight(text, query) {
  if (!query) return text;
  const idx = text.toLowerCase().indexOf(query);
  if (idx < 0) return text;
  return (<>{text.slice(0, idx)}<mark>{text.slice(idx, idx + query.length)}</mark>{text.slice(idx + query.length)}</>);
}

/* ── Sidebar: settings-group nav (collapse-aware) ── */
function SettingsSidebar({ counts, active, onNavigate }) {
  const { collapsed } = useShell();
  if (collapsed) {
    return (
      <>
        {SECTIONS.map(s => (
          <button key={s.id} className={`shell-railbtn shell-tip${active === s.id ? ' on' : ''}`}
            data-tip={s.name} onClick={() => onNavigate(s.id)}>
            <Ic name={s.icon} size={18} />
          </button>
        ))}
      </>
    );
  }
  return (
    <div className="section-nav">
      <div className="snav-label">Settings Groups</div>
      {SECTIONS.map(s => (
        <button key={s.id} className={`snav-item${active === s.id ? ' active' : ''}`} onClick={() => onNavigate(s.id)}>
          <Ic name={s.icon} size={13} /> {s.name}
          <span className="snav-count">{counts[s.id] || 0}</span>
        </button>
      ))}
      <div className="snav-legend">
        <div className="snav-legend-title">Legend</div>
        <div className="snav-legend-rows">
          <div className="snav-legend-row"><span className="badge badge-default" style={{ fontSize: 9 }}>default</span> At default value</div>
          <div className="snav-legend-row"><span className="badge badge-modified" style={{ fontSize: 9 }}>modified</span> Overridden</div>
          <div className="snav-legend-row"><span className="badge badge-env" style={{ fontSize: 9 }}>env</span> From env var</div>
          <div className="snav-legend-row"><span className="badge badge-cmdline" style={{ fontSize: 9 }}>cmd</span> Command line</div>
        </div>
      </div>
    </div>
  );
}

/* ── Setting row ── */
function SettingRow({ s, value, source, isPending, active, search, tweaks, onOpen }) {
  const atDefault = value === s.defaultVal;
  const isEnv = source === 'environment';
  const isCmdLine = source === 'command_line';
  const isModified = (value !== s.defaultVal) || source === 'modified';

  let badge;
  if (isCmdLine) badge = <span className="badge badge-cmdline">cmd</span>;
  else if (isEnv) badge = <span className="badge badge-env">env</span>;
  else if (isModified && !atDefault) badge = <span className="badge badge-modified">modified</span>;
  else badge = <span className="badge badge-default">default</span>;

  const cls = ['setting-row',
    isModified ? 'modified' : '',
    isPending ? 'has-restart' : '',
    active ? 'active' : '',
    !tweaks.highlight ? 'no-highlight' : '',
  ].filter(Boolean).join(' ');

  return (
    <div className={cls} onClick={() => onOpen(s.key)}>
      <div className="row-key">
        <span className="key-name">{search ? highlight(s.key, search) : s.key}</span>
      </div>
      <div className={'row-value' + (atDefault ? ' at-default' : '')}>{value || '—'}</div>
      <div className="row-actions">
        {badge}
        {tweaks.type && <span className="type-badge">{s.type}</span>}
        <button className="row-edit-btn" onClick={e => { e.stopPropagation(); onOpen(s.key); }}>
          <Ic name="pencil" size={12} />
        </button>
      </div>
      {tweaks.desc && <div className="row-desc">{s.desc}</div>}
    </div>
  );
}

/* ── Rail header (railHeader slot) ── */
function SettingDetailHeader({ s, onClose }) {
  return (
    <div className="dp-head">
      <div className="dp-head-icon" style={{ background: 'var(--c-indigo-bg)', color: 'var(--c-indigo-text)' }}>
        <Ic name="sliders-horizontal" size={15} />
      </div>
      <div className="dp-head-body">
        <div className="dp-head-title mono">{s.key}</div>
        <div className="dp-head-sub">{s.sectionLabel}</div>
      </div>
      <button className="dp-close" onClick={onClose} title="Close"><Ic name="x" size={15} /></button>
    </div>
  );
}

/* ── Rail body (rail slot) — edit form with save/cancel ── */
function SettingDetailPanel({ settingKey, values, sources, onSave, onClose }) {
  const s = SETTINGS.find(x => x.key === settingKey);
  const [draft, setDraft] = useState('');
  useEffect(() => {
    if (s) setDraft(values[s.key] !== undefined ? values[s.key] : s.current);
  }, [settingKey]);

  const currentVal = values[s.key] !== undefined ? values[s.key] : s.current;
  const source = sources[s.key];
  const dirty = draft !== currentVal;
  const isSelect = s.type === 'option' || (s.options && s.options.length);

  return (
    <div className="dp-panel">
      <div className="dp-status-row">
        {source === 'environment'  && <span className="badge badge-env">environment</span>}
        {source === 'command_line' && <span className="badge badge-cmdline">command_line</span>}
        {source === 'modified'     && <span className="badge badge-modified">modified</span>}
        {source === 'default'      && <span className="badge badge-default">default</span>}
        <span className="type-badge">{s.type}</span>
        {s.requiresRestart && <span className="badge badge-restart">↺ requires restart</span>}
      </div>

      <div className="dp-body">
        <div className="dp-section">
          <p className="dp-desc">{s.desc}</p>
        </div>

        <div className="dp-section">
          <div className="dp-sec-lbl">Values</div>
          <div className="dp-rows">
            <div className="dp-row">
              <span className="dp-row-k"><Ic name="circle-dot" size={13} /> Current</span>
              <span className="dp-row-v mono">{currentVal || '—'}</span>
            </div>
            <div className="dp-row">
              <span className="dp-row-k"><Ic name="rotate-ccw" size={13} /> Default</span>
              <span className="dp-row-v mono" style={!s.defaultVal ? { color: 'hsl(var(--muted-foreground))', fontStyle: 'italic' } : null}>{s.defaultVal || '—'}</span>
            </div>
          </div>
        </div>

        {s.requiresRestart && (
          <div className="dp-restart-warn">
            <Ic name="alert-triangle" size={15} />
            <div><strong>Requires server restart.</strong> This change takes effect after restarting the Bodhi server.</div>
          </div>
        )}

        <div className="dp-section">
          <div className="dp-sec-lbl">New value</div>
          <div className="dp-field">
            {isSelect ? (
              <select className="field-select" value={draft} onChange={e => setDraft(e.target.value)}>
                {(s.options || []).map(o => <option key={o} value={o}>{o}</option>)}
              </select>
            ) : (
              <input className="field-input" type={s.type === 'number' ? 'number' : 'text'}
                value={draft} placeholder={s.defaultVal || ''} onChange={e => setDraft(e.target.value)} />
            )}
            <span className="dp-field-hint">
              {s.type === 'number' ? 'Enter a numeric value.' : s.type === 'option' ? 'Choose from the available options.' : 'Enter the new string value.'}
            </span>
          </div>
        </div>
      </div>

      <div className="dp-foot">
        <button className="dp-btn dp-btn-accent" disabled={!dirty} onClick={() => onSave(s.key, draft)}>
          <Ic name="check" size={14} /> {dirty ? 'Save changes' : 'Saved'}
        </button>
        <div className="dp-foot-row">
          <button className="dp-btn dp-btn-outline" onClick={onClose}>Cancel</button>
          {draft !== s.defaultVal && (
            <button className="dp-btn dp-btn-outline" onClick={() => setDraft(s.defaultVal)}>
              <Ic name="rotate-ccw" size={13} /> Reset
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

/* ── Main content (child of AppShell so it can open the rail) ── */
function SettingsBody({ search, setSearch, filter, setFilter, setFilterChip, isVisible, anyVisible,
                        values, sources, pendingRestart, tweaks, q, selKey, onSelect,
                        scrollRef, sectionRefs, onScroll }) {
  const { openRail } = useShell();
  const open = key => { onSelect(key); openRail(); };

  const catCounts = {
    all:      SETTINGS.length,
    modified: SETTINGS.filter(s => sources[s.key] === 'modified' || values[s.key] !== s.defaultVal).length,
    env:      SETTINGS.filter(s => sources[s.key] === 'environment').length,
    restart:  SETTINGS.filter(s => s.requiresRestart).length,
  };

  return (
    <div className="settings-content l-page">
      <ListToolbar
        categories={[
          { id: 'all',      label: 'All',           badge: catCounts.all },
          { id: 'modified', label: 'Modified',      badge: catCounts.modified },
          { id: 'env',      label: 'Env',           badge: catCounts.env },
          { id: 'restart',  label: 'Needs Restart', badge: catCounts.restart },
        ]}
        category={filter || 'all'}
        onCategory={id => setFilter(id === 'all' ? null : id)}
        search={search} onSearch={setSearch} searchPlaceholder="Search settings…" />

      <div className="settings-scroll" ref={scrollRef} onScroll={onScroll}>
        {SECTIONS.map(sec => {
          const rows = SETTINGS.filter(s => s.section === sec.id && isVisible(s));
          if (!anyVisible || rows.length === 0) return null;
          return (
            <div className="section-group" key={sec.id} ref={el => sectionRefs.current[sec.id] = el}>
              <div className="section-header">
                <Ic name={sec.icon} size={14} />
                <span className="section-header-name">{sec.headerName}</span>
                <span className="section-header-desc">{sec.headerDesc}</span>
              </div>
              {rows.map(s => (
                <SettingRow key={s.key} s={s} value={values[s.key]} source={sources[s.key]}
                  isPending={pendingRestart.includes(s.key)} active={s.key === selKey}
                  search={q} tweaks={tweaks} onOpen={open} />
              ))}
            </div>
          );
        })}
        {!anyVisible && (
          <div className="empty-state">
            <Ic name="search-x" size={32} />
            <p>No settings match your search.</p>
          </div>
        )}
      </div>
    </div>
  );
}

/* ── Main App ── */
function AppSettingsApp() {
  const [tweaks] = useState({ desc: true, type: true, highlight: true });

  const [values, setValues] = useState(() => Object.fromEntries(SETTINGS.map(s => [s.key, s.current])));
  const [sources, setSources] = useState(() => Object.fromEntries(SETTINGS.map(s => [s.key, s.source])));
  const [pendingRestart, setPendingRestart] = useState(
    () => SETTINGS.filter(s => s.requiresRestart && s.current !== s.defaultVal).map(s => s.key));
  const [bannerState, setBannerState] = useState(
    () => SETTINGS.some(s => s.requiresRestart && s.current !== s.defaultVal) ? 'pending' : null);

  const [search, setSearch] = useState('');
  const [filter, setFilter] = useState(null);
  const [selKey, setSelKey] = useState(null);
  const [activeSection, setActiveSection] = useState('app');

  const scrollRef = useRef(null);
  const sectionRefs = useRef({});

  const counts = useMemo(() => {
    const c = {};
    SECTIONS.forEach(s => { c[s.id] = SETTINGS.filter(x => x.section === s.id).length; });
    return c;
  }, []);

  const q = search.toLowerCase().trim();
  const isVisible = s => {
    const val = values[s.key];
    if (filter === 'modified' && !(sources[s.key] === 'modified' || val !== s.defaultVal)) return false;
    if (filter === 'env' && sources[s.key] !== 'environment') return false;
    if (filter === 'restart' && !s.requiresRestart) return false;
    if (q) {
      const hay = (s.key + ' ' + s.desc + ' ' + val).toLowerCase();
      if (!hay.includes(q)) return false;
    }
    return true;
  };

  const anyVisible = SETTINGS.some(isVisible);

  const navigate = secId => {
    setActiveSection(secId);
    const el = sectionRefs.current[secId];
    const scroll = scrollRef.current;
    if (el && scroll) scroll.scrollTo({ top: el.offsetTop - 2, behavior: 'smooth' });
  };

  const onScroll = () => {
    const scroll = scrollRef.current;
    if (!scroll) return;
    let active = null;
    SECTIONS.forEach(s => {
      const el = sectionRefs.current[s.id];
      if (el && el.offsetTop - scroll.scrollTop <= 60) active = s.id;
    });
    if (active && active !== activeSection) setActiveSection(active);
  };

  const saveSetting = (key, newVal) => {
    const s = SETTINGS.find(x => x.key === key);
    setValues(p => ({ ...p, [key]: newVal }));
    setSources(p => ({ ...p, [key]: newVal !== s.defaultVal ? 'modified' : 'default' }));
    setPendingRestart(prev => {
      const next = new Set(prev);
      if (s.requiresRestart && newVal !== s.defaultVal) next.add(key);
      else next.delete(key);
      const arr = [...next];
      setBannerState(arr.length > 0 ? 'pending' : null);
      return arr;
    });
  };

  const triggerRestart = () => {
    setPendingRestart([]);
    setBannerState('success');
    setTimeout(() => setBannerState(null), 4000);
  };

  const setFilterChip = f => setFilter(prev => prev === f ? null : f);

  const selSetting = SETTINGS.find(x => x.key === selKey) || null;

  /* Restart banner node */
  const banner = bannerState && (
    bannerState === 'success' ? (
      <div className="restart-banner leaf">
        <Ic name="check-circle-2" size={15} />
        <div className="restart-banner-text">Server restarted successfully. All pending settings are now active.</div>
        <button className="btn-dismiss" onClick={() => setBannerState(null)}><Ic name="x" size={13} /></button>
      </div>
    ) : (
      <div className="restart-banner">
        <Ic name="alert-triangle" size={15} />
        <div className="restart-banner-text">
          <strong>{pendingRestart.length} setting{pendingRestart.length > 1 ? 's' : ''}</strong> require a server restart to take effect.{' '}
          <span style={{ opacity: .7, fontSize: 11.5 }}>({pendingRestart.join(', ')})</span>
        </div>
        <button className="btn-restart" onClick={triggerRestart}><Ic name="refresh-cw" size={12} /> Restart Server</button>
        <button className="btn-dismiss" onClick={() => setBannerState(null)} title="Dismiss"><Ic name="x" size={13} /></button>
      </div>
    )
  );

  return (
    <>
      <AppShell
        section="settings" subPage={null} resizeKey="settings"
        breadcrumb={[
          { label: 'Bodhi', href: 'Chat.html' },
          { label: 'App Settings', current: true },
        ]}
        sidebar={<SettingsSidebar counts={counts} active={activeSection} onNavigate={navigate} />}
        banner={banner}
        rail={selKey ? <SettingDetailPanel settingKey={selKey} values={values} sources={sources}
              onSave={saveSetting} onClose={() => setSelKey(null)} /> : null}
        railHeader={selSetting ? <SettingDetailHeader s={selSetting} onClose={() => setSelKey(null)} /> : undefined}
        contentClass="flush" mainScroll={false} railScroll={false}
      >
        <SettingsBody
          search={search} setSearch={setSearch}
          filter={filter} setFilter={setFilter} setFilterChip={setFilterChip}
          isVisible={isVisible} anyVisible={anyVisible}
          values={values} sources={sources} pendingRestart={pendingRestart}
          tweaks={tweaks} q={q} selKey={selKey} onSelect={setSelKey}
          scrollRef={scrollRef} sectionRefs={sectionRefs} onScroll={onScroll}
        />
      </AppShell>
    </>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<AppSettingsApp />);
