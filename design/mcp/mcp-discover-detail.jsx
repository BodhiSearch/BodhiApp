/* ═══════════════════════════════════════════════════
   Bodhi MCP Discover — detail rail
   mcp/mcp-discover-detail.jsx   (load after mcp-catalog.jsx)

   The right-rail server detail panel — a SINGLE scrolling panel (no
   tabs): description + server details (incl. supported auth as a row) +
   my instances + connect-with / request. All actions live HERE, never on
   the cards/rows. The footer only carries admin's "Configure server" or
   the un-registered request/connect action; for a regular user on a
   configured server there's no footer button (they connect inline).
   Exports: SpecRow, DiscoverRailHeader, DetailPanel.
═══════════════════════════════════════════════════ */

function SpecRow({ k, v, small }) {
  return <div className="spec-row"><span className="spec-k">{k}</span><span className="spec-v" style={small ? { fontSize: 11, wordBreak: 'break-all' } : null}>{v}</span></div>;
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

/* ── Connect pieces ── */
function MyInstances({ s, actions }) {
  if (!s.userInstances.length) return null;
  return (
    <div className="p-section">
      <div className="p-sec-lbl">My Instances</div>
      {s.userInstances.map(inst => (
        <div className="user-instance-row" key={inst.id}>
          <span className={'ui-dot ' + inst.status}></span>
          <div className="ui-body">
            <div className="ui-name">{inst.name}</div>
            <div className="ui-meta">
              <span>{inst.authType === 'oauth' ? 'OAuth' : inst.authType === 'key' ? 'API Key' : 'Public'}</span>
              <span>·</span><span className="ui-cfg">{inst.authName}</span>
            </div>
          </div>
          <div className="ui-actions">
            {inst.status === 'connected'
              ? <button className="ui-play-btn" title="Playground" onClick={() => goToPlayground(inst.id, inst.name, s.id)}><Ic name="play" size={11} /></button>
              : <span className="ui-pending"><Ic name="clock" size={11} />Authorizing</span>}
            <button className="ui-icn-btn" title="Edit instance" onClick={() => goToEditInstance(inst, s.id)}><Ic name="pencil" size={12} /></button>
            <button className="ui-icn-btn danger" title="Delete instance" onClick={() => actions.deleteInstance(s.id, inst.id)}><Ic name="trash-2" size={12} /></button>
          </div>
        </div>
      ))}
    </div>
  );
}

function ConnectWith({ s }) {
  const mechs = availableAuth(s);
  return (
    <div className="p-section">
      <div className="p-sec-lbl">Connect with</div>
      <div className="p-sec-hint">Pick an auth mechanism to create a new instance.</div>
      {mechs.map(ac => {
        const m = AUTH_META[ac.type];
        const title = ac.type === 'oauth' ? 'OAuth' : ac.type === 'key' ? 'API Key' : 'Public';
        const sub = ac.builtin ? 'Always available · no setup' : (ac.name + (ac.detail ? ' · ' + ac.detail : ''));
        return (
          <div className="auth-connect-row" key={ac.name} onClick={() => goToNewInstance(s.id, ac.name)}>
            <div className="acr-icon" style={{ background: m.iconBg }}><Ic name={m.icon} size={13} color={m.iconColor} /></div>
            <div className="acr-body"><div className="acr-name">{title}</div><div className="acr-desc">{sub}</div></div>
            <div className="acr-arrow"><Ic name="chevron-right" size={14} /></div>
          </div>
        );
      })}
    </div>
  );
}

/* Non-admin request flow for an un-registered server */
function RequestPanel({ s, actions }) {
  if (s.requestStatus === 'pending') {
    return (
      <div className="connect-note pending">
        <Ic name="clock" size={15} />
        <div><div className="cn-title">Approval pending</div><div className="cn-sub">An admin has been notified. You'll be able to connect once this server is added.</div></div>
      </div>
    );
  }
  if (s.requestStatus === 'rejected') {
    return (
      <div className="p-section">
        <div className="connect-note rejected">
          <Ic name="x-circle" size={15} />
          <div><div className="cn-title">Request declined</div><div className="cn-sub">An admin declined adding this server. You can request again with a note.</div></div>
        </div>
      </div>
    );
  }
  return (
    <div className="connect-note neutral">
      <Ic name="info" size={15} />
      <div><div className="cn-title">Not in this workspace yet</div><div className="cn-sub">This server hasn't been added. Request it and an admin will review.</div></div>
    </div>
  );
}

function DetailPanel({ s, role, actions }) {
  const isAdmin = role === 'admin';
  const supported = (s.auth && s.auth.length) ? s.auth : ['none'];
  const foot = renderFoot(s, role, actions);

  return (
    <div className="mcp-detail">
      <div className="panel-body">
        <div className="p-section">
          <div className="p-sec-lbl">Description</div>
          <div style={{ fontSize: 13, lineHeight: 1.6, color: 'hsl(var(--muted-foreground))' }}>{s.desc}</div>
        </div>

        <div className="p-section">
          <div className="p-sec-lbl">Server</div>
          <div className="spec-table">
            <SpecRow k="URL" v={s.url} small />
            <SpecRow k="Transport" v={TRANSPORT_LABEL[s.transport] || s.transport} />
            <SpecRow k="Publisher" v={s.publisher} />
            <div className="spec-row spec-row-auth">
              <span className="spec-k">Supported auth</span>
              <span className="spec-auth"><AuthBadges auths={supported} /></span>
            </div>
          </div>
        </div>

        {s.disabled ? (
          <div className="connect-note neutral">
            <Ic name="ban" size={15} />
            <div><div className="cn-title">Disabled by admin</div><div className="cn-sub">This server is currently turned off for the whole workspace.</div></div>
          </div>
        ) : !s.registered ? (
          isAdmin ? (
            <div className="connect-note neutral">
              <Ic name="settings-2" size={15} />
              <div><div className="cn-title">Not configured yet</div><div className="cn-sub">Register this server to let users connect. The URL is pre-filled for you.</div></div>
            </div>
          ) : <RequestPanel s={s} actions={actions} />
        ) : (<>
          <MyInstances s={s} actions={actions} />
          <ConnectWith s={s} />
        </>)}
      </div>

      {foot && <div className="panel-foot">{foot}</div>}
    </div>
  );
}

/* Footer action — admin's Configure/Connect-Server or the un-registered
   request flow. A regular user on a configured server connects inline, so
   there's no footer button (returns null → footer is not rendered). */
function renderFoot(s, role, actions) {
  const isAdmin = role === 'admin';
  if (s.disabled) {
    return <>
      <button className="btn-full btn-disabled" disabled><Ic name="ban" size={14} /> Unavailable</button>
      {isAdmin && <button className="btn-full btn-ghost" onClick={() => goToViewServer(s.id)}><Ic name="settings-2" size={14} /> Configure server</button>}
    </>;
  }
  if (!s.registered) {
    // Admin: registering and connecting an un-added server are the same action → one button.
    if (isAdmin) return <button className="btn-full btn-lotus" onClick={() => goToNewServer({ url: s.url, name: s.name })}><Ic name="plus" size={14} /> Connect Server</button>;
    if (s.requestStatus === 'pending') return <button className="btn-full btn-disabled" disabled><Ic name="clock" size={14} /> Pending approval</button>;
    const label = s.requestStatus === 'rejected' ? 'Request again' : 'Request this server';
    return <button className="btn-full btn-lotus" onClick={() => actions.requestServer(s.id)}><Ic name="send" size={14} /> {label}</button>;
  }
  // registered / configured
  if (isAdmin) return <button className="btn-full btn-indigo" onClick={() => goToViewServer(s.id)}><Ic name="settings-2" size={14} /> Configure server</button>;
  return null;
}

Object.assign(window, { SpecRow, DiscoverRailHeader, DetailPanel });
