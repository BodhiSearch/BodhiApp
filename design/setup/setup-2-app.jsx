/* Setup · Step 2 — Login & Admin Setup */

const ADMIN_CAPS = [
  'Manage user access and permissions',
  'Unrestricted access to system-wide settings',
  'Review and approve incoming access requests',
];

function Step2() {
  return (
    <SetupShell current={1}>
      <section className="su-card su-rise">
        <div className="su-card-pad">
          <div className="su-card-head">
            <div className="su-feature-icon" style={{ margin: '0 auto 16px' }}>
              <Icon name="shield-check" size={19} />
            </div>
            <h2 className="su-card-title">Admin Setup</h2>
            <p className="su-card-sub">
              You’re setting up Bodhi in authenticated mode. The email address you log in with
              will be granted the admin role for this instance.
            </p>
          </div>

          <div className="su-well">
            <h3 className="su-well-title">As an admin, you can</h3>
            <ul className="su-checklist">
              {ADMIN_CAPS.map((c) => (
                <li key={c}>
                  <span className="su-check"><Icon name="check" size={15} strokeWidth={3} /></span>
                  <span>{c}</span>
                </li>
              ))}
            </ul>
          </div>

          <a className="su-btn su-btn-primary is-block" href="setup-3-local-models.html">
            Continue with Login <Icon name="arrow-right" size={17} />
          </a>
          <p className="su-note" style={{ marginTop: 14 }}>Log in with a valid email address to continue.</p>
        </div>
      </section>
    </SetupShell>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<Step2 />);
