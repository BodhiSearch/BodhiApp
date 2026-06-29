/* ═══════════════════════════════════════════════════
   BODHI MCP — SERVER MANAGEMENT (admin, on AppShell)
   mcp/bodhi-mcp-server-app.jsx

   TWO separate pages (each HTML sets window.MCP_SERVER_PAGE):
     • create — New MCP Server. Register a server + one OPTIONAL inline
                auth mechanism. (MCP-New-Server.html)
     • view   — Configure server hub. Per-section editing: basic info
                (edit → save), and auth mechanisms — add INLINE (the
                "Add auth mechanism" button becomes a form in place; Cancel
                restores the button) and delete (with confirmation).
                (MCP-Server.html?server=<id>)

   Auth Type order: Header / Query Key first (no auto-discovery), then
   OAuth. Selecting OAuth runs dynamic client registration (DCR); on
   failure it falls back to pre-registered manual entry.
═══════════════════════════════════════════════════ */
const { useState, useEffect, useRef } = React;
const PAGE = window.MCP_SERVER_PAGE || 'create';

/* AUTH_META + KNOWN_SERVERS (registered servers, by id) come from
   mcp-catalog.jsx — the single source of truth loaded before this. */

/* deterministic DCR outcome — known hosts "support" it, others fall back to manual */
function attemptDcr(url) {
  try {
    const u = new URL(url);
    const ok = /(notion|linear|slack|github\.com|exa\.ai|supabase|deepwiki|google\.com|context7)/.test(u.hostname);
    return { ok, host: u.hostname, origin: u.origin };
  } catch (e) { return { ok: false, host: null, origin: '' }; }
}

function buildMechs(id) {
  const known = KNOWN_SERVERS[id] || { authConfigs: [] };
  const explicit = (known.authConfigs || []).filter(c => c.type !== 'none');
  return [{ ...PUBLIC_AC }, ...explicit];
}

/* ── Auth mechanism config block (create + inline add) ──
   Auth Type order: Header / Query Key first (no discovery), then OAuth.
   DCR fires ONLY when OAuth is chosen. */
function AuthBlock({ serverUrl, allowNone, onSummary }) {
  const [type, setType] = useState(allowNone ? 'none' : 'key');
  const [name, setName] = useState(allowNone ? 'public' : 'header-default');
  const [phase, setPhase] = useState('idle'); // idle | discovering | discovered | manual
  const [regType, setRegType] = useState('dcr');
  const [clientId, setClientId] = useState('');
  const [clientSecret, setClientSecret] = useState('');
  const [authEndpoint, setAuthEndpoint] = useState('');
  const [tokenEndpoint, setTokenEndpoint] = useState('');
  const [scopes, setScopes] = useState('');
  const [noUrl, setNoUrl] = useState(false);
  const [keys, setKeys] = useState([{ via: 'header', name: '' }]);
  const timer = useRef(null);

  const startDiscovery = () => {
    setPhase('discovering');
    clearTimeout(timer.current);
    timer.current = setTimeout(() => {
      const r = attemptDcr(serverUrl);
      if (r.ok) {
        setRegType('dcr'); setClientId('dcr_' + Math.random().toString(36).slice(2, 12)); setClientSecret('');
        setAuthEndpoint(r.origin + '/authorize'); setTokenEndpoint(r.origin + '/token'); setNoUrl(false); setPhase('discovered');
      } else { setRegType('pre'); setNoUrl(!r.host); setPhase('manual'); }
    }, 1500);
  };

  // DCR fires only when OAuth becomes the selected type (never on load — default is Header/Query)
  useEffect(() => { if (type === 'oauth') startDiscovery(); return () => clearTimeout(timer.current); }, [type]);

  useEffect(() => {
    if (!onSummary) return;
    let detail = 'No authentication required';
    if (type === 'oauth') detail = regType === 'dcr' ? 'Dynamic registration' : 'Pre-registered client';
    else if (type === 'key') { const k = keys[0] || {}; detail = (k.via === 'query' ? 'Query: ' : 'Header: ') + (k.name || '…'); }
    onSummary({ type, name: type === 'none' ? 'public' : name, detail, valid: !!type });
  }, [type, name, regType, keys]);

  const nameForType = t => t === 'oauth' ? 'oauth-default' : t === 'key' ? 'header-default' : 'public';
  const onTypeChange = t => { setType(t); setName(nameForType(t)); };

  return (
    <>
      <div className="bf-field">
        <label className="bf-label"><span className="bf-label-text">Type</span></label>
        <select className="bf-select" value={type} onChange={e => onTypeChange(e.target.value)}>
          {allowNone && <option value="none">None (Public)</option>}
          <option value="key">Header / Query Params</option>
          <option value="oauth">OAuth</option>
        </select>
      </div>

      {type === 'none' && (
        <div className="ns-public-note">
          <Ic name="unlock" size={15} />
          <div><strong>Public access</strong> — no authentication required. Any user with access to this workspace can connect without credentials.</div>
        </div>
      )}

      {type !== 'none' && (
        <div className="bf-field">
          <label className="bf-label"><span className="bf-label-text">Name</span></label>
          <input className="bf-input" value={name} onChange={e => setName(e.target.value)} placeholder={nameForType(type)} />
          <div className="bf-hint">Internal identifier for this auth mechanism.</div>
        </div>
      )}

      {type === 'key' && (<>
        <div className="bf-field" style={{ marginBottom: 8 }}><label className="bf-label"><span className="bf-label-text">Key Definitions</span></label></div>
        {keys.map((k, i) => (
          <div className="ns-keydef" key={i}>
            <select className="bf-select" value={k.via} onChange={e => setKeys(ks => ks.map((x, j) => j === i ? { ...x, via: e.target.value } : x))}>
              <option value="header">Header</option>
              <option value="query">Query</option>
            </select>
            <input className="bf-input" placeholder="e.g. Authorization" value={k.name}
              onChange={e => setKeys(ks => ks.map((x, j) => j === i ? { ...x, name: e.target.value } : x))} />
            <button className="ns-keydef-del" title="Remove key" disabled={keys.length === 1}
              onClick={() => setKeys(ks => ks.filter((_, j) => j !== i))} style={keys.length === 1 ? { opacity: .4, cursor: 'not-allowed' } : null}>
              <Ic name="trash-2" size={15} />
            </button>
          </div>
        ))}
        <button className="ns-add-key" onClick={() => setKeys(ks => [...ks, { via: 'header', name: '' }])}><Ic name="plus" size={14} /> Add Key</button>
      </>)}

      {type === 'oauth' && (<>
        {phase === 'discovering' && (
          <div className="ns-discovering"><span className="ns-spinner"></span>Discovering via dynamic client registration…</div>
        )}
        {phase === 'discovered' && (<>
          <div className="ns-banner success"><Ic name="check-circle" size={15} /><div><span className="ns-banner-title">Dynamic client registration succeeded.</span> Endpoints and a client ID were registered automatically — review and save.</div></div>
          <div className="bf-field"><label className="bf-label"><span className="bf-label-text">Registration Type</span></label><span className="ns-regpill"><Ic name="zap" size={12} /> Dynamic Client Registration</span></div>
          <div className="bf-field"><label className="bf-label"><span className="bf-label-text">Client ID</span></label><input className="bf-input bf-input-mono" value={clientId} onChange={e => setClientId(e.target.value)} /></div>
          <div className="bf-field"><label className="bf-label"><span className="bf-label-text">Authorization Endpoint</span></label><input className="bf-input" value={authEndpoint} onChange={e => setAuthEndpoint(e.target.value)} /></div>
          <div className="bf-field"><label className="bf-label"><span className="bf-label-text">Token Endpoint</span></label><input className="bf-input" value={tokenEndpoint} onChange={e => setTokenEndpoint(e.target.value)} /></div>
          <div className="bf-field"><label className="bf-label"><span className="bf-label-text">Scopes</span><span className="bf-optional">Optional</span></label><input className="bf-input" value={scopes} onChange={e => setScopes(e.target.value)} placeholder="e.g. mcp:tools mcp:read" /></div>
          <button className="ns-link" onClick={() => { setRegType('pre'); setPhase('manual'); setNoUrl(false); }}><Ic name="pencil" size={12} /> Enter client details manually instead</button>
        </>)}
        {phase === 'manual' && (<>
          <div className={'ns-banner ' + (noUrl ? 'info' : 'fail')}>
            <Ic name={noUrl ? 'info' : 'alert-triangle'} size={15} />
            <div>{noUrl
              ? <><span className="ns-banner-title">No server URL to discover from.</span> Provide the client details manually below.</>
              : <><span className="ns-banner-title">Dynamic client registration failed for this URL.</span> Enter the client details manually below.</>}</div>
          </div>
          <div className="bf-field">
            <label className="bf-label"><span className="bf-label-text">Registration Type</span></label>
            <select className="bf-select" value={regType} onChange={e => { const v = e.target.value; setRegType(v); if (v === 'dcr') startDiscovery(); }}>
              <option value="pre">Pre-Registered</option>
              <option value="dcr">Dynamic Client Registration</option>
            </select>
          </div>
          <div className="bf-field"><label className="bf-label"><span className="bf-label-text">Client ID</span></label><input className="bf-input" value={clientId} onChange={e => setClientId(e.target.value)} placeholder="Client ID" /></div>
          <div className="bf-field"><label className="bf-label"><span className="bf-label-text">Client Secret</span><span className="bf-optional">Optional</span></label><input className="bf-input" value={clientSecret} onChange={e => setClientSecret(e.target.value)} placeholder="Client Secret" /></div>
          <div className="bf-field"><label className="bf-label"><span className="bf-label-text">Authorization Endpoint</span></label><input className="bf-input" value={authEndpoint} onChange={e => setAuthEndpoint(e.target.value)} placeholder="https://auth.example.com/authorize" /></div>
          <div className="bf-field"><label className="bf-label"><span className="bf-label-text">Token Endpoint</span></label><input className="bf-input" value={tokenEndpoint} onChange={e => setTokenEndpoint(e.target.value)} placeholder="https://auth.example.com/token" /></div>
          <div className="bf-field"><label className="bf-label"><span className="bf-label-text">Scopes</span><span className="bf-optional">Optional</span></label><input className="bf-input" value={scopes} onChange={e => setScopes(e.target.value)} placeholder="e.g. mcp:tools mcp:read" /></div>
          {!noUrl && <button className="ns-link" onClick={startDiscovery}><Ic name="refresh-cw" size={12} /> Retry dynamic discovery</button>}
        </>)}
      </>)}
    </>
  );
}

/* ── Confirmation dialog ── */
function ConfirmDialog({ open, title, body, confirmLabel, onConfirm, onCancel }) {
  if (!open) return null;
  return (
    <div className="ns-cd-scrim" onClick={onCancel}>
      <div className="ns-cd" onClick={e => e.stopPropagation()}>
        <div className="ns-cd-title">{title}</div>
        <div className="ns-cd-body">{body}</div>
        <div className="ns-cd-actions">
          <button className="bf-btn bf-btn-ghost" onClick={onCancel}>Cancel</button>
          <button className="bf-btn ns-btn-danger" onClick={onConfirm}>{confirmLabel}</button>
        </div>
      </div>
    </div>
  );
}

/* shared shell wrapper */
function FormPage({ crumb, children }) {
  return (
    <AppShell
      section="mcp" subPage={PAGE === 'create' ? 'new-server' : 'my-mcps'} resizeKey="mcp"
      breadcrumb={[
        { label: 'Bodhi', href: 'Chat.html' },
        { label: 'MCP', href: 'MCP-My-MCPs.html' },
        { label: PAGE === 'create' ? 'Explore' : 'My MCPs', href: PAGE === 'create' ? 'MCP-Explore.html' : 'MCP-My-MCPs.html' },
        ...crumb,
      ]}
      contentClass="flush" mainScroll={false}
    >
      <div className="bf-scroll"><div className="bf-container">{children}</div></div>
    </AppShell>
  );
}

/* ════════ CREATE ════════ */
function CreateServerApp() {
  const params = new URLSearchParams(window.location.search);
  const [name, setName] = useState(params.get('name') || '');
  const [url, setUrl] = useState(params.get('url') || '');
  const [desc, setDesc] = useState('');
  const [enabled, setEnabled] = useState(true);
  const [authOpen, setAuthOpen] = useState(true);
  const [saving, setSaving] = useState(false);
  const [err, setErr] = useState(false);

  const save = () => {
    if (!url.trim() || !name.trim()) { setErr(true); return; }
    setSaving(true); setTimeout(() => { window.location.href = 'MCP-Explore.html'; }, 1000);
  };

  return (
    <FormPage crumb={[{ label: 'New Server', current: true }]}>
      <div className="bf-card">
        <div className="bf-card-head">
          <h1 className="bf-card-title">New MCP Server</h1>
          <p className="bf-card-sub">Register a new MCP server for users to connect to.</p>
        </div>
        <div className="bf-card-body">
          <div className="bf-field">
            <label className="bf-label"><span className="bf-label-text">URL</span><span className="bf-req">*</span></label>
            <input className={'bf-input' + (err && !url.trim() ? ' is-error' : '')} placeholder="https://mcp.example.com/mcp" value={url} onChange={e => setUrl(e.target.value)} />
          </div>
          <div className="bf-field">
            <label className="bf-label"><span className="bf-label-text">Name</span><span className="bf-req">*</span></label>
            <input className={'bf-input' + (err && !name.trim() ? ' is-error' : '')} placeholder="My MCP Server" value={name} onChange={e => setName(e.target.value)} />
          </div>
          <div className="bf-field">
            <label className="bf-label"><span className="bf-label-text">Description</span><span className="bf-optional">Optional</span></label>
            <textarea className="bf-textarea" placeholder="Optional description" value={desc} onChange={e => setDesc(e.target.value)}></textarea>
          </div>
          <div className="ns-enable">
            <div className={'bf-switch' + (enabled ? ' on' : '')} role="switch" aria-checked={enabled} onClick={() => setEnabled(v => !v)}></div>
            <span className="ns-enable-label">Enabled</span>
          </div>

          <div className="bf-divider"></div>
          <div className={'ns-collapse' + (authOpen ? ' open' : '')}>
            <button className="ns-collapse-head" onClick={() => setAuthOpen(o => !o)}>
              <span className="chev"><Ic name="chevron-right" size={16} /></span>
              Authentication Configuration <span style={{ color: 'hsl(var(--muted-foreground))', fontWeight: 500 }}>(Optional)</span>
            </button>
            {authOpen && <div className="ns-collapse-body"><AuthBlock serverUrl={url} allowNone={true} /></div>}
          </div>
          <div className="bf-hint" style={{ marginTop: 10 }}>Public access is always available. Configure OAuth or a header/query key only if the server requires it — you can add more mechanisms later from the server page.</div>
        </div>
        <div className="bf-footer">
          <div className="bf-footer-spacer"></div>
          <button className="bf-btn bf-btn-ghost" onClick={() => history.back()}>Cancel</button>
          <button className="bf-btn bf-btn-primary" onClick={save} disabled={saving}>{saving ? 'Saving…' : <><Ic name="server-cog" size={14} /> Save</>}</button>
        </div>
      </div>
    </FormPage>
  );
}

/* ════════ VIEW / CONFIGURE (per-section editing, inline add-auth) ════════ */
function ViewServerApp() {
  const params = new URLSearchParams(window.location.search);
  const id = params.get('server') || 'notion';
  const known = KNOWN_SERVERS[id] || { name: id, url: '', desc: '', enabled: true, authConfigs: [] };

  const [editing, setEditing] = useState(false);
  const [name, setName] = useState(known.name);
  const [desc, setDesc] = useState(known.desc);
  const [enabled, setEnabled] = useState(known.enabled);
  const [draftName, setDraftName] = useState(known.name);
  const [draftDesc, setDraftDesc] = useState(known.desc);
  const [draftEnabled, setDraftEnabled] = useState(known.enabled);
  const [saved, setSaved] = useState(false);
  const [mechs, setMechs] = useState(buildMechs(id));
  const [confirm, setConfirm] = useState(null);   // {name,label}
  const [adding, setAdding] = useState(false);
  const [addDraft, setAddDraft] = useState({ valid: false });

  const beginEdit = () => { setDraftName(name); setDraftDesc(desc); setDraftEnabled(enabled); setEditing(true); setSaved(false); };
  const saveBasic = () => { setName(draftName); setDesc(draftDesc); setEnabled(draftEnabled); setEditing(false); setSaved(true); setTimeout(() => setSaved(false), 2200); };

  const doDelete = () => { setMechs(ms => ms.filter(m => m.name !== confirm.name)); setConfirm(null); };
  const startAdd = () => { setAddDraft({ valid: false }); setAdding(true); };
  const saveAdd = () => {
    if (!addDraft.valid) return;
    setMechs(ms => [...ms, { type: addDraft.type, name: addDraft.name, detail: addDraft.detail }]);
    setAdding(false);
  };

  return (
    <FormPage crumb={[{ label: name, current: true }]}>
      <div className="bf-card">
        <div className="bf-card-head">
          <h1 className="bf-card-title">Configure server</h1>
          <p className="bf-card-sub">Manage <strong>{name}</strong> — edit basic details or its auth mechanisms independently.</p>
        </div>
        <div className="bf-card-body">

          {/* ── Basic information ── */}
          <div className="bf-section">
            <div className="ns-sec-head">
              <div className="bf-section-title" style={{ margin: 0 }}>Basic information</div>
              {!editing && <button className="ns-edit-btn" onClick={beginEdit}><Ic name="pencil" size={13} /> Edit</button>}
              {saved && <span className="ns-saved"><Ic name="check" size={13} /> Saved</span>}
            </div>

            {!editing ? (
              <div className="ns-read">
                <div className="ns-read-row"><span className="ns-read-k">Name</span><span className="ns-read-v">{name}</span></div>
                <div className="ns-read-row"><span className="ns-read-k">URL</span><span className="ns-read-v mono">{known.url}</span></div>
                <div className="ns-read-row"><span className="ns-read-k">Description</span><span className="ns-read-v" style={{ fontWeight: 400, color: 'hsl(var(--muted-foreground))' }}>{desc || '—'}</span></div>
                <div className="ns-read-row"><span className="ns-read-k">Status</span><span className={'ns-status ' + (enabled ? 'on' : 'off')}><Ic name={enabled ? 'circle-check' : 'circle-slash'} size={13} />{enabled ? 'Enabled' : 'Disabled'}</span></div>
              </div>
            ) : (
              <div className="ns-edit-form">
                <div className="bf-field">
                  <label className="bf-label"><span className="bf-label-text">URL</span><span className="ns-locked" style={{ marginLeft: 'auto' }}><Ic name="lock" size={11} /> locked</span></label>
                  <input className="bf-input" value={known.url} disabled />
                  <div className="bf-hint">The base URL is the server identity and can't be changed after creation.</div>
                </div>
                <div className="bf-field"><label className="bf-label"><span className="bf-label-text">Name</span><span className="bf-req">*</span></label><input className="bf-input" value={draftName} onChange={e => setDraftName(e.target.value)} /></div>
                <div className="bf-field"><label className="bf-label"><span className="bf-label-text">Description</span><span className="bf-optional">Optional</span></label><textarea className="bf-textarea" value={draftDesc} onChange={e => setDraftDesc(e.target.value)}></textarea></div>
                <div className="ns-enable" style={{ marginBottom: 4 }}>
                  <div className={'bf-switch' + (draftEnabled ? ' on' : '')} role="switch" aria-checked={draftEnabled} onClick={() => setDraftEnabled(v => !v)}></div>
                  <span className="ns-enable-label">Enabled</span>
                </div>
                <div className="ns-edit-actions">
                  <button className="bf-btn bf-btn-ghost" onClick={() => setEditing(false)}>Cancel</button>
                  <button className="bf-btn bf-btn-primary" onClick={saveBasic} disabled={!draftName.trim()}><Ic name="check" size={14} /> Save changes</button>
                </div>
              </div>
            )}
          </div>

          <div className="bf-divider"></div>

          {/* ── Auth mechanisms ── */}
          <div className="bf-section" style={{ marginBottom: 0 }}>
            <div className="bf-section-title">Auth mechanisms</div>
            <div className="bf-section-desc">Public is always available. Add OAuth or header/query keys for servers that require it. Mechanisms can be deleted but not edited — delete and re-add to change one.</div>

            <div className="ns-auth-list" style={{ marginTop: 0, marginBottom: 10 }}>
              {mechs.map((ac, i) => {
                const m = AUTH_META[ac.type] || AUTH_META.none;
                return (
                  <div className="ns-auth-row" key={ac.name + i}>
                    <div className="ns-auth-ico" style={{ background: m.iconBg }}><Ic name={m.icon} size={14} color={m.iconColor} /></div>
                    <div className="ns-auth-body">
                      <div className="ns-auth-name">{m.label}{!ac.builtin && <span style={{ fontFamily: 'var(--font-mono)', fontSize: 11.5, fontWeight: 500, color: 'hsl(var(--muted-foreground))' }}>{ac.name}</span>}{ac.builtin && <span className="ns-builtin">Built-in</span>}</div>
                      <div className="ns-auth-detail">{ac.detail}</div>
                    </div>
                    {!ac.builtin && <button className="ns-del" title="Delete auth mechanism" onClick={() => setConfirm({ name: ac.name, label: m.label })}><Ic name="trash-2" size={14} /></button>}
                  </div>
                );
              })}
            </div>

            {/* inline add (below the existing mechanisms): the button becomes a form in place; Cancel restores the button */}
            {adding ? (
              <div className="ns-add-form">
                <AuthBlock serverUrl={known.url} allowNone={false} onSummary={setAddDraft} />
                <div className="ns-add-form-actions">
                  <button className="bf-btn bf-btn-primary" onClick={saveAdd} disabled={!addDraft.valid}><Ic name="check" size={14} /> Save</button>
                  <button className="bf-btn bf-btn-secondary" onClick={() => setAdding(false)}>Cancel</button>
                </div>
              </div>
            ) : (
              <button className="ns-add-auth" onClick={startAdd}><Ic name="plus" size={14} /> Add auth mechanism</button>
            )}
          </div>
        </div>

        <div className="bf-footer">
          <div className="bf-footer-spacer"></div>
          <button className="bf-btn bf-btn-secondary" onClick={() => { window.location.href = 'MCP-My-MCPs.html'; }}><Ic name="arrow-left" size={14} /> Back to My MCPs</button>
        </div>
      </div>

      <ConfirmDialog open={!!confirm} title="Delete auth mechanism?"
        body={confirm ? <>This removes the <strong>{confirm.label}</strong> mechanism <code>{confirm.name}</code>. Existing user instances using it will stop working. This can't be undone.</> : null}
        confirmLabel="Delete" onConfirm={doDelete} onCancel={() => setConfirm(null)} />
    </FormPage>
  );
}

const Root = PAGE === 'view' ? ViewServerApp : CreateServerApp;
ReactDOM.createRoot(document.getElementById('root')).render(<Root />);
