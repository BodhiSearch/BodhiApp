/* Setup · Step 5 — Browser Extension */

function Step5() {
  const [status, setStatus] = React.useState('notfound'); // notfound · checking · found

  const check = () => {
    setStatus('checking');
    setTimeout(() => setStatus('found'), 1300);
  };

  return (
    <SetupShell current={4}>
      <div className="su-rise">
        <section className="su-card">
          <div className="su-card-pad">
            <div className="su-card-head">
              <div className="su-feature-icon" style={{ margin: '0 auto 16px' }}><Icon name="puzzle" size={19} /></div>
              <h2 className="su-card-title">Browser Extension Setup</h2>
              <p className="su-card-sub">Install the Bodhi extension to unlock AI features on any website you visit.</p>
            </div>

            <div className="su-form-field">
              <p className="su-field-label">Browser</p>
              <div className="su-browser">
                <svg width="21" height="21" viewBox="0 0 24 24" aria-hidden="true">
                  <circle cx="12" cy="12" r="10" fill="none" stroke="currentColor" strokeWidth="1.6" />
                  <circle cx="12" cy="12" r="4" fill="none" stroke="currentColor" strokeWidth="1.6" />
                  <path d="M12 8h9M12 8l-4.5 7.8M12 8 7.5 .8M12 16l4.5 7.2M12 16 3 12" fill="none" stroke="currentColor" strokeWidth="1.2" opacity="0.55" />
                </svg>
                <span>Google Chrome</span>
                <span className="su-pill">Detected</span>
                <span className="su-chevron"><Icon name="chevron-down" size={18} /></span>
              </div>
            </div>

            <div className="su-link-row" style={{ marginBottom: 22 }}>
              <span className="su-link-label">Extension available in the Chrome Web Store</span>
              <a className="su-inline-link" href="#" onClick={(e) => e.preventDefault()}>
                Install Bodhi Extension <Icon name="arrow-up-right" size={14} />
              </a>
            </div>

            {status === 'found' ? (
              <div className="su-callout" style={{ background: 'hsl(var(--success) / .1)', borderColor: 'hsl(var(--success) / .35)' }}>
                <div className="su-feature-icon" style={{ margin: '0 auto 14px', background: 'hsl(var(--success) / .16)', color: 'hsl(var(--success))' }}>
                  <Icon name="check" size={19} strokeWidth={3} />
                </div>
                <h3 className="su-callout-title" style={{ color: 'hsl(var(--success))' }}>Extension Connected</h3>
                <p className="su-callout-sub">Bodhi is now active in your browser. You’re all set.</p>
              </div>
            ) : (
              <div className="su-callout">
                <h3 className="su-callout-title">Extension Not Found</h3>
                <p className="su-callout-sub">Install the extension, then verify the connection below.</p>
                <button className="su-btn su-btn-secondary" type="button" onClick={check} disabled={status === 'checking'}>
                  <Icon name="refresh-cw" size={16} /> {status === 'checking' ? 'Checking…' : 'Check Again'}
                </button>
              </div>
            )}
          </div>
        </section>

        <div className="su-info" style={{ marginTop: 22 }}>
          <p>The extension enables AI features directly in your browser tabs.</p>
          <p>You can always install it later from the Settings page.</p>
        </div>

        <div className="su-nav">
          <a className="su-btn su-btn-ghost" href="setup-4-api-models.html"><Icon name="arrow-left" size={17} /> Back</a>
          <span className="su-nav-spacer" />
          <a className="su-btn su-btn-secondary" href="setup-6-complete.html">Skip for now</a>
          <a className="su-btn su-btn-primary" href="setup-6-complete.html">Continue <Icon name="arrow-right" size={17} /></a>
        </div>
      </div>
    </SetupShell>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<Step5 />);
