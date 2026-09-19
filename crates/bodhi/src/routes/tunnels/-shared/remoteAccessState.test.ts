import type { TunnelStatus } from '@bodhiapp/ts-client';
import { describe, expect, it } from 'vitest';

import {
  addressPill,
  deriveRemoteAccessState,
  isLocked,
  isSetupDone,
  overallLabel,
  validateSubdomain,
} from './remoteAccessState';

function status(overrides: Partial<TunnelStatus> = {}): TunnelStatus {
  return {
    available: true,
    unavailable_reason: null,
    enabled: false,
    state: 'disabled',
    binary: {
      state: 'ready',
      path: '/opt/homebrew/bin/cloudflared',
      source: 'path',
      version: '2026.9.1',
      minimum_version: '2025.2.0',
      error: null,
    },
    login: {
      state: 'ready',
      cert_path: '~/.cloudflared/cert.pem',
      source: 'standard_location',
      zone: 'example.com',
      error: null,
    },
    hostname: null,
    subdomain: null,
    auto_reconnect: true,
    public_url: null,
    oauth_redirect_uri: null,
    auth_sync: { state: 'not_attempted', error: null },
    error_code: null,
    error_message: null,
    ...overrides,
  } as TunnelStatus;
}

describe('deriveRemoteAccessState', () => {
  it('reports unavailable before looking at anything else', () => {
    expect(deriveRemoteAccessState(status({ available: false, binary: { ...status().binary, state: 'ready' } }))).toBe(
      'unavailable'
    );
  });

  it.each([
    ['missing', 'binary-missing'],
    ['waiting', 'binary-missing'],
    ['unsupported', 'binary-old'],
    ['invalid', 'path-invalid'],
  ] as const)('maps binary state %s to %s', (binaryState, expected) => {
    expect(deriveRemoteAccessState(status({ binary: { ...status().binary, state: binaryState } }))).toBe(expected);
  });

  it.each([
    ['missing', 'login-needed'],
    ['waiting', 'login-needed'],
    ['invalid', 'login-failed'],
  ] as const)('maps login state %s to %s once the binary is ready', (loginState, expected) => {
    expect(deriveRemoteAccessState(status({ login: { ...status().login, state: loginState } }))).toBe(expected);
  });

  it('prefers the binary problem when both checks fail', () => {
    const both = status({
      binary: { ...status().binary, state: 'missing' },
      login: { ...status().login, state: 'invalid' },
    });
    expect(deriveRemoteAccessState(both)).toBe('binary-missing');
  });

  it('asks for an address when the checks pass and nothing is saved', () => {
    expect(deriveRemoteAccessState(status())).toBe('address');
  });

  it('reports off, not address, once a hostname is saved', () => {
    expect(deriveRemoteAccessState(status({ hostname: 'bodhi.example.com' }))).toBe('off');
  });

  it('reports creating while the connector is coming up', () => {
    expect(deriveRemoteAccessState(status({ state: 'connecting' }))).toBe('creating');
  });

  it('separates a DNS conflict from any other provisioning failure', () => {
    expect(deriveRemoteAccessState(status({ state: 'failed', error_code: 'dns_conflict' }))).toBe('dns-conflict');
    expect(deriveRemoteAccessState(status({ state: 'failed', error_code: 'cloudflared_exited' }))).toBe(
      'create-failed'
    );
    expect(deriveRemoteAccessState(status({ state: 'failed' }))).toBe('create-failed');
  });

  it('surfaces a DNS conflict that left the connection state disabled', () => {
    // enable() records the code and returns an error without starting a connector.
    expect(deriveRemoteAccessState(status({ state: 'disabled', error_code: 'dns_conflict' }))).toBe('dns-conflict');
    expect(
      deriveRemoteAccessState(status({ state: 'disabled', error_code: 'dns_conflict', hostname: 'bodhi.example.com' }))
    ).toBe('dns-conflict');
  });

  it.each([
    ['syncing', 'kc-syncing'],
    ['unreachable', 'kc-fail-net'],
    ['rejected', 'kc-fail-auth'],
    ['synced', 'live'],
    ['not_attempted', 'live'],
  ] as const)('maps auth sync %s on a connected tunnel to %s', (syncState, expected) => {
    expect(deriveRemoteAccessState(status({ state: 'connected', auth_sync: { state: syncState, error: null } }))).toBe(
      expected
    );
  });
});

describe('derived affordances', () => {
  it('locks the form only while the connector is up or coming up', () => {
    expect(isLocked('live')).toBe(true);
    expect(isLocked('kc-syncing')).toBe(true);
    expect(isLocked('kc-fail-net')).toBe(true);
    expect(isLocked('kc-fail-auth')).toBe(true);
    expect(isLocked('creating')).toBe(true);
    expect(isLocked('off')).toBe(false);
    expect(isLocked('address')).toBe(false);
  });

  it('leaves the form editable after a failed start, so the user can correct it', () => {
    // `status.enabled` is true here (the child exists); locking on it would trap the user.
    expect(isLocked('create-failed')).toBe(false);
    expect(isLocked('dns-conflict')).toBe(false);
  });

  it('treats step 3 as reachable only after both checks pass', () => {
    expect(isSetupDone('address')).toBe(true);
    expect(isSetupDone('off')).toBe(true);
    expect(isSetupDone('binary-missing')).toBe(false);
    expect(isSetupDone('login-needed')).toBe(false);
  });

  it('labels the tunnel as on whichever way the sign-in sync failed', () => {
    expect(addressPill('kc-fail-net')).toBe('On · sign-in broken');
    expect(addressPill('kc-fail-auth')).toBe('On · sign-in broken');
    expect(addressPill('kc-syncing')).toBe('On · syncing');
    expect(addressPill('live')).toBe('On');
    expect(overallLabel('kc-fail-net')).toBe('Sign-in broken');
    expect(overallLabel('kc-fail-auth')).toBe('Sign-in broken');
  });
});

describe('validateSubdomain', () => {
  it('accepts a single label', () => {
    expect(validateSubdomain('bodhi')).toBeNull();
    expect(validateSubdomain('  bodhi-01  ')).toBeNull();
  });

  it('rejects a dotted name, which Universal SSL cannot serve over HTTPS', () => {
    expect(validateSubdomain('my.bodhi')).toMatch(/no dots/i);
  });

  it('rejects empty, overlong and out-of-charset input', () => {
    expect(validateSubdomain('   ')).toMatch(/enter a subdomain/i);
    expect(validateSubdomain('a'.repeat(64))).toMatch(/63 characters/i);
    expect(validateSubdomain('-bodhi')).toMatch(/letters, numbers/i);
    expect(validateSubdomain('bodhi_app')).toMatch(/letters, numbers/i);
  });
});
