/* ═══════════════════════════════════════════════════
   Bodhi MCP Discover — config slide-over
   mcp/mcp-discover-config.jsx   (load after mcp-discover-data.jsx)

   The admin-only slide-over drawer for editing server details or
   adding / editing an auth method (OAuth, header key, or public).
   Exports: ConfigDrawer.
═══════════════════════════════════════════════════ */
const { useState: useCfgState, useEffect: useCfgEffect } = React;

function ConfigDrawer({ state, onClose, onSave }) {
  const open = Boolean(state);
  const s = state ? state.server : null;
  const mode = state ? state.mode : null; // 'add' | number | 'edit-server'
  const existing = (s && typeof mode === 'number') ? s.authConfigs[mode] : null;
  const [authType, setAuthType] = useCfgState('oauth');
  const [authName, setAuthName] = useCfgState('');

  useCfgEffect(() => {
    if (!s) return;
    setAuthType(existing ? existing.type : 'oauth');
    setAuthName(existing ? existing.name : '');
  }, [state]);

  const isEditServer = mode === 'edit-server';
  const title = !s ? '' : isEditServer ? `Edit: ${s.name}` : typeof mode === 'number' ? `Edit Auth: ${existing.name}` : `Add Auth for ${s.name}`;
  const sub = isEditServer ? 'Update server URL, name, and settings' : typeof mode === 'number' ? 'Update authentication configuration' : 'Configure how users connect to this server';

  return (
    <div className="config-overlay" style={{ pointerEvents: open ? 'auto' : 'none' }}>
      <div className={'config-scrim' + (open ? ' visible' : '')} onClick={onClose}></div>
      <div className={'config-drawer' + (open ? ' open' : '')}>
        {s && (<>
          <div className="config-drawer-head">
            <div className="config-drawer-icon" style={{ background: s.iconBg }}><span style={{ color: s.iconColor, fontSize: 16, fontWeight: 800 }}>{s.icon}</span></div>
            <div className="config-drawer-title"><h2>{title}</h2><p>{sub}</p></div>
            <button className="icon-btn" onClick={onClose}><Ic name="x" size={14} /></button>
          </div>
          <div className="config-drawer-body">
            {isEditServer ? (<>
              <div className="form-field"><div className="form-label">URL <span className="req">*</span></div><input className="form-input" defaultValue={s.url} placeholder="https://mcp.example.com/mcp" /><div className="form-hint">The MCP server endpoint URL</div></div>
              <div className="form-field"><div className="form-label">Name <span className="req">*</span></div><input className="form-input" defaultValue={s.name} /></div>
              <div className="form-field"><div className="form-label">Description <span className="hint">(optional)</span></div><textarea className="form-textarea" defaultValue={s.desc}></textarea></div>
            </>) : (<>
              <div className="form-field">
                <div className="form-label">Auth Type <span className="req">*</span></div>
                <select className="form-select" value={authType} onChange={e => setAuthType(e.target.value)}>
                  <option value="oauth">OAuth 2.0</option>
                  <option value="key">Header / Query Key</option>
                  <option value="none">Public (No auth)</option>
                </select>
              </div>
              <div className="form-field"><div className="form-label">Config Name <span className="req">*</span></div><input className="form-input" value={authName} onChange={e => setAuthName(e.target.value)} placeholder="e.g. oauth-default" /><div className="form-hint">Internal identifier for this auth config</div></div>
              {authType === 'oauth' && (<>
                <div className="form-divider"></div>
                <div className="form-section-head">OAuth Configuration</div>
                <div className="form-field"><div className="form-label">Authorization Endpoint</div><input className="form-input" placeholder="https://example.com/authorize" defaultValue={existing?.authEndpoint || ''} /></div>
                <div className="form-field"><div className="form-label">Token Endpoint</div><input className="form-input" placeholder="https://example.com/token" defaultValue={existing?.tokenEndpoint || ''} /></div>
                <div className="form-field"><div className="form-label">Scopes <span className="hint">(optional)</span></div><input className="form-input" placeholder="read write" defaultValue={existing?.scopes || ''} /></div>
              </>)}
              {authType === 'key' && (<>
                <div className="form-divider"></div>
                <div className="form-section-head">Key / Value Configuration</div>
                <div className="form-field"><div className="form-label">Inject Via</div><select className="form-select"><option>Header</option><option>Query Parameter</option></select></div>
                <div className="form-field"><div className="form-label">Key name <span className="req">*</span></div><input className="form-input" placeholder="e.g. x-api-key" defaultValue={existing?.keyName || ''} /></div>
              </>)}
              {authType === 'none' && (
                <><div className="form-divider"></div>
                <div style={{ padding: 12, background: 'var(--c-leaf-bg)', border: '1px solid var(--c-leaf-bd)', borderRadius: 8, fontSize: 13, color: 'var(--c-leaf-text)' }}>
                  <strong>Public access</strong> — no authentication required. All users with access to this app can connect without providing any credentials.
                </div></>
              )}
            </>)}
          </div>
          <div className="config-drawer-foot">
            <button className="btn-cta-primary" onClick={() => onSave(s.id, isEditServer ? null : { type: authType, name: authName })}><Ic name="check" size={14} /> {isEditServer ? 'Save Changes' : 'Save Auth'}</button>
            <button className="btn-cta-secondary" onClick={onClose}>Cancel</button>
          </div>
        </>)}
      </div>
    </div>
  );
}

Object.assign(window, { ConfigDrawer });
