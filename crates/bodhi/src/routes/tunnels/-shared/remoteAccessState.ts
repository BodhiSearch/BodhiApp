import type { TunnelStatus } from '@bodhiapp/ts-client';

/**
 * The page shows exactly one of these at a time and derives every pill, fold
 * and panel from it. Mirrors the state list in `design/tunnels/ra-app.jsx`.
 */
export type RemoteAccessState =
  | 'unavailable'
  | 'binary-missing'
  | 'binary-old'
  | 'path-invalid'
  | 'login-needed'
  | 'login-failed'
  | 'address'
  | 'creating'
  | 'create-failed'
  | 'dns-conflict'
  | 'kc-syncing'
  | 'kc-fail-net'
  | 'kc-fail-auth'
  | 'live'
  | 'off';

export type Tone = 'idle' | 'checking' | 'ok' | 'attn' | 'fail';

const STATE_META: Record<RemoteAccessState, { tone: Tone; overall: string }> = {
  unavailable: { tone: 'idle', overall: 'Unavailable' },
  'binary-missing': { tone: 'attn', overall: 'Needs attention' },
  'binary-old': { tone: 'attn', overall: 'Needs attention' },
  'path-invalid': { tone: 'fail', overall: 'Needs attention' },
  'login-needed': { tone: 'attn', overall: 'Needs attention' },
  'login-failed': { tone: 'fail', overall: 'Needs attention' },
  address: { tone: 'idle', overall: 'Not set up' },
  creating: { tone: 'checking', overall: 'Creating' },
  'create-failed': { tone: 'fail', overall: 'Needs attention' },
  'dns-conflict': { tone: 'attn', overall: 'Needs attention' },
  'kc-syncing': { tone: 'checking', overall: 'Finishing up' },
  'kc-fail-net': { tone: 'attn', overall: 'Sign-in broken' },
  'kc-fail-auth': { tone: 'attn', overall: 'Sign-in broken' },
  live: { tone: 'ok', overall: 'On' },
  off: { tone: 'idle', overall: 'Off' },
};

/** Steps 1 and 2 have passed, so step 3 is reachable. */
const SETUP_DONE: readonly RemoteAccessState[] = [
  'address',
  'creating',
  'create-failed',
  'dns-conflict',
  'kc-syncing',
  'kc-fail-net',
  'kc-fail-auth',
  'live',
  'off',
];

/** The connector is up, whatever the Keycloak sync did. */
const CONNECTED: readonly RemoteAccessState[] = ['kc-syncing', 'kc-fail-net', 'kc-fail-auth', 'live'];

export function deriveRemoteAccessState(status: TunnelStatus): RemoteAccessState {
  if (!status.available) return 'unavailable';

  switch (status.binary.state) {
    case 'invalid':
      return 'path-invalid';
    case 'unsupported':
      return 'binary-old';
    case 'missing':
    case 'waiting':
      return 'binary-missing';
  }

  switch (status.login.state) {
    case 'invalid':
      return 'login-failed';
    case 'missing':
    case 'waiting':
      return 'login-needed';
  }

  switch (status.state) {
    case 'connecting':
      return 'creating';
    case 'connected':
      // The tunnel carries traffic either way; only sign-in through it is at
      // stake, and the two failures need different remedies.
      if (status.auth_sync.state === 'syncing') return 'kc-syncing';
      if (status.auth_sync.state === 'unreachable') return 'kc-fail-net';
      if (status.auth_sync.state === 'rejected') return 'kc-fail-auth';
      return 'live';
    case 'failed':
    case 'disabled':
      // A refused enable records the conflict but leaves the connection state
      // alone, so the code is what identifies it — not the state.
      if (status.error_code === 'dns_conflict') return 'dns-conflict';
      if (status.state === 'failed') return 'create-failed';
      return status.hostname ? 'off' : 'address';
  }
}

export function toneOf(state: RemoteAccessState): Tone {
  return STATE_META[state].tone;
}

export function overallLabel(state: RemoteAccessState): string {
  return STATE_META[state].overall;
}

export function isSetupDone(state: RemoteAccessState): boolean {
  return SETUP_DONE.includes(state);
}

export function isConnected(state: RemoteAccessState): boolean {
  return CONNECTED.includes(state);
}

/**
 * Derived from the connection state rather than `status.enabled`. `enabled` is
 * only `running.is_some()`, so a connector that spawned and then failed would
 * otherwise leave the form read-only with no way to correct it.
 */
export function isLocked(state: RemoteAccessState): boolean {
  return isConnected(state) || state === 'creating';
}

export function addressPill(state: RemoteAccessState): string {
  switch (state) {
    case 'live':
      return 'On';
    case 'kc-syncing':
      return 'On · syncing';
    case 'kc-fail-net':
    case 'kc-fail-auth':
      return 'On · sign-in broken';
    case 'off':
      return 'Off';
    case 'creating':
      return 'Creating';
    case 'create-failed':
      return 'Failed';
    case 'dns-conflict':
      return 'Address in use';
    default:
      return 'Not set';
  }
}

export function addressTone(state: RemoteAccessState): Tone {
  if (!isSetupDone(state)) return 'idle';
  return toneOf(state);
}

/**
 * A single DNS label. Dots are refused rather than quietly accepted: Cloudflare
 * Universal SSL covers `zone` and `*.zone` but not deeper names, so a hostname
 * with a dot in the subdomain cannot serve HTTPS and is useless for sign-in.
 */
export function validateSubdomain(value: string): string | null {
  const subdomain = value.trim();
  if (!subdomain) return 'Enter a subdomain.';
  if (subdomain.includes('.')) return 'One level only — no dots. Use a single word such as “bodhi”.';
  if (subdomain.length > 63) return 'Too long — 63 characters at most.';
  if (!/^[a-z0-9]([a-z0-9-]*[a-z0-9])?$/i.test(subdomain)) return 'Letters, numbers and hyphens only.';
  return null;
}
