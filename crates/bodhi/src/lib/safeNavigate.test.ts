import { describe, it, expect, vi, beforeEach } from 'vitest';
import { safeNavigate } from './safeNavigate';

describe('safeNavigate', () => {
  const originalLocation = window.location;

  beforeEach(() => {
    Object.defineProperty(window, 'location', {
      writable: true,
      value: { ...originalLocation, href: '' },
    });
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  it.each([
    ['http://example.com/callback', 'http URL'],
    ['https://example.com/callback', 'https URL'],
    ['/ui/chat', 'relative URL'],
  ])('allows safe URL: %s (%s)', (url) => {
    expect(safeNavigate(url)).toBe(true);
    expect(window.location.href).toBe(url);
  });

  it.each([
    ["javascript:alert('xss')", 'javascript: scheme'],
    ["javascript:fetch('/dev/secrets').then(r=>r.json()).then(d=>{document.body.innerHTML='XSS'})//", 'javascript: with payload'],
    ['JAVASCRIPT:alert(1)', 'uppercase scheme'],
    ['jAvAsCrIpT:alert(1)', 'mixed case scheme'],
    ['  javascript:alert(1)', 'leading whitespace'],
    ['\tjavascript:alert(1)', 'tab prefix'],
    ['data:text/html,<script>alert(1)</script>', 'data: URI'],
    ['vbscript:MsgBox(1)', 'vbscript: URI'],
  ])('blocks dangerous URL: %s (%s)', (url) => {
    expect(safeNavigate(url)).toBe(false);
    expect(window.location.href).toBe('');
  });

  it.each([
    ['', 'empty string'],
    ['   ', 'whitespace only'],
  ])('blocks empty/blank URL: %s (%s)', (url) => {
    expect(safeNavigate(url)).toBe(false);
    expect(console.error).toHaveBeenCalledWith('Blocked navigation to empty URL');
  });
});
