import { describe, it, expect } from 'vitest';

import { isBareRoute, resolveShellRoute } from './resolveShellRoute';

describe('isBareRoute', () => {
  it.each([
    '/',
    '/home/',
    '/setup/',
    '/setup/download-models/',
    '/login/',
    '/auth/callback/',
    '/request-access/',
    '/mcps/oauth/callback/',
    // the standalone OAuth access-request review (in scope, rendered via BareLayout)
    '/apps/access-requests/review/',
  ])('treats %s as bare', (p) => {
    expect(isBareRoute(p)).toBe(true);
  });

  it.each(['/chat/', '/models/', '/tokens/', '/settings/', '/users/', '/mcps/'])('treats %s as a shell route', (p) => {
    expect(isBareRoute(p)).toBe(false);
  });

  it('handles the /ui basepath prefix', () => {
    expect(isBareRoute('/ui/login/')).toBe(true);
    expect(isBareRoute('/ui/chat/')).toBe(false);
  });
});

describe('resolveShellRoute', () => {
  it('returns null for bare routes', () => {
    expect(resolveShellRoute('/login/')).toBeNull();
  });

  it('resolves section landings', () => {
    expect(resolveShellRoute('/chat/')).toEqual({ section: 'chat', subPage: null });
    expect(resolveShellRoute('/settings/')).toEqual({ section: 'settings', subPage: 'app-settings' });
  });

  it('longest-prefix matches sub-pages over section landing', () => {
    expect(resolveShellRoute('/models/alias/new/')).toEqual({ section: 'models', subPage: 'new-local-model' });
    expect(resolveShellRoute('/tokens/new/')).toEqual({ section: 'api-keys', subPage: 'new-token' });
  });

  it('maps both /users/ routes to the users section', () => {
    expect(resolveShellRoute('/users/')).toEqual({ section: 'users', subPage: 'manage-users' });
    expect(resolveShellRoute('/users/access-requests/')).toEqual({ section: 'users', subPage: 'access-requests' });
  });

  it('resolves the tokens landing to the api-tokens sub-page', () => {
    expect(resolveShellRoute('/tokens/')).toEqual({ section: 'api-keys', subPage: 'api-tokens' });
  });

  it('falls back to the section landing for an app route not in the nav', () => {
    // An unknown sub-path under /models/ matches the models section (my-models shares the /models/ href).
    expect(resolveShellRoute('/models/unknown-subpage/')).toEqual({ section: 'models', subPage: 'my-models' });
  });
});
