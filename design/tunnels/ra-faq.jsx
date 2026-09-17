/* ═══════════════════════════════════════════════════
   REMOTE ACCESS · FAQ / debugging reference
   tunnels/ra-faq.jsx  (load after ra-parts.jsx)
   One shared, always-present list at the bottom of every
   remote-access page. Every entry is collapsed by default;
   in-page buttons expand and scroll to the relevant one.
═══════════════════════════════════════════════════ */
const RA_FAQ_GROUPS = [
{
  group: 'cloudflared',
  items: [
  {
    id: 'faq-install',
    q: 'How do I install cloudflared?',
    a:
    <>
        <p>Install it with your usual package manager, then use <strong>Check again</strong> on step 1.</p>
        <div className="ra-cmd"><span className="ra-cmd-os">macOS</span>brew install cloudflared</div>
        <div className="ra-cmd"><span className="ra-cmd-os">Windows</span>winget install --id Cloudflare.cloudflared</div>
        <div className="ra-cmd"><span className="ra-cmd-os">Linux</span>See Cloudflare’s package repository instructions</div>
        <p>
          Cloudflare’s own commands, including Linux repository setup and direct downloads, are in{' '}
          <a href="https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/" target="_blank" rel="noreferrer">
            the cloudflared downloads documentation <RA_Ic name="external-link" size={11} />
          </a>.
        </p>
        <p>BodhiApp never downloads or installs software — you install cloudflared, BodhiApp detects it.</p>
      </>

  },
  {
    id: 'faq-update',
    q: 'My version is unsupported — how do I update?',
    a:
    <>
        <p>Nothing is missing; the existing install needs to move forward. Cloudflare supports <span className="ra-mono">2025.2.0</span> and newer.</p>
        <div className="ra-cmd"><span className="ra-cmd-os">macOS</span>brew upgrade cloudflared</div>
        <div className="ra-cmd"><span className="ra-cmd-os">Windows</span>winget upgrade --id Cloudflare.cloudflared</div>
        <div className="ra-cmd"><span className="ra-cmd-os">Linux</span>Update through the same package repository you installed from</div>
        <p>Older versions can fail during tunnel provisioning in ways that are hard to read, which is why they are refused up front.</p>
      </>

  },
  {
    id: 'faq-path',
    q: 'Can I point BodhiApp at a specific binary?',
    a:
    <>
        <p>
          Yes. Enter the full path to the executable in <strong>Binary location</strong> on step 1 and press
          <strong> Check again</strong>. BodhiApp runs a version check on that file; if it passes, the path is
          stored and used from then on. If it fails, the path is not stored and the step says why.
        </p>
        <p>This works whether or not auto-detection found something — an entered path always wins over a detected one, and clearing the field returns to auto-detection.</p>
      </>

  },
  {
    id: 'faq-notdetected',
    q: 'It works in my terminal but isn’t detected here',
    a:
    <>
        <p>
          Apps launched from the Dock, Finder or a desktop shortcut inherit a minimal <span className="ra-mono">PATH</span>,
          not your shell’s. BodhiApp also probes the standard install locations, but a non-standard one it
          cannot guess stays invisible.
        </p>
        <p>Two fixes, either is fine: enter the full path in <strong>Binary location</strong>, or relaunch BodhiApp from a terminal.</p>
      </>

  }]

},
{
  group: 'Cloudflare sign-in',
  items: [
  {
    id: 'faq-signin',
    q: 'How do we detect your Cloudflare login?',
    a:
    <>
        <p>
          Signing in with <span className="ra-mono">cloudflared tunnel login</span> writes a certificate to{' '}
          <span className="ra-mono">~/.cloudflared/cert.pem</span>. BodhiApp checks that this file exists at
          the default location and is valid — that is how it knows your login and tunnel setup are complete.
          It never runs the login itself and never sees your Cloudflare password or an API token.
        </p>
        <p>
          The certificate also records which of your domains you picked during the login, so BodhiApp reads
          the domain from it and only offers subdomains under that domain.
        </p>
        <p>
          If your certificate lives elsewhere, enter its full path in <strong>Certificate location</strong>{' '}
          and check again. Cloudflare’s own walkthrough is in{' '}
          <a href="https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/get-started/create-remote-tunnel/" target="_blank" rel="noreferrer">
            the tunnel setup documentation <RA_Ic name="external-link" size={11} />
          </a>.
        </p>
      </>

  },
  {
    id: 'faq-zone',
    q: 'The domain is wrong — can I change it?',
    a:
    <>
        <p>
          Only by signing in again. One login covers one domain, and the tunnel can only be hosted there.
          Run <span className="ra-mono">cloudflared tunnel login</span> again and pick the domain you want;
          the new certificate replaces the old one.
        </p>
        <p>You choose the subdomain here afterwards, one level deep — <span className="ra-mono">bodhi</span>, not <span className="ra-mono">bodhi.internal</span>.</p>
      </>

  },
  {
    id: 'faq-cert',
    q: 'The certificate can’t be found or used',
    a:
    <>
        <p>
          Signing in writes <span className="ra-mono">cert.pem</span> into <span className="ra-mono">~/.cloudflared/</span>,
          and that file is what BodhiApp checks. If it isn’t there, either the login didn’t finish in the
          browser, or it ran as a different user with a different home directory.
        </p>
        <p>
          If your certificate lives somewhere else — a custom <span className="ra-mono">--origincert</span> path,
          or a service account’s home — enter its full path in <strong>Certificate location</strong> and check
          again. A path that validates is stored and used from then on.
        </p>
        <p>
          A file that exists but fails validation is usually truncated or unreadable by the account BodhiApp
          runs as. Running <span className="ra-mono">cloudflared tunnel login</span> again replaces it.
        </p>
      </>

  }]

},
{
  group: 'Creating the tunnel',
  items: [
  {
    id: 'faq-createfail',
    q: 'Why can tunnel creation fail?',
    a:
    <>
        <p>Almost always one of four things, in this order of likelihood:</p>
        <p>
          The domain you picked during login isn’t on the Cloudflare account you’re signed in as; your
          Cloudflare account lacks permission to create tunnels or DNS records on it; the login has
          expired and needs running again; or this machine can’t reach Cloudflare at all.
        </p>
        <p>Retrying is safe. If it keeps failing, run <span className="ra-mono">cloudflared tunnel login</span> again and watch which domain you select.</p>
      </>

  },
  {
    id: 'faq-kcsync',
    q: 'Why does the sign-in redirect URL need syncing?',
    a:
    <>
        <p>
          Sign-in runs through Keycloak, and Keycloak only redirects back to addresses it has been told
          about. When the tunnel comes up, BodhiApp registers <span className="ra-mono">https://&lt;your-address&gt;/ui/auth/callback</span>
          on this instance’s Keycloak client so sign-in works over the tunnel too. Turning remote access off keeps it, so
          switching back on is quick; changing the address replaces the entry rather than adding another.
        </p>
        <p>
          If the sync fails, the tunnel still carries traffic — the address loads, but sign-in through it
          is refused with a redirect-URI error. Local and LAN sign-in are unaffected.
        </p>
        <p>
          Failures are usually one of two things: this machine can’t reach the Keycloak server right now,
          or the credentials this instance uses aren’t allowed to edit its own client. The first clears
          up on retry; the second needs a Keycloak admin to grant the instance permission on its client.
        </p>
      </>

  },
  {
    id: 'faq-dns',
    q: 'Something already answers at that address',
    a:
    <>
        <p>
          A DNS record for that subdomain already exists on your domain and points somewhere else.
          Replacing it repoints that one address at this BodhiApp instance; nothing else on the domain
          changes.
        </p>
        <p>If that address is in use by something you still need, pick a different subdomain instead. The old record can be restored from your Cloudflare dashboard.</p>
      </>

  },
  {
    id: 'faq-dnsspread',
    q: 'It’s connected but the address doesn’t work yet',
    a: <p>A new address can take a few minutes to become reachable everywhere while DNS propagates. Nothing is wrong — wait a couple of minutes and try again, ideally in a new browser tab.</p>
  }]

},
{
  group: 'Turning remote access on',
  items: [
  {
    id: 'faq-exposure',
    q: 'What becomes reachable from the internet?',
    a:
    <>
        <p>
          The whole instance — the web UI and every API surface (<code>/v1</code>, <code>/anthropic/v1</code>,
          <code> /v1beta</code>) — answers at your hostname. Sign-in is still required past the login page,
          but the login page and every endpoint become publicly reachable.
        </p>
        <p>There is no IP allowlist, no per-path control and no rate limiting here. It applies to everyone on this instance, not just you.</p>
      </>

  },
  {
    id: 'faq-local',
    q: 'Does this change local or LAN access?',
    a: <p>No. The tunnel is additive. <span className="ra-mono">http://localhost:1135</span> and your LAN address keep working exactly as before, whether the tunnel is up, down or off.</p>
  },
  {
    id: 'faq-restart',
    q: 'What happens after BodhiApp restarts?',
    a: <p>With <strong>Reconnect automatically</strong> on, remote access comes back by itself at the same address. With it off, BodhiApp starts with remote access off and waits for you to turn it on. Either way the address and tunnel are kept.</p>
  },
  {
    id: 'faq-off',
    q: 'What does turning remote access off do?',
    a:
    <>
        <p>It stops the tunnel, so the public address stops answering. Your address, the Cloudflare tunnel and its DNS record all stay in place, so turning it back on takes seconds and nothing needs setting up again.</p>
        <p>Nothing is deleted on Cloudflare. Local and LAN access are unaffected.</p>
      </>

  }]

},
{
  group: 'Traffic through Cloudflare',
  items: [
  {
    id: 'faq-timeout',
    q: 'Long non-streaming requests get cut off',
    a:
    <>
        <p>
          Cloudflare’s edge closes a connection that goes quiet for roughly 100–125 seconds, so a single
          non-streaming response that takes longer than that to begin can be dropped.
        </p>
        <p>Streaming requests send data continuously and are unaffected. There is no adjustment for this below a Cloudflare Enterprise plan.</p>
      </>

  },
  {
    id: 'faq-bots',
    q: 'SDK calls fail but the browser UI works',
    a: <p>Bot Fight Mode on your Cloudflare zone can silently block traffic that looks automated, which is what most SDK clients look like. On a free Cloudflare plan there is no way to exempt them. Check this first, in your own Cloudflare dashboard.</p>
  }]

},
{
  group: 'Deployment',
  items: [
  {
    id: 'faq-unavailable',
    q: 'Why is remote access unavailable on some deployments?',
    a:
    <>
        <p>BodhiApp runs the tunnel as a child process it manages. In a container it cannot do that, so the capability is inactive rather than broken — there is nothing to retry.</p>
        <p>This is not the same as an admin turning remote access off, which is reversible and keeps the hostname. To use a tunnel, run BodhiApp as a desktop app on a machine you control.</p>
      </>

  }]

}];


function raScrollToFaq(id) {
  // Measure after React has committed the expansion, then set scrollTop
  // directly — smooth scrollTo is ignored in some embedded engines.
  setTimeout(() => {
    const el = document.getElementById(id);
    if (!el) return;
    const sc = el.closest('.bf-scroll, .shell-body');
    if (!sc) return;
    const top = el.getBoundingClientRect().top - sc.getBoundingClientRect().top + sc.scrollTop - 12;
    const before = sc.scrollTop;
    try { sc.scrollTo({ top, behavior: 'smooth' }); } catch (e) {}
    setTimeout(() => { if (sc.scrollTop === before) sc.scrollTop = top; }, 60);
  }, 0);
}

/* Hook: owns which entries are open, plus the "open the relevant one" action.
   The list lives in the shell's right rail, which may be collapsed — so reveal
   only records the target; <RAFaqRail> opens the rail and does the scrolling. */
function useRAFaq() {
  const [open, setOpen] = React.useState([]);
  const [flash, setFlash] = React.useState(null);
  const [req, setReq] = React.useState(null);   // { id, seq }
  const toggle = (id) => setOpen((o) => o.includes(id) ? o.filter((x) => x !== id) : [...o, id]);
  const reveal = (id) => {
    setOpen((o) => o.includes(id) ? o : [...o, id]);
    setFlash(id);
    setReq((r) => ({ id, seq: (r ? r.seq : 0) + 1 }));
    setTimeout(() => setFlash(null), 1400);
  };
  return { faqProps: { open, flash, onToggle: toggle, req }, reveal };
}

/* Rail header — the panel's own title row, with a close affordance. */
function RAFaqRailHeader() {
  const { collapseRail, closeRail, isMobile } = useShell();
  return (
    <div className="ra-faq-railhead">
      <span className="ra-faq-railhead-t"><RA_Ic name="circle-help" size={14} /> Help &amp; debugging</span>
      <button className="shell-icon-btn" title="Close"
      onClick={() => isMobile ? closeRail() : collapseRail()}>
        <RA_Ic name="x" size={15} />
      </button>
    </div>);

}

/* The FAQ as the right rail: same list, rail chrome, and it opens itself
   when an in-page "why this happens" button asks for an entry. */
function RAFaqRail({ req, ...faq }) {
  const { openRail } = useShell();
  const seq = req ? req.seq : 0;
  React.useEffect(() => {
    if (!seq) return;
    openRail();
    raScrollToFaq(req.id);
  }, [seq]);
  return (
    <div className="ra-faq-rail">
      <p className="ra-faq-rail-sub">Everything about getting this working, in one place.</p>
      <RAFaq {...faq} inRail />
    </div>);

}

function RAFaq({ open = [], flash, onToggle, inRail }) {
  return (
    <section className={'ra-faq' + (inRail ? ' in-rail' : '')} aria-label="Help and debugging">
      {!inRail &&
      <div className="ra-faq-head">
        <span className="ra-faq-title">Help &amp; debugging</span>
        <span className="ra-faq-sub">Everything about getting this working, in one place.</span>
      </div>}
      {RA_FAQ_GROUPS.map((g) =>
      <div className="ra-faq-group" key={g.group}>
          <div className="ra-faq-glabel">{g.group}</div>
          {g.items.map((it) => {
          const isOpen = open.includes(it.id);
          return (
            <div className={`ra-faq-item${isOpen ? ' is-open' : ''}${flash === it.id ? ' is-flash' : ''}`} id={it.id} key={it.id}>
                <button className="ra-faq-q" onClick={() => onToggle(it.id)} aria-expanded={isOpen}>
                  <span>{it.q}</span>
                  <span className="ra-faq-chev"><RA_Ic name="chevron-down" size={15} /></span>
                </button>
                {isOpen && <div className="ra-faq-a">{it.a}</div>}
              </div>);

        })}
        </div>
      )}
    </section>);

}

Object.assign(window, { RA_FAQ_GROUPS, raScrollToFaq, useRAFaq, RAFaq, RAFaqRail, RAFaqRailHeader });
