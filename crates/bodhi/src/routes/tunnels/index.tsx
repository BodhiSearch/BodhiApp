import { useEffect, useState } from 'react';

import type { TunnelStatus } from '@bodhiapp/ts-client';
import { createFileRoute } from '@tanstack/react-router';
import { Ban, CloudCog, Copy, ExternalLink, Power, RefreshCw, Users } from 'lucide-react';

import AppInitializer from '@/components/AppInitializer';
import { FaqLink, FaqRail, FaqRailHeader, FaqRevealProvider, useFaqRail } from '@/components/faq-rail';
import { useShellChrome } from '@/components/shell';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  useDisableTunnel,
  useEnableTunnel,
  useSyncTunnelAuthorization,
  useTunnelSetup,
  useTunnelStatus,
  useUpdateTunnelPreferences,
} from '@/hooks/tunnels';
import { useToastMessages } from '@/hooks/useToastMessages';
import {
  CommandLine,
  EditAction,
  Note,
  PathField,
  Progress,
  StatusPill,
  Step,
} from '@/routes/tunnels/-components/RemoteAccessParts';
import { TUNNEL_FAQ_GROUPS } from '@/routes/tunnels/-shared/faqContent';
import {
  addressPill,
  addressTone,
  deriveRemoteAccessState,
  isConnected,
  isLocked,
  isSetupDone,
  overallLabel,
  toneOf,
  validateSubdomain,
  type RemoteAccessState,
} from '@/routes/tunnels/-shared/remoteAccessState';

export const Route = createFileRoute('/tunnels/')({
  staticData: { section: 'settings', subPage: 'tunnels' },
  component: TunnelsPage,
});

const BREADCRUMB = [
  { label: 'Bodhi' },
  { label: 'Settings', href: '/settings/' },
  { label: 'Remote Access', current: true },
];

const CERT_DEFAULT = '~/.cloudflared/cert.pem';

function useCopy() {
  const { showError, showSuccess } = useToastMessages();
  return async (value: string, label: string) => {
    try {
      await navigator.clipboard.writeText(value);
      showSuccess('Copied', `${label} copied to your clipboard.`);
    } catch {
      showError('Copy failed', 'Your browser did not allow clipboard access.');
    }
  };
}

function BinaryStep({
  state,
  status,
  value,
  onChange,
  onCheck,
  checking,
  onLocked,
}: {
  state: RemoteAccessState;
  status: TunnelStatus;
  value: string;
  onChange: (value: string) => void;
  onCheck: () => void;
  checking: boolean;
  onLocked: () => void;
}) {
  const detected = status.binary.path;
  const locked = isConnected(state);
  const field = {
    id: 'tunnel-binary-path',
    label: 'Binary location',
    placeholder: '/usr/local/bin/cloudflared',
    foundHint: (
      <>
        Found at <span className="font-mono">{detected}</span>.
      </>
    ),
    missingHint: 'Not found. Enter the full path to use your own.',
  };
  const check = (
    <Button
      type="button"
      variant="outline"
      size="sm"
      onClick={onCheck}
      disabled={checking}
      data-testid="tunnel-binary-check"
    >
      <RefreshCw className={checking ? 'animate-spin motion-reduce:animate-none' : ''} aria-hidden="true" /> Check again
    </Button>
  );

  if (!['binary-missing', 'binary-old', 'path-invalid'].includes(state)) {
    return (
      <Step
        n={1}
        tone="ok"
        title="cloudflared"
        pill="Ready"
        collapsible
        defaultFolded={isSetupDone(state)}
        lockFolded={locked}
        summary={detected ?? undefined}
        action={<EditAction locked={locked} onLocked={onLocked} />}
        status={
          <>
            Version <span className="font-mono">{status.binary.version}</span> at{' '}
            <span className="font-mono">{detected}</span>.
          </>
        }
      >
        <PathField {...field} value={value} onChange={onChange} detected={detected} disabled={locked} />
        <div className="flex flex-wrap gap-2">{check}</div>
      </Step>
    );
  }

  const copy = {
    'binary-missing': {
      tone: 'attn' as const,
      pill: 'Not found',
      status: 'Remote access needs the cloudflared program, and it isn’t here yet.',
      detected: null,
    },
    'binary-old': {
      tone: 'attn' as const,
      pill: 'Too old',
      status: (
        <>
          Version <span className="font-mono">{status.binary.version}</span> is older than Cloudflare supports. Update
          to <span className="font-mono">{status.binary.minimum_version}</span> or newer.
        </>
      ),
      detected,
    },
    'path-invalid': {
      tone: 'fail' as const,
      pill: 'Path rejected',
      status: 'The path you gave couldn’t be used, so it wasn’t saved.',
      detected: null,
    },
  }[state as 'binary-missing' | 'binary-old' | 'path-invalid'];

  return (
    <Step n={1} tone={copy.tone} title="cloudflared" pill={copy.pill} status={copy.status}>
      <PathField {...field} value={value} onChange={onChange} detected={copy.detected} error={status.binary.error} />
      <div className="flex flex-wrap items-center gap-2">
        {check}
        {state === 'binary-missing' && (
          <>
            <FaqLink id="faq-install">How to install it</FaqLink>
            <FaqLink id="faq-notdetected">It’s installed but not detected</FaqLink>
          </>
        )}
        {state === 'binary-old' && <FaqLink id="faq-update">How to update it</FaqLink>}
        {state === 'path-invalid' && <FaqLink id="faq-path">Pointing at a binary manually</FaqLink>}
      </div>
    </Step>
  );
}

function LoginStep({
  state,
  status,
  value,
  onChange,
  onCheck,
  checking,
  onLocked,
}: {
  state: RemoteAccessState;
  status: TunnelStatus;
  value: string;
  onChange: (value: string) => void;
  onCheck: () => void;
  checking: boolean;
  onLocked: () => void;
}) {
  const copyText = useCopy();
  const detected = status.login.cert_path;
  const locked = isConnected(state);

  if (['binary-missing', 'binary-old', 'path-invalid'].includes(state)) {
    return (
      <Step n={2} tone="idle" title="Cloudflare sign-in" pill="Waiting" status="Checked once cloudflared is ready." />
    );
  }

  const field = {
    id: 'tunnel-cert-path',
    label: 'Certificate location',
    placeholder: CERT_DEFAULT,
    foundHint: (
      <>
        Found at <span className="font-mono">{detected}</span>.
      </>
    ),
    missingHint: (
      <>
        Nothing at <span className="font-mono">{CERT_DEFAULT}</span>. Enter the full path to use your own.
      </>
    ),
  };
  const check = (
    <Button
      type="button"
      variant="outline"
      size="sm"
      onClick={onCheck}
      disabled={checking}
      data-testid="tunnel-cert-check"
    >
      <RefreshCw className={checking ? 'animate-spin motion-reduce:animate-none' : ''} aria-hidden="true" /> Check again
    </Button>
  );

  if (state === 'login-needed' || state === 'login-failed') {
    const failed = state === 'login-failed';
    return (
      <Step
        n={2}
        tone={failed ? 'fail' : 'attn'}
        title="Cloudflare sign-in"
        pill={failed ? 'Not confirmed' : 'Signed out'}
        status={
          failed
            ? 'That certificate couldn’t be used, so it wasn’t saved.'
            : 'The domain you pick while signing in is the one your address sits on.'
        }
      >
        <PathField
          {...field}
          value={value}
          onChange={onChange}
          detected={failed ? null : detected}
          error={status.login.error}
        />
        <p className="text-sm text-muted-foreground">
          To sign in and generate the certificate, run this in a terminal:
        </p>
        <CommandLine onCopy={(command) => copyText(command, 'Command')}>cloudflared tunnel login</CommandLine>
        <div className="flex flex-wrap items-center gap-2">
          {check}
          <FaqLink id={failed ? 'faq-cert' : 'faq-signin'}>
            {failed ? 'Problems with the certificate' : 'How we detect Cloudflare sign-in'}
          </FaqLink>
        </div>
      </Step>
    );
  }

  return (
    <Step
      n={2}
      tone="ok"
      title="Cloudflare sign-in"
      pill="Ready"
      collapsible
      defaultFolded={isSetupDone(state)}
      lockFolded={locked}
      summary={status.login.zone ?? undefined}
      action={<EditAction locked={locked} onLocked={onLocked} />}
      status={
        <>
          Signed in, on <strong>{status.login.zone}</strong>.
        </>
      }
    >
      <PathField {...field} value={value} onChange={onChange} detected={detected} disabled={locked} />
      <div className="flex flex-wrap gap-2">{check}</div>
    </Step>
  );
}

function KeycloakSync({
  state,
  status,
  onRetry,
  retrying,
}: {
  state: RemoteAccessState;
  status: TunnelStatus;
  onRetry: () => void;
  retrying: boolean;
}) {
  if (state === 'kc-syncing') {
    return (
      <Note tone="checking" title="Syncing the sign-in redirect URL with Keycloak" testId="tunnel-kc-note">
        <p>
          Registering <span className="font-mono text-xs">{status.oauth_redirect_uri}</span> so sign-in works through
          the tunnel.
        </p>
      </Note>
    );
  }
  if (state === 'live') {
    return (
      <Note tone="ok" title="Sign-in redirect URL synced with Keycloak" testId="tunnel-kc-note">
        <p>
          You can sign in at <strong>{status.public_url}</strong>.
        </p>
      </Note>
    );
  }
  if (state !== 'kc-fail-net' && state !== 'kc-fail-auth') return null;
  const unreachable = state === 'kc-fail-net';
  return (
    <Note
      tone="warn"
      title={
        unreachable
          ? 'Couldn’t reach Keycloak to update the sign-in redirect'
          : 'Keycloak refused the redirect URL change'
      }
      testId="tunnel-kc-note"
    >
      <p>
        {unreachable
          ? 'The tunnel is up, but signing in through the address will be refused until the redirect URL is registered.'
          : 'This instance isn’t allowed to edit its own Keycloak client, so the redirect URL wasn’t added. Signing in through the address will be refused until it is.'}{' '}
        API-key requests keep working.
      </p>
      {status.auth_sync.error && <p className="break-words font-mono text-xs">{status.auth_sync.error}</p>}
      <div className="flex flex-wrap items-center gap-2">
        <Button type="button" size="sm" onClick={onRetry} disabled={retrying} data-testid="tunnel-retry-sync">
          <RefreshCw className={retrying ? 'animate-spin motion-reduce:animate-none' : ''} aria-hidden="true" /> Retry
          sync
        </Button>
        <FaqLink id="faq-kcsync">Why this is needed</FaqLink>
      </div>
    </Note>
  );
}

function AddressStep({
  state,
  status,
  subdomain,
  setSubdomain,
  onCreate,
  onDisable,
  onRetrySync,
  onAutoReconnect,
  pending,
  createError,
}: {
  state: RemoteAccessState;
  status: TunnelStatus;
  subdomain: string;
  setSubdomain: (value: string) => void;
  onCreate: (replaceDns: boolean) => void;
  onDisable: () => void;
  onRetrySync: () => void;
  onAutoReconnect: (value: boolean) => void;
  pending: { enable: boolean; disable: boolean; sync: boolean; preferences: boolean };
  createError: string | null;
}) {
  const copyText = useCopy();
  const [confirming, setConfirming] = useState(false);
  const [fieldError, setFieldError] = useState<string | null>(null);

  useEffect(() => {
    setConfirming(false);
  }, [state]);

  if (!isSetupDone(state)) {
    return (
      <Step n={3} tone="idle" title="Remote access" pill="Waiting" status="Available once the two checks above pass." />
    );
  }

  const zone = status.login.zone ?? '';
  const on = isConnected(state);
  const busy = state === 'creating';
  const locked = isLocked(state);
  const host = status.hostname ?? `${subdomain.trim() || 'bodhi'}.${zone}`;

  const submit = () => {
    const error = validateSubdomain(subdomain);
    setFieldError(error);
    if (!error) setConfirming(true);
  };

  return (
    <Step
      n={3}
      tone={addressTone(state)}
      title="Remote access"
      pill={addressPill(state)}
      spin={busy || state === 'kc-syncing'}
      status={
        on ? undefined : (
          <>
            Pick the address this instance answers on, under <span className="font-mono">{zone}</span>.
          </>
        )
      }
    >
      <div className="space-y-2">
        <label className="text-sm font-medium" htmlFor="tunnel-subdomain">
          Subdomain
        </label>
        <div className="flex flex-wrap items-center gap-2">
          <Input
            id="tunnel-subdomain"
            data-testid="tunnel-subdomain"
            autoComplete="off"
            spellCheck={false}
            className="min-w-0 flex-1 font-mono text-sm"
            placeholder="bodhi"
            value={subdomain}
            disabled={locked}
            onChange={(event) => {
              setSubdomain(event.target.value);
              setFieldError(null);
            }}
          />
          <span className="font-mono text-sm text-muted-foreground">.{zone}</span>
          {on && status.public_url && (
            <div className="flex gap-2">
              <Button asChild variant="outline" size="sm" data-testid="tunnel-open">
                <a href={status.public_url} target="_blank" rel="noreferrer" aria-label={`Open ${status.public_url}`}>
                  <ExternalLink aria-hidden="true" />
                </a>
              </Button>
              <Button
                type="button"
                variant="outline"
                size="sm"
                data-testid="tunnel-copy"
                aria-label="Copy address"
                onClick={() => copyText(status.public_url ?? '', 'Public URL')}
              >
                <Copy aria-hidden="true" />
              </Button>
            </div>
          )}
        </div>
        {fieldError && (
          <p data-testid="tunnel-subdomain-error" className="text-sm text-destructive">
            {fieldError}
          </p>
        )}
      </div>

      <label className="flex cursor-pointer items-center gap-2 text-sm">
        <input
          type="checkbox"
          data-testid="tunnel-auto-reconnect"
          checked={status.auto_reconnect}
          onChange={(event) => onAutoReconnect(event.target.checked)}
          disabled={pending.preferences}
        />
        Connect automatically after BodhiApp restarts
      </label>

      {busy && <Progress at={1} />}

      <KeycloakSync state={state} status={status} onRetry={onRetrySync} retrying={pending.sync} />

      {state === 'dns-conflict' && (
        <Note tone="warn" title={`Something already answers at ${host}`} testId="tunnel-dns-conflict">
          <p>Replace that DNS record, or pick a different subdomain.</p>
          <div className="flex flex-wrap items-center gap-2">
            <Button
              type="button"
              size="sm"
              onClick={() => onCreate(true)}
              disabled={pending.enable}
              data-testid="tunnel-replace-dns"
            >
              Replace it and continue
            </Button>
            <FaqLink id="faq-dns">What gets replaced</FaqLink>
          </div>
        </Note>
      )}

      {state === 'create-failed' && (
        <Note tone="fail" title="Couldn’t set up the tunnel" testId="tunnel-create-failed">
          <p>{status.error_message ?? 'Cloudflare didn’t accept the request.'}</p>
          <div className="flex flex-wrap items-center gap-2">
            <Button
              type="button"
              size="sm"
              onClick={() => onCreate(false)}
              disabled={pending.enable}
              data-testid="tunnel-retry-create"
            >
              <RefreshCw aria-hidden="true" /> Try again
            </Button>
            <FaqLink id="faq-createfail">Common causes</FaqLink>
          </div>
        </Note>
      )}

      {createError && state !== 'dns-conflict' && state !== 'create-failed' && (
        <Note tone="fail" title="Couldn’t set up the tunnel" testId="tunnel-create-error">
          <p>{createError}</p>
        </Note>
      )}

      {on && (
        <div className="flex flex-wrap items-center gap-2">
          <Button
            type="button"
            variant="outline"
            onClick={onDisable}
            disabled={pending.disable}
            data-testid="tunnel-turn-off"
          >
            <Power aria-hidden="true" /> {pending.disable ? 'Turning off…' : 'Turn off'}
          </Button>
        </div>
      )}

      {state === 'off' && (
        <div className="flex flex-wrap items-center gap-2">
          <Button type="button" onClick={() => onCreate(false)} disabled={pending.enable} data-testid="tunnel-turn-on">
            <Power aria-hidden="true" /> Turn on
          </Button>
          <span className="inline-flex items-center gap-1 text-xs text-muted-foreground">
            <Users className="h-3 w-3" aria-hidden="true" /> Applies to everyone on this instance
          </span>
        </div>
      )}

      {state === 'address' &&
        (confirming ? (
          <div className="space-y-3" data-testid="tunnel-confirm">
            <Note tone="warn" title="This opens the whole instance to the internet">
              <p>
                The web UI and every API surface become reachable by anyone at <strong>https://{host}</strong>. Sign-in
                is still required past the login page. Instance-wide, not just for you.
              </p>
              <FaqLink id="faq-exposure">What exactly is exposed</FaqLink>
            </Note>
            <div className="flex flex-wrap gap-2">
              <Button
                type="button"
                onClick={() => onCreate(false)}
                disabled={pending.enable}
                data-testid="tunnel-confirm-create"
              >
                <Power aria-hidden="true" /> Yes, create it
              </Button>
              <Button
                type="button"
                variant="ghost"
                onClick={() => setConfirming(false)}
                data-testid="tunnel-cancel-create"
              >
                Cancel
              </Button>
            </div>
          </div>
        ) : (
          <div className="flex flex-wrap items-center gap-2">
            <Button type="button" onClick={submit} disabled={pending.enable} data-testid="tunnel-create">
              <CloudCog aria-hidden="true" /> Create tunnel
            </Button>
            <span className="inline-flex items-center gap-1 text-xs text-muted-foreground">
              <Users className="h-3 w-3" aria-hidden="true" /> Applies to everyone on this instance
            </span>
          </div>
        ))}
    </Step>
  );
}

function Unavailable({ reason }: { reason?: string | null }) {
  return (
    <section className="rounded-lg border p-8 text-center" data-testid="tunnel-unavailable">
      <Ban className="mx-auto mb-4 h-8 w-8 text-muted-foreground" aria-hidden="true" />
      <h2 className="text-lg font-semibold">Remote access isn’t available on this deployment</h2>
      <p className="mx-auto mt-2 max-w-prose text-sm text-muted-foreground">
        {reason ?? 'This instance can’t run the tunnel program.'}
      </p>
      <div className="mt-3">
        <FaqLink id="faq-unavailable">Why, and what to do instead</FaqLink>
      </div>
    </section>
  );
}

function TunnelsScreen() {
  const { data: status, isLoading } = useTunnelStatus();
  const { showError, showSuccess } = useToastMessages();
  const [binaryPath, setBinaryPath] = useState('');
  const [certPath, setCertPath] = useState('');
  const [subdomain, setSubdomain] = useState('');
  const [lockNote, setLockNote] = useState(false);
  const [createError, setCreateError] = useState<string | null>(null);

  const { faqProps, reveal } = useFaqRail();
  useShellChrome({
    breadcrumb: BREADCRUMB,
    rail: (
      <FaqRail
        groups={TUNNEL_FAQ_GROUPS}
        subtitle="Everything about getting this working, in one place."
        {...faqProps}
      />
    ),
    railHeader: <FaqRailHeader />,
    railDefaultOpen: false,
    // The help rail is always published, so it must not open itself the way an
    // on-demand rail (a selected row's details) should.
    railAutoOpen: false,
    railWidth: 360,
  });

  useEffect(() => {
    if (status?.subdomain) setSubdomain(status.subdomain);
  }, [status?.subdomain]);

  const setup = useTunnelSetup({
    onSuccess: () => showSuccess('Checks complete', 'Remote Access prerequisites were updated.'),
    onError: (message) => showError('Could not validate setup', message),
  });
  const preferences = useUpdateTunnelPreferences({
    onError: (message) => showError('Could not save preference', message),
  });
  const sync = useSyncTunnelAuthorization({
    onSuccess: (next) => {
      if (next.auth_sync.state === 'synced') showSuccess('Sign-in redirect synced', 'Remote login is available.');
    },
    onError: (message) => showError('Sync failed', message),
  });
  const enable = useEnableTunnel({
    onSuccess: () => {
      setCreateError(null);
      showSuccess('Creating tunnel', 'BodhiApp is connecting to Cloudflare.');
    },
    // A DNS conflict is identified by the refetched status, not by this message.
    onError: (message) => setCreateError(message),
  });
  const disable = useDisableTunnel({
    onSuccess: () => {
      setCreateError(null);
      showSuccess('Remote access off', 'The connector has stopped. Configuration was kept.');
    },
    onError: (message) => showError('Could not turn off remote access', message),
  });

  if (isLoading || !status) {
    return (
      <main
        className="mx-auto w-full max-w-3xl p-4 text-sm text-muted-foreground sm:p-6"
        data-testid="tunnels-page"
        data-pagestatus="loading"
      >
        Loading Remote Access…
      </main>
    );
  }

  const state = deriveRemoteAccessState(status);
  const create = (replaceDns: boolean) => {
    setCreateError(null);
    enable.mutate({ subdomain: subdomain.trim(), auto_reconnect: status.auto_reconnect, replace_dns: replaceDns });
  };

  return (
    <main
      className="mx-auto w-full max-w-3xl space-y-6 p-4 sm:p-6"
      data-testid="tunnels-page"
      data-pagestatus="ready"
      data-test-state={state}
    >
      {/* Panels several levels down link into the rail; the provider saves
          threading `reveal` through every step component. */}
      <FaqRevealProvider reveal={reveal}>
        <header className="space-y-1">
          <h1 className="text-2xl font-semibold">Remote Access</h1>
          <p className="text-sm text-muted-foreground">
            Reach this instance from outside your network through a Cloudflare tunnel on your own domain. Admin-only,
            instance-wide.
          </p>
        </header>

        {state === 'unavailable' ? (
          <Unavailable reason={status.unavailable_reason} />
        ) : (
          <section className="rounded-lg border">
            <div className="flex flex-wrap items-start justify-between gap-3 border-b p-4 sm:p-5">
              <div>
                <h2 className="text-lg font-semibold">Set up remote access</h2>
                <p className="mt-1 text-sm text-muted-foreground">Three steps, on this machine.</p>
              </div>
              <StatusPill
                tone={toneOf(state)}
                label={overallLabel(state)}
                spin={toneOf(state) === 'checking'}
                testId="tunnel-overall-pill"
              />
            </div>
            <div className="p-4 sm:p-5">
              <BinaryStep
                state={state}
                status={status}
                value={binaryPath}
                onChange={setBinaryPath}
                onCheck={() => setup.mutate({ cloudflared_path: binaryPath })}
                checking={setup.isPending}
                onLocked={() => setLockNote(true)}
              />
              <LoginStep
                state={state}
                status={status}
                value={certPath}
                onChange={setCertPath}
                onCheck={() => setup.mutate({ origin_cert_path: certPath })}
                checking={setup.isPending}
                onLocked={() => setLockNote(true)}
              />
              <AddressStep
                state={state}
                status={status}
                subdomain={subdomain}
                setSubdomain={setSubdomain}
                onCreate={create}
                onDisable={() => disable.mutate(undefined)}
                onRetrySync={() => sync.mutate(undefined)}
                onAutoReconnect={(value) => preferences.mutate({ auto_reconnect: value })}
                pending={{
                  enable: enable.isPending,
                  disable: disable.isPending,
                  sync: sync.isPending,
                  preferences: preferences.isPending,
                }}
                createError={createError}
              />
              {lockNote && (
                <Note tone="info" title="Turn remote access off first" testId="tunnel-lock-note">
                  <p>A different binary or domain means a new tunnel.</p>
                </Note>
              )}
            </div>
          </section>
        )}
      </FaqRevealProvider>
    </main>
  );
}

function TunnelsPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <TunnelsScreen />
    </AppInitializer>
  );
}
