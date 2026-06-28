import { describe, it, expect } from 'vitest';

import { isBareRoute } from './resolveShellRoute';

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
