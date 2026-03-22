import { beforeEach, describe, expect, it, vi } from 'vitest';

import { handleSmartRedirect } from '@/lib/utils';
import { mockWindowLocation } from '@/tests/wrapper';

describe('handleSmartRedirect', () => {
  let navigateMock: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    navigateMock = vi.fn();
    mockWindowLocation('http://localhost:3000/ui/auth/callback');
  });

  it('handles relative path without BASE_PATH', () => {
    handleSmartRedirect('/login', navigateMock);
    expect(navigateMock).toHaveBeenCalledWith({ to: '/login' });
  });

  it('strips BASE_PATH from relative path', () => {
    handleSmartRedirect('/ui/chat', navigateMock);
    expect(navigateMock).toHaveBeenCalledWith({ to: '/chat' });
  });

  it('strips BASE_PATH to root', () => {
    handleSmartRedirect('/ui', navigateMock);
    expect(navigateMock).toHaveBeenCalledWith({ to: '/' });
  });

  it('handles relative path with query params', () => {
    handleSmartRedirect('/ui/chat?model=llama', navigateMock);
    expect(navigateMock).toHaveBeenCalledWith({ to: '/chat', search: { model: 'llama' } });
  });

  it('handles relative path with query and hash (hash ignored)', () => {
    handleSmartRedirect('/ui/setup/download-models?step=1#section', navigateMock);
    expect(navigateMock).toHaveBeenCalledWith({
      to: '/setup/download-models',
      search: { step: '1' },
    });
  });

  it('handles same-origin absolute URL', () => {
    handleSmartRedirect('http://localhost:3000/ui/chat', navigateMock);
    expect(navigateMock).toHaveBeenCalledWith({ to: '/chat' });
  });

  it('handles same-origin absolute URL with query params', () => {
    handleSmartRedirect('http://localhost:3000/ui/chat?model=llama', navigateMock);
    expect(navigateMock).toHaveBeenCalledWith({ to: '/chat', search: { model: 'llama' } });
  });

  it('redirects to external URL via window.location.href', () => {
    handleSmartRedirect('https://external.example.com/dashboard', navigateMock);
    expect(navigateMock).not.toHaveBeenCalled();
    expect(window.location.href).toBe('https://external.example.com/dashboard');
  });

  it('treats different port as external URL', () => {
    handleSmartRedirect('http://localhost:8080/ui/chat', navigateMock);
    expect(navigateMock).not.toHaveBeenCalled();
    expect(window.location.href).toBe('http://localhost:8080/ui/chat');
  });

  it('treats different protocol as external URL', () => {
    handleSmartRedirect('https://localhost:3000/ui/chat', navigateMock);
    expect(navigateMock).not.toHaveBeenCalled();
    expect(window.location.href).toBe('https://localhost:3000/ui/chat');
  });

  it('blocks navigation for malformed URLs (XSS protection)', () => {
    handleSmartRedirect('invalid-url-format', navigateMock);
    expect(navigateMock).not.toHaveBeenCalled();
    // safeNavigate blocks malformed URLs — location remains unchanged
    expect(window.location.href).not.toBe('invalid-url-format');
  });
});
