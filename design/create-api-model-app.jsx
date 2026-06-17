/* ═══════════════════════════════════════════
   CREATE NEW API MODEL — page wrapper
   The form itself lives in api-model-form.jsx (<ApiModelForm/>),
   shared with the setup wizard so both stay identical.
   Light / dark theme via Tweaks panel.
═══════════════════════════════════════════ */

function CreateApiModelApp() {
  return (
    <>
    <AppShell
      section="models" subPage="new-api-model" resizeKey="createmodel"
      breadcrumb={[
        { label: 'Bodhi', href: 'Bodhi Chat.html' },
        { label: 'Models', href: 'Bodhi Models.html' },
        { label: 'New API Model', current: true },
      ]}
      contentClass="flush" mainScroll={false}
    >
      <div className="bf-scroll">
        <div className="bf-container">
          <div className="bf-page-head">
            <h1 className="bf-page-title">Create New API Model</h1>
            <p className="bf-page-sub">Configure a new external AI API model. Connect a provider, choose routing, and pick which models forward through this alias.</p>
          </div>

          <ApiModelForm showCancel={true} />
        </div>
      </div>
    </AppShell>
    </>
  );
}

const camRoot = ReactDOM.createRoot(document.getElementById('root'));
camRoot.render(<CreateApiModelApp />);
