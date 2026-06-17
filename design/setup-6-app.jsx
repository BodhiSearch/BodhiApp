/* Setup · Step 6 — All Done */

const BRAND = {
  github: 'M12 .5C5.37.5 0 5.78 0 12.29c0 5.21 3.44 9.63 8.21 11.19.6.11.82-.26.82-.58 0-.29-.01-1.04-.02-2.05-3.34.71-4.04-1.59-4.04-1.59-.55-1.37-1.34-1.74-1.34-1.74-1.09-.74.08-.73.08-.73 1.21.08 1.84 1.23 1.84 1.23 1.07 1.8 2.81 1.28 3.5.98.11-.76.42-1.28.76-1.57-2.67-.3-5.47-1.31-5.47-5.84 0-1.29.47-2.34 1.23-3.17-.12-.3-.53-1.52.12-3.16 0 0 1.01-.32 3.3 1.21a11.6 11.6 0 0 1 3-.4c1.02 0 2.05.13 3 .4 2.29-1.53 3.3-1.21 3.3-1.21.65 1.64.24 2.86.12 3.16.77.83 1.23 1.88 1.23 3.17 0 4.54-2.81 5.53-5.49 5.83.43.37.81 1.1.81 2.22 0 1.6-.01 2.9-.01 3.29 0 .32.22.7.83.58A12.01 12.01 0 0 0 24 12.29C24 5.78 18.63.5 12 .5z',
  discord: 'M20.317 4.369a19.79 19.79 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.211.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.74 19.74 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 0 0 .031.057 19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028c.462-.63.874-1.295 1.226-1.994a.076.076 0 0 0-.041-.106 13.1 13.1 0 0 1-1.872-.892.077.077 0 0 1-.008-.128c.126-.094.252-.192.372-.291a.074.074 0 0 1 .077-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 0 1 .078.009c.12.099.246.198.373.292a.077.077 0 0 1-.006.127c-.598.349-1.22.645-1.873.892a.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.84 19.84 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.028zM8.02 15.331c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418z',
  x: 'M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z',
  youtube: 'M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z',
};

function BrandIcon({ name }) {
  return (
    <svg width="21" height="21" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <path d={BRAND[name]} />
    </svg>
  );
}

const COMMUNITY = [
  { icon: 'github',  title: 'Star on GitHub',  sub: 'Support the project, track updates, and contribute.' },
  { icon: 'discord', title: 'Join Discord',    sub: 'Connect with the community, get help, and share.' },
  { icon: 'x',       title: 'Follow on X',     sub: 'Stay updated with the latest news and announcements.' },
  { icon: 'youtube', title: 'Watch Tutorials', sub: 'Learn tips, tricks, and best practices.' },
];

function Step6() {
  return (
    <SetupShell current={5}>
      <div className="su-stagger">
        <header className="su-hero">
          <h1 className="su-hero-title">Setup Complete</h1>
          <p className="su-lead">Your Bodhi App is ready to use. Join our community to get the most out of it.</p>
        </header>

        <section className="su-card">
          <div className="su-card-pad">
            <div className="su-card-head" style={{ marginBottom: 18 }}>
              <h2 className="su-card-title" style={{ fontSize: 20 }}>Join our community</h2>
            </div>
            <div className="su-rows">
              {COMMUNITY.map((r) => (
                <a className="su-row" key={r.title} href="#" onClick={(e) => e.preventDefault()}>
                  <span className="su-row-icon"><BrandIcon name={r.icon} /></span>
                  <span className="su-row-body">
                    <h3 className="su-row-title">{r.title}</h3>
                    <p className="su-row-sub">{r.sub}</p>
                  </span>
                  <span className="su-row-arrow"><Icon name="arrow-right" size={18} /></span>
                </a>
              ))}
            </div>
          </div>
        </section>

        <section className="su-card">
          <div className="su-card-pad">
            <div className="su-card-head" style={{ marginBottom: 18 }}>
              <h2 className="su-card-title" style={{ fontSize: 20 }}>Quick resources</h2>
            </div>
            <div className="su-rows">
              <a className="su-row" href="#" onClick={(e) => e.preventDefault()} style={{ borderBottom: 'none' }}>
                <span className="su-row-icon"><Icon name="book-open" size={21} /></span>
                <span className="su-row-body">
                  <h3 className="su-row-title">Getting Started Guide</h3>
                  <p className="su-row-sub">Learn the basics and get up to speed quickly.</p>
                </span>
                <span className="su-row-arrow"><Icon name="arrow-right" size={18} /></span>
              </a>
            </div>
          </div>
        </section>

        <div className="su-nav">
          <a className="su-btn su-btn-primary is-block" href="Bodhi Chat.html">
            Start Using Bodhi App <Icon name="arrow-right" size={17} />
          </a>
        </div>
      </div>
    </SetupShell>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<Step6 />);
