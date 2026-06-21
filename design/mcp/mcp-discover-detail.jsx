/* ═══════════════════════════════════════════════════
   Bodhi MCP Discover — detail rail
   mcp/mcp-discover-detail.jsx   (load after mcp-discover-data.jsx)

   The right-rail server detail panel and its pieces: spec row, footer
   CTA button, the compact rail header, and the tabbed DetailPanel
   (about / capabilities / connection / connect / configure / metadata).
   Exports: SpecRow, FootButton, DiscoverRailHeader, DetailPanel.
═══════════════════════════════════════════════════ */

function SpecRow({ k, v, small }) {
  return <div className="spec-row"><span className="spec-k">{k}</span><span className="spec-v" style={small ? { fontSize: 11, wordBreak: 'break-all' } : null}>{v}</span></div>;
}

function FootButton({ s, role, setTab }) {
  if (s.disabled) return <button className="btn-full btn-disabled" disabled><Ic name="ban" size={14} />Unavailable</button>;
  const st = userStatusSummary(s);
  if (role === 'admin') return <button className="btn-full btn-indigo" onClick={() => setTab('configure')}><Ic name="settings-2" size={14} /> Configure Server</button>;
  if (st === 'connected' || st === 'pending') return <button className="btn-full btn-leaf" onClick={() => setTab('connect')}><Ic name="plug" size={14} /> Manage Instances</button>;
  if (st === 'approved') return <button className="btn-full btn-lotus" onClick={() => setTab('connect')}><Ic name="plus" size={14} /> Connect to this server</button>;
  return <button className="btn-full btn-lotus"><Ic name="send" size={14} /> Request Approval</button>;
}

/* Compact rail header — sits on the shared 56px header gridline */
function DiscoverRailHeader({ s, setActiveId }) {
  return (
    <div className="rail-head">
      <div className="rail-head-icon" style={{ background: s.iconBg }}><span style={{ color: s.iconColor }}>{s.icon}</span></div>
      <div className="rail-head-body">
        <div className="rail-head-title">{s.name}</div>
        <div className="rail-head-pub">{s.publisher}{s.verified && <Ic name="badge-check" size={10} color="var(--c-indigo-text)" />}<span style={{ margin: '0 2px', opacity: .4 }}>·</span>{s.category}</div>
      </div>
      <button className="panel-close" onClick={() => setActiveId(null)}><Ic name="x" size={14} /></button>
    </div>
  );
}

function DetailPanel({ s, role, tab, setTab, onConfig, actions }) {
  const st = userStatusSummary(s);
  const isAdmin = role === 'admin';
  const hasReqs = s.approvalRequests.length > 0;

  let statusPill;
  if (s.disabled) statusPill = <span className="auth-badge" style={{ background: 'hsl(var(--muted))', borderColor: 'hsl(var(--border))', color: 'hsl(var(--muted-foreground))' }}><Ic name="ban" size={10} />Admin Disabled</span>;
  else if (st === 'connected') statusPill = <span className="auth-badge" style={{ background: 'var(--c-connected-bg)', borderColor: 'var(--c-connected-bd)', color: 'var(--c-connected-text)' }}><Ic name="circle-check" size={10} />Connected</span>;
  else if (st === 'pending') statusPill = <span className="auth-badge" style={{ background: 'var(--c-pending-bg)', borderColor: 'var(--c-pending-bd)', color: 'var(--c-pending-text)' }}><Ic name="clock" size={10} />Request pending</span>;
  else if (st === 'approved') statusPill = <span className="auth-badge" style={{ background: 'var(--c-indigo-bg)', borderColor: 'var(--c-indigo-bd)', color: 'var(--c-indigo-text)' }}><Ic name="shield-check" size={10} />Admin approved</span>;
  else if (!s.adminApproved && isAdmin) statusPill = <span className="auth-badge" style={{ background: 'var(--c-saffron-bg)', borderColor: 'var(--c-saffron-bd)', color: 'var(--c-saffron-text)' }}><Ic name="alert-circle" size={10} />Not configured</span>;
  else statusPill = <span className="auth-badge auth-none"><Ic name="minus-circle" size={10} />Not in this app</span>;

  const tabs = ['about', 'capabilities', 'connection'];
  if (!isAdmin) tabs.push('connect');
  if (isAdmin) tabs.push('configure');
  tabs.push('metadata');
  const tabLabel = { about: 'About', capabilities: 'Capabilities', connection: 'Connection', connect: 'Connect', configure: 'Configure', metadata: 'Metadata' };

  return (
    <div className="mcp-detail">
      <div className="panel-status-row">
        {statusPill}
        <AuthBadges auths={s.auth} />
        <span className="auth-badge auth-none" style={{ marginLeft: 'auto' }}><Ic name="download" size={10} />{s.installs}</span>
      </div>
      <div className="panel-tabs">
        {tabs.map(t => (
          <button key={t} className={'ptab' + (t === 'configure' ? ' admin-tab' : '') + (tab === t ? ' on' : '')} onClick={() => setTab(t)}>
            {tabLabel[t]}{t === 'configure' && hasReqs && <span className="ptab-dot"></span>}
          </button>
        ))}
      </div>
      <div className="panel-body">
        {tab === 'about' && (<>
          <div className="p-section"><div className="p-sec-lbl">Description</div><div style={{ fontSize: 13, lineHeight: 1.6, color: 'hsl(var(--muted-foreground))' }}>{s.desc}</div></div>
          <div className="p-section"><div className="p-sec-lbl">Tools ({s.toolList.length}{s.tools == null ? '+' : ''})</div>
            <div className="tool-list">{s.toolList.map(t => <div className="tool-item" key={t.name}><div className="tool-name">{t.name}</div><div className="tool-desc">{t.desc}</div></div>)}</div>
          </div>
          <div className="p-section"><div className="p-sec-lbl">Stats (30d)</div>
            <div className="stat-grid">{Object.entries(s.stats).map(([k, v]) => <div className="stat-box" key={k}><div className="stat-val">{v}</div><div className="stat-lbl">{k}</div></div>)}</div>
          </div>
        </>)}

        {tab === 'capabilities' && (<>
          <div className="p-section"><div className="p-sec-lbl">Exposed Tools</div>
            <div className="cap-chips">{s.toolList.map(t => <span key={t.name} className="tag tag-indigo" style={{ fontSize: 12, padding: '3px 9px', fontFamily: 'var(--font-mono)' }}>{t.name}</span>)}</div>
          </div>
          <div className="p-section"><div className="p-sec-lbl">Transport</div>
            <div className="spec-table"><SpecRow k="protocol" v={s.transport} /><SpecRow k="streaming" v={s.transport === 'streamable-http' ? 'Yes' : 'No'} /></div>
          </div>
        </>)}

        {tab === 'connection' && (<>
          <div className="p-section"><div className="p-sec-lbl">Endpoint</div>
            <div className="spec-table"><SpecRow k="URL" v={s.url} small /><SpecRow k="Transport" v={s.transport} /></div>
          </div>
          <div className="p-section"><div className="p-sec-lbl">Auth methods configured</div>
            {s.authConfigs.length ? s.authConfigs.map((ac, i) => {
              const m = AUTH_META[ac.type];
              return (
                <div className="auth-method-row" style={{ cursor: 'default' }} key={i}>
                  <div className="auth-method-icon" style={{ background: m.iconBg }}><Ic name={m.icon} size={13} color={m.iconColor} /></div>
                  <div className="auth-method-body"><div className="auth-method-name">{ac.name}</div><div className="auth-method-detail">{ac.detail}</div></div>
                </div>
              );
            }) : <div style={{ fontSize: 12.5, color: 'hsl(var(--muted-foreground))', padding: '8px 0' }}>{isAdmin ? 'No auth configured yet. Use the Configure tab to add one.' : 'No authentication configured by admin.'}</div>}
          </div>
        </>)}

        {tab === 'connect' && (<>
          {s.userInstances.length > 0 && (
            <div className="p-section"><div className="p-sec-lbl">My Instances</div>
              {s.userInstances.map(inst => (
                <div className="user-instance-row" key={inst.id}>
                  <span className={'ui-dot ' + inst.status}></span>
                  <div className="ui-body"><div className="ui-name">{inst.name}</div>
                    <div className="ui-meta"><span>{inst.authType === 'oauth' ? 'OAuth' : inst.authType === 'key' ? 'API Key' : 'Public'}</span><span>·</span><span>{inst.time}</span></div>
                  </div>
                  <div className="ui-actions">
                    {inst.status === 'connected'
                      ? <button className="ui-play-btn" onClick={() => goToPlayground(inst.id, inst.name, s.id)}><Ic name="play" size={11} />Playground</button>
                      : <span style={{ fontSize: 11.5, color: 'var(--c-pending-text)', fontWeight: 600 }}>Pending</span>}
                    <button className="ui-del-btn" title="Delete instance" onClick={() => actions.deleteInstance(s.id, inst.id)}><Ic name="trash-2" size={11} /></button>
                  </div>
                </div>
              ))}
            </div>
          )}
          <div className="p-section">
            <div className="p-sec-lbl">Connect with…</div>
            {s.adminApproved && s.authConfigs.map(ac => {
              const m = AUTH_META[ac.type];
              const label = ac.type === 'oauth' ? 'Connect with OAuth' : ac.type === 'key' ? 'Connect with API Key' : 'Connect (Public)';
              const desc = ac.type === 'oauth' ? 'Authorize via OAuth redirect' : ac.type === 'key' ? 'Provide your API key' : 'No authentication needed';
              return (
                <div className="auth-connect-row" key={ac.name} onClick={() => goToNewMCP(s.id, ac.type)}>
                  <div className="acr-icon" style={{ background: m.iconBg }}><Ic name={m.icon} size={13} color={m.iconColor} /></div>
                  <div className="acr-body"><div className="acr-name">{label}</div><div className="acr-desc">{desc}</div></div>
                  <div className="acr-arrow"><Ic name="chevron-right" size={13} /></div>
                </div>
              );
            })}
            {s.adminApproved && !s.authConfigs.some(ac => ac.type === 'none') && (
              <div className="auth-connect-row" onClick={() => goToNewMCP(s.id, 'none')}>
                <div className="acr-icon" style={{ background: 'var(--c-leaf-bg)' }}><Ic name="unlock" size={13} color="var(--c-leaf-text)" /></div>
                <div className="acr-body"><div className="acr-name">Connect (Public)</div><div className="acr-desc">No authentication required</div></div>
                <div className="acr-arrow"><Ic name="chevron-right" size={13} /></div>
              </div>
            )}
            {!s.adminApproved && <div style={{ fontSize: 12.5, color: 'hsl(var(--muted-foreground))', padding: '8px 0' }}>This server is not yet approved by your admin.</div>}
          </div>
        </>)}

        {tab === 'configure' && (<>
          {hasReqs && (<>
            <div className="p-section">
              <div className="p-sec-lbl" style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                <span>Approval Requests</span>
                <span style={{ background: 'var(--c-saffron-bg)', border: '1px solid var(--c-saffron-bd)', color: 'var(--c-saffron-text)', fontSize: 10, fontWeight: 700, padding: '1px 6px', borderRadius: 99 }}>{s.approvalRequests.length}</span>
              </div>
              {s.approvalRequests.map(req => (
                <div className="approval-request-row" key={req.email}>
                  <div className="arr-avatar">{req.initials}</div>
                  <div className="arr-body"><div className="arr-email">{req.email}</div><div className="arr-time">{req.time}</div></div>
                  <div className="arr-actions">
                    <button className="arr-approve" onClick={() => actions.approveRequest(s.id, req.email)}>Approve</button>
                    <button className="arr-reject" onClick={() => actions.rejectRequest(s.id, req.email)}>Reject</button>
                  </div>
                </div>
              ))}
            </div>
            <div className="form-divider"></div>
          </>)}
          <div className="p-section">
            <div className="p-sec-lbl">Server Status</div>
            <div className="form-toggle-row">
              <div><div className="form-toggle-label">Enabled globally</div><div style={{ fontSize: 11.5, color: 'hsl(var(--muted-foreground))', marginTop: 2 }}>Allow users to connect to this server</div></div>
              <div className={'sw' + (!s.disabled ? ' on' : '')} onClick={() => actions.toggleDisabled(s.id)}></div>
            </div>
            {!s.adminApproved ? (
              <div className="form-toggle-row" style={{ marginTop: 8 }}>
                <div><div className="form-toggle-label">Admin approved</div><div style={{ fontSize: 11.5, color: 'hsl(var(--muted-foreground))', marginTop: 2 }}>Make visible and connectable to users</div></div>
                <div className={'sw' + (s.adminApproved ? ' on' : '')} onClick={() => actions.toggleApproved(s.id)}></div>
              </div>
            ) : (
              <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginTop: 8, padding: '8px 10px', background: 'var(--c-leaf-bg)', border: '1px solid var(--c-leaf-bd)', borderRadius: 8, fontSize: 12.5, fontWeight: 600, color: 'var(--c-leaf-text)' }}>
                <Ic name="shield-check" size={13} /> Approved & active
              </div>
            )}
          </div>
          <div className="p-section">
            <div className="p-sec-lbl" style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <span>Authentication Methods</span>
              <button style={{ fontSize: 11.5, fontWeight: 600, color: 'var(--c-indigo-text)', background: 'none', border: 'none', display: 'flex', alignItems: 'center', gap: 3 }} onClick={() => onConfig(s, 'add')}><Ic name="plus" size={11} /> Add</button>
            </div>
            {s.authConfigs.length ? s.authConfigs.map((ac, i) => {
              const m = AUTH_META[ac.type];
              return (
                <div className="auth-method-row" key={i}>
                  <div className="auth-method-icon" style={{ background: m.iconBg }}><Ic name={m.icon} size={13} color={m.iconColor} /></div>
                  <div className="auth-method-body"><div className="auth-method-name">{ac.name}</div><div className="auth-method-detail">{ac.detail}</div></div>
                  <div className="auth-method-actions">
                    <button className="auth-method-btn" title="Edit" onClick={() => onConfig(s, i)}><Ic name="pencil" size={12} /></button>
                    <button className="auth-method-btn danger" title="Delete" onClick={() => actions.deleteAuthConfig(s.id, i)}><Ic name="trash-2" size={12} /></button>
                  </div>
                </div>
              );
            }) : <div style={{ fontSize: 12.5, color: 'hsl(var(--muted-foreground))', padding: 12, border: '1px dashed hsl(var(--border))', borderRadius: 8, textAlign: 'center' }}>No auth configured. Click <strong>Add</strong> to set up OAuth, API Key, or leave as Public.</div>}
          </div>
        </>)}

        {tab === 'metadata' && (
          <div className="p-section"><div className="p-sec-lbl">Metadata</div>
            <div className="spec-table">
              <SpecRow k="license" v={s.meta.license} /><SpecRow k="repo" v={s.meta.repo} /><SpecRow k="publisher" v={s.publisher} />
              <SpecRow k="verified" v={s.verified ? '✓ Yes' : 'No'} /><SpecRow k="transport" v={s.transport} />
            </div>
          </div>
        )}
      </div>
      <div className="panel-foot">
        {tab === 'configure'
          ? <><button className="btn-full btn-indigo" onClick={() => onConfig(s, 'add')}><Ic name="plus" size={14} /> Add Auth Method</button>
              <button className="btn-full btn-ghost" onClick={() => onConfig(s, 'edit-server')}><Ic name="settings-2" size={14} /> Edit Server Details</button></>
          : <FootButton s={s} role={role} setTab={setTab} />}
      </div>
    </div>
  );
}

Object.assign(window, { SpecRow, FootButton, DiscoverRailHeader, DetailPanel });
