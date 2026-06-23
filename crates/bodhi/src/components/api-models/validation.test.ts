import { describe, expect, it } from 'vitest';

import {
  computeCanFetch,
  computeCanTest,
  computeShowExtras,
  hasEnvelope,
  isLlmLibertyCreate,
  llmLibertyFetchDisabledReason,
  llmLibertyTestDisabledReason,
  presetBaseUrl,
  presetExtras,
} from './validation';

describe('isLlmLibertyCreate', () => {
  it('is true for llm_liberty_oauth in create mode', () => {
    expect(isLlmLibertyCreate('llm_liberty_oauth', false)).toBe(true);
  });

  it('is false for llm_liberty_oauth in edit mode', () => {
    expect(isLlmLibertyCreate('llm_liberty_oauth', true)).toBe(false);
  });

  it('is false for other formats', () => {
    expect(isLlmLibertyCreate('openai', false)).toBe(false);
    expect(isLlmLibertyCreate(undefined, false)).toBe(false);
  });
});

describe('hasEnvelope', () => {
  it('is true for non-whitespace content', () => {
    expect(hasEnvelope('{"a":1}')).toBe(true);
  });

  it('is false for empty/whitespace/undefined', () => {
    expect(hasEnvelope('')).toBe(false);
    expect(hasEnvelope('   ')).toBe(false);
    expect(hasEnvelope(undefined)).toBe(false);
  });
});

describe('computeCanTest', () => {
  describe('non-liberty path', () => {
    it('requires base_url', () => {
      expect(computeCanTest({ api_format: 'openai', base_url: 'https://x' }, false)).toBe(true);
      expect(computeCanTest({ api_format: 'openai', base_url: '' }, false)).toBe(false);
      expect(computeCanTest({ api_format: 'openai' }, false)).toBe(false);
    });

    it('llm_liberty_oauth in edit mode uses base_url path', () => {
      expect(computeCanTest({ api_format: 'llm_liberty_oauth', base_url: 'https://x' }, true)).toBe(true);
      expect(computeCanTest({ api_format: 'llm_liberty_oauth', base_url: '' }, true)).toBe(false);
    });
  });

  describe('liberty create path', () => {
    it('requires envelope and at least one model', () => {
      expect(
        computeCanTest({ api_format: 'llm_liberty_oauth', llm_liberty_envelope: '{}', models: ['m'] }, false)
      ).toBe(true);
    });

    it('is false without envelope', () => {
      expect(computeCanTest({ api_format: 'llm_liberty_oauth', models: ['m'] }, false)).toBe(false);
    });

    it('is false without a model', () => {
      expect(computeCanTest({ api_format: 'llm_liberty_oauth', llm_liberty_envelope: '{}', models: [] }, false)).toBe(
        false
      );
      expect(computeCanTest({ api_format: 'llm_liberty_oauth', llm_liberty_envelope: '{}' }, false)).toBe(false);
    });
  });
});

describe('computeCanFetch', () => {
  it('non-liberty path requires base_url', () => {
    expect(computeCanFetch({ api_format: 'openai', base_url: 'https://x' }, false)).toBe(true);
    expect(computeCanFetch({ api_format: 'openai' }, false)).toBe(false);
  });

  it('liberty create path requires only envelope', () => {
    expect(computeCanFetch({ api_format: 'llm_liberty_oauth', llm_liberty_envelope: '{}' }, false)).toBe(true);
    expect(computeCanFetch({ api_format: 'llm_liberty_oauth' }, false)).toBe(false);
  });
});

describe('llmLibertyTestDisabledReason', () => {
  it('returns null on the non-liberty path', () => {
    expect(llmLibertyTestDisabledReason({ api_format: 'openai' }, false)).toBeNull();
    expect(llmLibertyTestDisabledReason({ api_format: 'llm_liberty_oauth' }, true)).toBeNull();
  });

  it('asks for envelope when missing', () => {
    expect(llmLibertyTestDisabledReason({ api_format: 'llm_liberty_oauth' }, false)).toBe(
      'Paste the LLM Liberty envelope to test connection'
    );
  });

  it('asks for a model when envelope present but no model', () => {
    expect(llmLibertyTestDisabledReason({ api_format: 'llm_liberty_oauth', llm_liberty_envelope: '{}' }, false)).toBe(
      'You need to add at least one model to test connection'
    );
  });

  it('returns empty string when satisfied', () => {
    expect(
      llmLibertyTestDisabledReason(
        { api_format: 'llm_liberty_oauth', llm_liberty_envelope: '{}', models: ['m'] },
        false
      )
    ).toBe('');
  });
});

describe('llmLibertyFetchDisabledReason', () => {
  it('returns null on the non-liberty path', () => {
    expect(llmLibertyFetchDisabledReason({ api_format: 'openai' }, false)).toBeNull();
    expect(llmLibertyFetchDisabledReason({ api_format: 'llm_liberty_oauth' }, true)).toBeNull();
  });

  it('asks for envelope when missing', () => {
    expect(llmLibertyFetchDisabledReason({ api_format: 'llm_liberty_oauth' }, false)).toBe(
      'Paste the LLM Liberty envelope to fetch models'
    );
  });

  it('returns empty string when envelope present', () => {
    expect(llmLibertyFetchDisabledReason({ api_format: 'llm_liberty_oauth', llm_liberty_envelope: '{}' }, false)).toBe(
      ''
    );
  });
});

describe('computeShowExtras', () => {
  it('is true for presets declaring defaults', () => {
    expect(computeShowExtras('anthropic_oauth')).toBe(true);
  });

  it('is false for presets without defaults', () => {
    expect(computeShowExtras('openai')).toBe(false);
    expect(computeShowExtras('gemini')).toBe(false);
  });

  it('is false for unknown or missing format', () => {
    expect(computeShowExtras('not_a_format')).toBe(false);
    expect(computeShowExtras(undefined)).toBe(false);
  });
});

describe('presetExtras', () => {
  it('serializes default headers and body for a preset that declares them', () => {
    const { headers, body } = presetExtras('anthropic_oauth');
    expect(headers).toContain('anthropic-version');
    expect(JSON.parse(headers)).toMatchObject({ 'anthropic-version': '2023-06-01' });
    expect(JSON.parse(body)).toMatchObject({ max_tokens: 4096 });
  });

  it('returns empty strings for presets without defaults', () => {
    expect(presetExtras('openai')).toEqual({ headers: '', body: '' });
  });

  it('returns empty strings for an unknown format', () => {
    expect(presetExtras('not_a_format')).toEqual({ headers: '', body: '' });
  });
});

describe('presetBaseUrl', () => {
  it('returns the preset base url', () => {
    expect(presetBaseUrl('openai')).toBe('https://api.openai.com/v1');
    expect(presetBaseUrl('anthropic')).toBe('https://api.anthropic.com/v1');
  });

  it('returns empty string for llm_liberty_oauth (envelope-provided)', () => {
    expect(presetBaseUrl('llm_liberty_oauth')).toBe('');
  });

  it('returns empty string for an unknown format', () => {
    expect(presetBaseUrl('not_a_format')).toBe('');
  });
});
