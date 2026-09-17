/* ═══════════════════════════════════════════════════
   REMOTE ACCESS — one page, three steps
   tunnels/ra-app.jsx
   Steps 1–2 establish the local setup; step 3 takes a subdomain,
   creates the tunnel inline and then holds the live controls.
   All troubleshooting and background lives in the FAQ rail (help icon, top right).
═══════════════════════════════════════════════════ */
const Ic = RA_Ic;

const RA_STATES = [
{ id: 'unavailable', label: 'Not available here', tone: 'idle', n: 'A1' },
{ id: 'binary-missing', label: 'cloudflared not found', tone: 'attn', n: 'B2' },
{ id: 'binary-old', label: 'Version unsupported', tone: 'attn', n: 'B4' },
{ id: 'path-invalid', label: 'Given path rejected', tone: 'fail', n: 'B2a' },
{ id: 'login-needed', label: 'Not signed in', tone: 'attn', n: 'B7' },
{ id: 'login-failed', label: 'Certificate rejected', tone: 'fail', n: 'B9' },
{ id: 'address', label: 'Ready for a subdomain', tone: 'idle', n: 'B5' },
{ id: 'creating', label: 'Creating the tunnel', tone: 'checking', n: 'C1' },
{ id: 'create-failed', label: 'Couldn’t set up the tunnel', tone: 'fail', n: 'C2' },
{ id: 'dns-conflict', label: 'Address already in use', tone: 'attn', n: 'C3' },
{ id: 'kc-syncing', label: 'Syncing sign-in redirect', tone: 'checking', n: 'C4' },
{ id: 'kc-fail-net', label: 'Keycloak unreachable', tone: 'attn', n: 'C5' },
{ id: 'kc-fail-auth', label: 'Keycloak rejected the change', tone: 'attn', n: 'C6' },
{ id: 'live', label: 'Remote access on', tone: 'ok', n: 'C7' },
{ id: 'off', label: 'Remote access off', tone: 'idle', n: 'C8' }];


const DETECTED = '/opt/homebrew/bin/cloudflared';
const CERT = '~/.cloudflared/cert.pem';
const SETUP_DONE = ['address', 'creating', 'create-failed', 'dns-conflict', 'kc-syncing', 'kc-fail-net', 'kc-fail-auth', 'live', 'off'];
/* Tunnel is up (whatever the Keycloak sync did) — local setup can't be re-cut */
const LIVE = ['kc-syncing', 'kc-fail-net', 'kc-fail-auth', 'live'];

/* Keycloak redirect-URL sync — runs once the tunnel connects. The tunnel
   stays up if it fails; only sign-in through the address is broken. */
const KC_FAIL = {
  'kc-fail-net': {
    title: 'Couldn’t reach Keycloak to update the sign-in redirect',
    body: <>The tunnel is up, but signing in through the address will be refused until the redirect URL is registered.</>
  },
  'kc-fail-auth': {
    title: 'Keycloak refused the redirect URL change',
    body: <>This instance isn’t allowed to edit its own Keycloak client, so the redirect URL wasn’t added. Signing in through the address will be refused until it is.</>
  }
};

function KeycloakSync({ state, setState, reveal, host }) {
  if (state === 'kc-syncing') {
    return (
      <RANote tone="checking" title="Syncing the sign-in redirect URL with Keycloak">
        Registering <span className="ra-mono">https://{host}/auth/callback</span> so sign-in works through the tunnel.
      </RANote>);

  }
  if (state === 'live') {
    return (
      <div className="ra-kc-ok">
        <Ic name="check" size={13} /> You can sign in from the remote access URL
      </div>);

  }
  const f = KC_FAIL[state];
  if (!f) return null;
  return (
    <RANote tone="warn" title={f.title}>
      {f.body}
      <div className="ra-actions">
        <button className="bf-btn bf-btn-primary" onClick={() => setState('kc-syncing')}>
          <Ic name="refresh-cw" size={13} /> Retry sync
        </button>
        <button className="bf-btn bf-btn-ghost" onClick={() => reveal('faq-kcsync')}>Why this is needed</button>
      </div>
    </RANote>);

}

/* ── A path we normally find ourselves, which the admin can override ── */
function PathField({ label, value, onChange, error, detected, placeholder, foundHint, missingHint }) {
  return (
    <div className="bf-field ra-pathfield">
      <div className="bf-label">
        <span className="bf-label-text">{label}</span>
        <span className="bf-optional"> · optional</span>
      </div>
      <input className="bf-input bf-input-mono" placeholder={detected || placeholder}
      value={value} onChange={(e) => onChange(e.target.value)} spellCheck={false} />
      <div className="bf-hint">{detected ? foundHint : missingHint}</div>
      {error && <div className="ra-err"><Ic name="alert-octagon" size={13} /> {error}</div>}
    </div>);

}

const BIN_FIELD = {
  label: 'Binary location', placeholder: '/usr/local/bin/cloudflared',
  foundHint: <>Found at <span className="ra-mono">/opt/homebrew/bin/cloudflared</span>.</>,
  missingHint: 'Not found. Enter the full path to use your own.'
};

const CERT_FIELD = {
  label: 'Certificate location', placeholder: CERT,
  foundHint: <>Found at <span className="ra-mono">{CERT}</span>.</>,
  missingHint: <>Nothing at <span className="ra-mono">{CERT}</span>. Enter the full path to use your own.</>
};

/* Folded steps offer an Edit affordance; while remote access is on,
   changing the local setup would tear the tunnel down, so it's refused. */
function EditAction({ locked, onLocked }) {
  if (locked) {
    return (
      <button className="bf-btn bf-btn-ghost bf-btn-sm" onClick={(e) => {e.stopPropagation();onLocked();}}>
        <Ic name="lock" size={12} /> Locked
      </button>);

  }
  return <span className="ra-fold-edit"><Ic name="pencil" size={12} /> Edit</span>;
}

/* ── Step 1 · cloudflared ── */
function BinaryStep({ state, path, setPath, reveal, onLocked }) {
  const done = ['login-needed', 'login-failed', ...SETUP_DONE].includes(state);
  const locked = LIVE.includes(state);
  if (done) {
    return (
      <RAStep n={1} tone="ok" title="cloudflared" pill="Ready" collapsible
      defaultFolded={SETUP_DONE.includes(state)} lockFolded={locked}
      summary={DETECTED} action={<EditAction locked={locked} onLocked={onLocked} />}
      status={<>Version <span className="ra-mono">2026.8.1</span> at <span className="ra-mono">{DETECTED}</span>.</>}>
        <PathField {...BIN_FIELD} value={path} onChange={setPath} detected={DETECTED} />
        <div className="ra-actions">
          <button className="bf-btn bf-btn-secondary"><Ic name="refresh-cw" size={13} /> Check again</button>
        </div>
      </RAStep>);

  }

  const copy = {
    'binary-missing': {
      tone: 'attn', pill: 'Not found',
      status: 'Remote access needs the cloudflared program, and it isn’t here yet.',
      cta: { label: 'How to install it', icon: 'book-open', faq: 'faq-install' },
      alt: { label: 'It’s installed but not detected', faq: 'faq-notdetected' }
    },
    'binary-old': {
      tone: 'attn', pill: 'Too old',
      status: <>Version <span className="ra-mono">2023.8.2</span> is older than Cloudflare supports.</>,
      detected: DETECTED,
      cta: { label: 'How to update it', icon: 'arrow-up-circle', faq: 'faq-update' }
    },
    'path-invalid': {
      tone: 'fail', pill: 'Path rejected',
      status: 'The path you gave couldn’t be used, so it wasn’t saved.',
      err: 'Not a runnable cloudflared binary.',
      cta: { label: 'Pointing at a binary manually', icon: 'book-open', faq: 'faq-path' }
    }
  }[state];

  return (
    <RAStep n={1} tone={copy.tone} title="cloudflared" pill={copy.pill} status={copy.status}>
      <PathField {...BIN_FIELD} value={path} onChange={setPath} detected={copy.detected} error={copy.err} />
      <div className="ra-actions">
        <button className="bf-btn bf-btn-primary" onClick={() => reveal(copy.cta.faq)}>
          <Ic name={copy.cta.icon} size={13} /> {copy.cta.label}
        </button>
        <button className="bf-btn bf-btn-secondary"><Ic name="refresh-cw" size={13} /> Check again</button>
        {copy.alt &&
        <button className="bf-btn bf-btn-ghost" onClick={() => reveal(copy.alt.faq)}>{copy.alt.label}</button>}
      </div>
    </RAStep>);

}

/* ── Step 2 · Cloudflare sign-in, done by the admin in a terminal.
   Signing in writes a certificate; that file is what we check for. ── */
function LoginStep({ state, cert, setCert, reveal, onLocked }) {
  const blocked = ['binary-missing', 'binary-old', 'path-invalid'].includes(state);
  const locked = LIVE.includes(state);
  if (blocked) {
    return <RAStep n={2} tone="idle" title="Cloudflare sign-in" pill="Waiting"
    status="Checked once cloudflared is ready." />;
  }
  if (state === 'login-needed' || state === 'login-failed') {
    const failed = state === 'login-failed';
    return (
      <RAStep n={2} tone={failed ? 'fail' : 'attn'} title="Cloudflare sign-in"
      pill={failed ? 'Not confirmed' : 'Signed out'}
      status={failed ?
      'That certificate couldn’t be used, so it wasn’t saved.' :
      'The domain you pick while signing in is the one your address sits on.'}>
        <PathField {...CERT_FIELD} value={cert} onChange={setCert}
        error={failed ? 'Not a usable Cloudflare certificate.' : null} />
        <div className="ra-runline">To log in and generate the certificate, run this in your terminal:</div>
        <RACode>cloudflared tunnel login</RACode>
        <div className="ra-actions">
          <button className="bf-btn bf-btn-primary"><Ic name="refresh-cw" size={13} /> Check again</button>
          <button className="bf-btn bf-btn-ghost" onClick={() => reveal(failed ? 'faq-cert' : 'faq-signin')}>
            How we detect Cloudflare login
          </button>
        </div>
      </RAStep>);

  }
  return (
    <RAStep n={2} tone="ok" title="Cloudflare sign-in" pill="Ready" collapsible
    defaultFolded={SETUP_DONE.includes(state)} lockFolded={locked}
    summary={RA_ZONE} action={<EditAction locked={locked} onLocked={onLocked} />}
    status={<>Signed in, on <strong>{RA_ZONE}</strong>.</>}>
      <PathField {...CERT_FIELD} value={cert} onChange={setCert} detected={CERT} />
      <div className="ra-actions">
        <button className="bf-btn bf-btn-secondary"><Ic name="refresh-cw" size={13} /> Check again</button>
        <button className="bf-btn bf-btn-ghost" onClick={() => reveal('faq-zone')}>Using a different domain</button>
      </div>    </RAStep>);

}

/* ── Inline creation progress ── */
const RA_STAGES = [
{ id: 'start', label: 'Starting up' },
{ id: 'connect', label: 'Connecting to Cloudflare' },
{ id: 'done', label: 'Connected' }];

function Progress({ at = 1 }) {
  return (
    <div className="ra-prog">
      {RA_STAGES.map((s, i) => {
        const st = i < at ? 'done' : i === at ? 'now' : 'next';
        return (
          <div className={`ra-prog-row is-${st}`} key={s.id}>
            <span className="ra-prog-ic">
              {st === 'done' ? <Ic name="check" size={12} /> :
              st === 'now' ? <span className="ra-spin" style={{ display: 'inline-flex' }}><Ic name="refresh-cw" size={12} /></span> :
              <span className="ra-prog-dot" />}
            </span>
            {s.label}
          </div>);

      })}
    </div>);

}

/* ── Step 3 · remote access itself. The form never changes shape:
   subdomain, auto-connect, actions. Turning on only locks the field
   and adds Open / Copy. ── */
function AddressStep({ state, setState, reveal, lockNote }) {
  const [sub, setSub] = React.useState('bodhi');
  const [err, setErr] = React.useState(null);
  const [confirming, setConfirming] = React.useState(false);
  const [auto, setAuto] = React.useState(true);
  const host = `${sub.trim() || 'bodhi'}.${RA_ZONE}`;

  if (!SETUP_DONE.includes(state)) {
    return <RAStep n={3} tone="idle" title="Remote access" pill="Waiting"
    status="Available once the two checks above pass." />;
  }

  const on = LIVE.includes(state);
  const kcFail = !!KC_FAIL[state];
  const busy = state === 'creating';
  const locked = on || busy;

  const submit = () => {
    const v = sub.trim();
    if (!v) return setErr('Enter a subdomain.');
    if (v.includes('.')) return setErr('One level only — no dots. Use a single word such as “bodhi”.');
    if (!/^[a-z0-9]([a-z0-9-]*[a-z0-9])?$/i.test(v)) return setErr('Letters, numbers and hyphens only.');
    setErr(null);
    setConfirming(true);
  };

  const tone = state === 'live' ? 'ok' : kcFail ? 'attn' : state === 'kc-syncing' ? 'checking' :
  state === 'create-failed' ? 'fail' : state === 'dns-conflict' ? 'attn' : busy ? 'checking' : 'idle';
  const pill = state === 'live' ? 'On' : state === 'kc-syncing' ? 'On · syncing' :
  kcFail ? 'On · sign-in broken' : state === 'off' ? 'Off' :
  state === 'create-failed' ? 'Failed' : state === 'dns-conflict' ? 'Address in use' :
  busy ? 'Creating' : 'Not set';

  return (
    <RAStep n={3} tone={tone} title="Remote access" pill={pill} spin={busy || state === 'kc-syncing'}
    status={on ? undefined : <>Pick the address this instance answers on, under <span className="ra-mono">{RA_ZONE}</span>.</>}>
      <div className={`bf-field${locked ? ' is-locked' : ''}`}>
        <div className="bf-label"><span className="bf-label-text">Subdomain</span>{!locked && <span className="bf-req">*</span>}</div>
        <div className="ra-subdomain">
          <input className="bf-input bf-input-mono" placeholder="bodhi" value={sub} spellCheck={false} disabled={locked}
          onChange={(e) => {setSub(e.target.value);setErr(null);}} />
          <span className="ra-subdomain-sfx">.{RA_ZONE}</span>
          {on &&
          <div className="ra-subdomain-acts">
              <button className="bf-btn bf-btn-secondary bf-btn-icon" title="Open" aria-label={`Open https://${host}`}><Ic name="external-link" size={14} /></button>
              <button className="bf-btn bf-btn-secondary bf-btn-icon" title="Copy address" aria-label="Copy address"><Ic name="copy" size={14} /></button>
            </div>}
        </div>
        {err && <div className="ra-err"><Ic name="alert-octagon" size={13} /> {err}</div>}
      </div>

      <label className="bf-check-row ra-auto">
        <input type="checkbox" className="bf-checkbox" checked={auto} onChange={() => setAuto(!auto)} />
        <span className="bf-check-label">Connect automatically after BodhiApp restarts</span>
      </label>

      {busy && <Progress at={1} />}

      <KeycloakSync state={state} setState={setState} reveal={reveal} host={host} />

      {state === 'create-failed' &&
      <RANote tone="fail" title="Couldn’t set up the tunnel">
          Cloudflare didn’t accept the request.
          <div className="ra-actions">
            <button className="bf-btn bf-btn-primary" onClick={() => setState('creating')}><Ic name="refresh-cw" size={13} /> Try again</button>
            <button className="bf-btn bf-btn-ghost" onClick={() => reveal('faq-createfail')}>Common causes</button>
          </div>
        </RANote>}

      {state === 'dns-conflict' &&
      <RANote tone="warn" title={`Something already answers at ${host}`}>
          Replace that DNS record, or pick a different subdomain.
          <div className="ra-actions">
            <button className="bf-btn bf-btn-primary" onClick={() => setState('creating')}>Replace it and continue</button>
            <button className="bf-btn bf-btn-ghost" onClick={() => reveal('faq-dns')}>What gets replaced</button>
          </div>
        </RANote>}

      {on &&
      <div className="ra-actions">
          <button className="bf-btn bf-btn-secondary" onClick={() => setState('off')}>
            <Ic name="power" size={13} /> Turn off
          </button>
        </div>}

      {state === 'off' &&
      <div className="ra-actions">
          <button className="bf-btn bf-btn-primary" onClick={() => setState('kc-syncing')}>
            <Ic name="power" size={13} /> Turn on
          </button>
          <span className="ra-admin"><Ic name="users" size={12} /> Applies to everyone on this instance</span>
        </div>}

      {state === 'address' && (confirming ?
      <div className="ra-confirm">
            <RAExposure host={host} onMore={() => reveal('faq-exposure')} />
            <div className="ra-actions">
              <button className="bf-btn bf-btn-primary" onClick={() => setState('creating')}>
                <Ic name="power" size={13} /> Yes, create it
              </button>
              <button className="bf-btn bf-btn-ghost" onClick={() => setConfirming(false)}>Cancel</button>
            </div>
          </div> :

      <div className="ra-actions">
            <button className="bf-btn bf-btn-primary" onClick={submit}><Ic name="cloud-cog" size={13} /> Create tunnel</button>
            <span className="ra-admin"><Ic name="users" size={12} /> Applies to everyone on this instance</span>
          </div>)
      }

      {lockNote && <RANote tone="info" title="Turn remote access off first">A different binary or domain means a new tunnel.</RANote>}
    </RAStep>);

}

function Unavailable({ reveal }) {
  return (
    <div className="bf-card">
      <div className="ra-unavail">
        <div className="ra-unavail-ic"><Ic name="ban" size={20} /></div>
        <div className="ra-unavail-t">Remote access isn’t available on this deployment</div>
        <div className="ra-unavail-s">This instance can’t run the tunnel program.</div>
        <div className="ra-actions" style={{ justifyContent: 'center' }}>
          <button className="bf-btn bf-btn-secondary" onClick={() => reveal('faq-unavailable')}>Why, and what to do instead</button>
        </div>
      </div>
    </div>);

}

function RemoteAccessApp() {
  const [state, setState] = React.useState('binary-missing');
  const [path, setPath] = React.useState('');
  const [cert, setCert] = React.useState('');
  const [lockNote, setLockNote] = React.useState(false);
  const { faqProps, reveal } = useRAFaq();
  const cur = RA_STATES.find((s) => s.id === state);
  const onLocked = () => setLockNote(true);

  React.useEffect(() => {setLockNote(false);}, [state]);

  const pill = {
    live: 'On', off: 'Off', creating: 'Creating', unavailable: 'Unavailable',
    'kc-syncing': 'Finishing up', 'kc-fail-net': 'Sign-in broken', 'kc-fail-auth': 'Sign-in broken'
  }[state] || (cur.tone === 'fail' || cur.tone === 'attn' ? 'Needs attention' : 'Not set up');

  return (
    <AppShell
      section="settings" subPage="remote-access" resizeKey="settings"
      breadcrumb={RA_BC}
      sidebar={<RASidebar states={RA_STATES} active={state} onPick={setState} />}
      rail={<RAFaqRail {...faqProps} />}
      railHeader={<RAFaqRailHeader />}
      railDefaultOpen={false}
      railWidth={360} railMin={300} railMax={520}
      railToggleIcon="circle-help" railToggleTitle="Help & debugging"
      contentClass="flush" mainScroll={false}>
      
      <div className="bf-scroll">
        <div className="bf-container">
          <RAPageHeader note="Admin-only, instance-wide." />
          {state === 'unavailable' ?
          <Unavailable reveal={reveal} /> :

          <div className="bf-card">
              <div className="bf-card-head">
                <div className="ra-card-head-row">
                  <div>
                    <div className="bf-card-title">Set up remote access</div>
                    <div className="bf-card-sub">Three steps, on this machine.</div>
                  </div>
                  <RAPill tone={cur.tone} label={pill} spin={cur.tone === 'checking'} />
                </div>
              </div>
              <div className="bf-card-body">
                <div className="ra-ladder">
                  <BinaryStep state={state} path={path} setPath={setPath} reveal={reveal} onLocked={onLocked} />
                  <LoginStep state={state} cert={cert} setCert={setCert} reveal={reveal} onLocked={onLocked} />
                  <AddressStep state={state} setState={setState} reveal={reveal} lockNote={lockNote} />
                </div>
              </div>
            </div>
          }
        </div>
      </div>
    </AppShell>);

}

ReactDOM.createRoot(document.getElementById('root')).render(<RemoteAccessApp />);
