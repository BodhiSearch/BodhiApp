/* Setup · Step 4 — API Models
   Embeds the shared <ApiModelForm/> (api-model-form.jsx) — the SAME form
   used on the Create API Model page — inside the setup wizard chrome. */

function Step4() {
  return (
    <SetupShell current={3}>
      <div className="su-rise">
        <div className="su-hero" style={{ marginBottom: 26 }}>
          <h2 className="su-card-title">Set up API Models</h2>
          <p className="su-card-sub">Configure cloud-based AI models to complement your local ones. You can connect a provider now, or skip and add models later.</p>
        </div>

        <ApiModelForm showCancel={false} />

        <div className="su-info" style={{ marginTop: 22 }}>
          <p>Don’t have an API key? You can skip this step and add API models later — they always complement your local models from the Models page.</p>
        </div>

        <div className="su-nav">
          <a className="su-btn su-btn-ghost" href="setup-3-local-models.html"><Icon name="arrow-left" size={17} /> Back</a>
          <span className="su-nav-spacer" />
          <a className="su-btn su-btn-primary" href="setup-5-extension.html">Continue <Icon name="arrow-right" size={17} /></a>
        </div>
      </div>
    </SetupShell>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<Step4 />);
