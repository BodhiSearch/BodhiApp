import { describe, it, expect, beforeEach } from 'vitest';

import { isUiV2Enabled, UI_V2_FLAG_PREFIX } from './uiV2Flags';

describe('uiV2Flags', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('defaults to false (old screen) when the flag is unset', () => {
    expect(isUiV2Enabled('mcp-discover')).toBe(false);
  });

  it('returns true when the flag is set to "true"', () => {
    localStorage.setItem(`${UI_V2_FLAG_PREFIX}mcp-discover`, 'true');
    expect(isUiV2Enabled('mcp-discover')).toBe(true);
  });

  it('returns false for any non-"true" value', () => {
    localStorage.setItem(`${UI_V2_FLAG_PREFIX}chat`, '1');
    expect(isUiV2Enabled('chat')).toBe(false);
  });

  it('is independent per screen', () => {
    localStorage.setItem(`${UI_V2_FLAG_PREFIX}new-mcp`, 'true');
    expect(isUiV2Enabled('new-mcp')).toBe(true);
    expect(isUiV2Enabled('mcp-discover')).toBe(false);
  });
});
