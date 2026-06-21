/* Setup · Step 1 — Get Started (welcome + features + server setup) */

const FEATURES = [
  { icon: 'lock',         title: 'Complete Privacy',      body: 'Your data stays under your control. Choose local models for maximum privacy, or connect to APIs you trust.' },
  { icon: 'wallet',       title: 'Cost Freedom',          body: 'Run unlimited local AI without fees. Bring your own API keys for cloud models — you control the costs.' },
  { icon: 'blend',        title: 'Hybrid Flexibility',    body: 'Run local models on your own hardware, or connect to OpenAI, Anthropic, and other API providers.', isNew: true },
  { icon: 'users-round',  title: 'Multi-User Ready',      body: 'Built for teams and families. Role-based access with admin controls and full user management.', isNew: true },
  { icon: 'globe',        title: 'Browser AI Revolution', body: 'Enable AI on any website through our browser extension. Publishers save costs; users get enhanced experiences.', isNew: true },
  { icon: 'sparkles',     title: 'Open Ecosystem',        body: 'Powered by llama.cpp. Compatible with HuggingFace models, OpenAI-style APIs, and more.' },
];

function Step1() {
  const [name, setName] = React.useState('');

  return (
    <SetupShell current={0}>
      <div className="su-stagger">
        <header className="su-hero">
          <h1 className="su-hero-title">Welcome to <span className="su-sanskrit">बोधि</span> Bodhi</h1>
          <p className="su-lead">Your personal AI hub — local, remote, and everywhere. “Bodhi” comes from ancient Sanskrit, meaning deep wisdom and awakening.</p>
        </header>

        <div className="su-features">
          {FEATURES.map((f) => (
            <article className="su-feature" key={f.title}>
              {f.isNew && <span className="su-badge-new">New</span>}
              <div className="su-feature-icon"><Icon name={f.icon} size={19} /></div>
              <h3>{f.title}</h3>
              <p>{f.body}</p>
            </article>
          ))}
        </div>

        <section className="su-card">
          <div className="su-card-pad">
            <div className="su-card-head">
              <h2 className="su-card-title">Set up your Bodhi server</h2>
              <p className="su-card-sub">Give this instance a name so you can recognize it later.</p>
            </div>

            <div className="su-form-field">
              <label className="su-form-label" htmlFor="srv-name">
                Server Name <span className="su-req">Required</span>
              </label>
              <input id="srv-name" className="su-input" type="text"
                     placeholder="John Doe's Bodhi App Server"
                     value={name} onChange={(e) => setName(e.target.value)} />
              <p className="su-hint">Minimum 10 characters. This will identify your server instance.</p>
            </div>

            <div className="su-form-field">
              <label className="su-form-label" htmlFor="srv-desc">
                Description <span style={{ fontWeight: 400, color: 'hsl(var(--muted-foreground))', fontSize: 12 }}>Optional</span>
              </label>
              <textarea id="srv-desc" className="su-textarea"
                        placeholder="A short description of your Bodhi server instance…" />
              <p className="su-hint">Optional — helps you tell instances apart.</p>
            </div>
          </div>
        </section>

        <div className="su-nav">
          <a className="su-btn su-btn-primary is-block" href="setup-2-login.html">
            Set up Bodhi Server <Icon name="arrow-right" size={17} />
          </a>
        </div>
      </div>
    </SetupShell>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<Step1 />);
