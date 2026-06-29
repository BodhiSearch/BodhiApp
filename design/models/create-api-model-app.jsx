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
        { label: 'Bodhi', href: 'Chat.html' },
        { label: 'Models', href: 'Models-My-Models.html' },
        { label: 'New API Model', current: true },
      ]}
      contentClass="flush" mainScroll={false}
    >
      <div className="bf-scroll">
        <div className="bf-container">
          <ApiModelForm
            showCancel={true}
            title="Create New API Model"
            subtitle="Connect an external provider and choose which of its models route through this alias."
          />
        </div>
      </div>
    </AppShell>
    </>
  );
}

const camRoot = ReactDOM.createRoot(document.getElementById('root'));
camRoot.render(<CreateApiModelApp />);
