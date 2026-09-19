import { FaqCommand } from '@/components/faq-rail';
import type { FaqGroup } from '@/components/faq-rail';

/**
 * Remote Access help, shown in the shell's right rail.
 *
 * Ids are deep-link targets: the page passes them to `reveal(id)` from the
 * error or attention panel that the entry explains. Keep them stable.
 */

export type TunnelFaqId =
  | 'faq-install'
  | 'faq-update'
  | 'faq-path'
  | 'faq-notdetected'
  | 'faq-signin'
  | 'faq-zone'
  | 'faq-cert'
  | 'faq-createfail'
  | 'faq-kcsync'
  | 'faq-dns'
  | 'faq-dnsspread'
  | 'faq-exposure'
  | 'faq-local'
  | 'faq-restart'
  | 'faq-off'
  | 'faq-timeout'
  | 'faq-bots'
  | 'faq-unavailable';

const DOWNLOADS_URL = 'https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/';

export const TUNNEL_FAQ_GROUPS: FaqGroup[] = [
  {
    label: 'cloudflared',
    entries: [
      {
        id: 'faq-install',
        question: 'How do I install cloudflared?',
        answer: (
          <>
            <p>Install it with your usual package manager, then use Check again on step 1.</p>
            <FaqCommand os="macOS">brew install cloudflared</FaqCommand>
            <FaqCommand os="Windows">winget install --id Cloudflare.cloudflared</FaqCommand>
            <FaqCommand os="Linux">See Cloudflare&rsquo;s package repository instructions</FaqCommand>
            <p>
              Cloudflare&rsquo;s own commands, including Linux repository setup and direct downloads, are in{' '}
              <a href={DOWNLOADS_URL} target="_blank" rel="noreferrer">
                the cloudflared downloads documentation
              </a>
              .
            </p>
            <p>BodhiApp never downloads or installs it for you, so you stay in control of what runs on this machine.</p>
          </>
        ),
      },
      {
        id: 'faq-update',
        question: 'How do I update it?',
        answer: (
          <>
            <p>
              Remote access needs 2025.2.0 or newer. Older builds are missing the token-based start this feature relies
              on.
            </p>
            <FaqCommand os="macOS">brew upgrade cloudflared</FaqCommand>
            <FaqCommand os="Windows">winget upgrade --id Cloudflare.cloudflared</FaqCommand>
            <FaqCommand os="Linux">Update through the same package repository you installed from</FaqCommand>
            <p>Then use Check again. The detected version is shown on step 1.</p>
          </>
        ),
      },
      {
        id: 'faq-path',
        question: 'Can I point at a specific binary?',
        answer: (
          <>
            <p>
              Yes. Open Use a specific path on step 1 and give the full path to the executable. That is useful when you
              keep several versions, or when it lives somewhere off the PATH.
            </p>
            <p>
              The path is validated when you save it: if it is not an executable of a supported version, it is refused
              rather than stored.
            </p>
          </>
        ),
      },
      {
        id: 'faq-notdetected',
        question: 'It is installed, but not detected',
        answer: (
          <>
            <p>
              BodhiApp searches its own PATH, which is not always the PATH of your terminal &mdash; a desktop app
              launched from the dock inherits the system environment, not your shell profile.
            </p>
            <p>
              Confirm the location with <code>which cloudflared</code> (or <code>where cloudflared</code> on Windows),
              then enter that path under Use a specific path.
            </p>
          </>
        ),
      },
    ],
  },
  {
    label: 'Cloudflare sign-in',
    entries: [
      {
        id: 'faq-signin',
        question: 'How does BodhiApp know I am signed in?',
        answer: (
          <>
            <p>
              Signing in is something you do in a terminal. It opens a browser, you pick a domain, and Cloudflare writes
              a certificate to your machine.
            </p>
            <FaqCommand os="any">cloudflared tunnel login</FaqCommand>
            <p>
              BodhiApp reads that certificate to learn which domain you authorised. It never sees your Cloudflare
              password and cannot sign in on your behalf.
            </p>
          </>
        ),
      },
      {
        id: 'faq-zone',
        question: 'Can I use a different domain?',
        answer: (
          <>
            <p>
              The certificate authorises exactly one domain, which is why the address on step 3 is fixed. To use a
              different one, run the sign-in again and choose it, then use Check again.
            </p>
            <p>
              The part you choose is a single label &mdash; <code>bodhi</code>, not <code>bodhi.lab</code>.
              Cloudflare&rsquo;s certificate covers your domain and one level beneath it, so a multi-level name is
              reachable but fails its TLS handshake, which would break sign-in through the address.
            </p>
          </>
        ),
      },
      {
        id: 'faq-cert',
        question: 'The certificate is missing or unreadable',
        answer: (
          <>
            <p>
              Sign-in writes <code>cert.pem</code> to <code>~/.cloudflared/</code>. If it is missing, the sign-in did
              not finish; run it again and complete the browser step.
            </p>
            <p>
              If it exists but is refused, it is truncated or from an unrelated tool. Delete it and run the sign-in
              again. You can also point at a specific certificate under Use a specific path on step 2.
            </p>
          </>
        ),
      },
    ],
  },
  {
    label: 'Creating the tunnel',
    entries: [
      {
        id: 'faq-createfail',
        question: 'Creating the tunnel failed',
        answer: (
          <>
            <p>The usual causes, in the order worth checking:</p>
            <ul className="list-disc space-y-1 pl-4">
              <li>The certificate has been revoked in the Cloudflare dashboard &mdash; sign in again.</li>
              <li>The account lost permission to manage tunnels for that domain.</li>
              <li>No network route to Cloudflare, usually a proxy or firewall.</li>
            </ul>
            <p>The message under the step is what cloudflared reported, with anything secret removed.</p>
          </>
        ),
      },
      {
        id: 'faq-kcsync',
        question: 'What is the sign-in redirect step?',
        answer: (
          <>
            <p>
              Signing in through the public address only works if the authorization server expects that address back.
              BodhiApp registers <code>https://&lt;your-address&gt;/ui/auth/callback</code> for you when the tunnel
              starts.
            </p>
            <p>
              If it fails, the tunnel still carries traffic and API keys keep working &mdash; only browser sign-in
              through the address is affected.
            </p>
            <p>
              The registration is kept when you turn remote access off, so switching it back on is quick. Changing the
              address replaces the entry rather than adding another.
            </p>
          </>
        ),
      },
      {
        id: 'faq-dns',
        question: 'What does replacing the DNS record change?',
        answer: (
          <>
            <p>
              Only the one record for the address you chose. It is repointed at this instance&rsquo;s tunnel. No other
              record in the domain is read or modified.
            </p>
            <p>
              You are asked first because whatever the name pointed at before &mdash; another machine, another service
              &mdash; stops receiving traffic at that name immediately.
            </p>
          </>
        ),
      },
      {
        id: 'faq-dnsspread',
        question: 'The address does not resolve yet',
        answer: (
          <p>
            A new record usually works within a minute, but resolvers that already cached a previous answer can take
            longer. If the connector shows as connected, the tunnel is fine and you are waiting on DNS.
          </p>
        ),
      },
    ],
  },
  {
    label: 'Turning remote access on',
    entries: [
      {
        id: 'faq-exposure',
        question: 'What exactly becomes reachable?',
        answer: (
          <>
            <p>
              The whole instance &mdash; the same web interface and the same API surface you have locally, on one public
              address.
            </p>
            <p>
              Authentication does not change: everything that required signing in still does, and API endpoints still
              require a key. What changes is who can reach the door, not who can open it.
            </p>
            <p>It applies to every user of this instance, not just you.</p>
          </>
        ),
      },
      {
        id: 'faq-local',
        question: 'Does local access still work?',
        answer: (
          <p>
            Yes. The address on your machine and on your LAN keeps working exactly as before. The public address is an
            addition, not a replacement.
          </p>
        ),
      },
      {
        id: 'faq-restart',
        question: 'What happens when BodhiApp restarts?',
        answer: (
          <>
            <p>
              With Reconnect on start enabled, the connector is started again with the saved address. With it off,
              remote access stays down until you turn it on.
            </p>
            <p>
              A failed reconnect is reported and left alone &mdash; it is a single attempt, never a retry loop, so a
              misconfiguration cannot turn into repeated calls to Cloudflare.
            </p>
          </>
        ),
      },
      {
        id: 'faq-off',
        question: 'What does turning it off do?',
        answer: (
          <>
            <p>
              The connector stops and the public address stops serving immediately. Local and LAN access are unaffected.
            </p>
            <p>
              Two things are kept on purpose so switching back on is quick: the DNS record, and the sign-in redirect
              registration. While it is off, visiting the address shows a Cloudflare error saying the tunnel is not
              running &mdash; that is expected, not a fault.
            </p>
          </>
        ),
      },
    ],
  },
  {
    label: 'Traffic through Cloudflare',
    entries: [
      {
        id: 'faq-timeout',
        question: 'Long replies get cut off',
        answer: (
          <p>
            Cloudflare applies its own limits to a proxied connection, and a long non-streaming generation can exceed
            them. Streaming responses are the reliable way to handle long generations through the tunnel, since data
            keeps flowing rather than waiting on one slow reply.
          </p>
        ),
      },
      {
        id: 'faq-bots',
        question: 'Requests are challenged or blocked',
        answer: (
          <p>
            The address sits behind your Cloudflare account, so its security rules apply. Bot protection or a WAF rule
            can challenge API clients, which cannot answer a browser challenge. If scripted clients are being blocked,
            adjust the rules for this hostname in the Cloudflare dashboard.
          </p>
        ),
      },
    ],
  },
  {
    label: 'Deployment',
    entries: [
      {
        id: 'faq-unavailable',
        question: 'Why is remote access unavailable here?',
        answer: (
          <>
            <p>
              It is a single-instance feature and is off unless this deployment enables it. Each instance claims its own
              tunnel, so several instances sharing one address would hand visitors to whichever answered first.
            </p>
            <p>
              For a deployment that is already behind a proxy or load balancer, expose it there instead &mdash; that
              layer is the right place to own a public address.
            </p>
          </>
        ),
      },
    ],
  },
];
