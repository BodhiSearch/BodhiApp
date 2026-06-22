import type { AliasResponse } from '@bodhiapp/ts-client';
import { describe, it, expect } from 'vitest';

import { convertApiToForm, convertFormToApi, type AliasFormData } from './alias';

const baseForm: AliasFormData = {
  alias: 'my-alias',
  repo: 'org/repo-GGUF',
  filename: 'repo-Q4_K_M.gguf',
  snapshot: 'main',
  request_params_text: '',
  system_prompt: '',
  context_params: '',
};

describe('alias schema text<->json conversion', () => {
  it('parses request_params_text key=value lines into typed params', () => {
    const api = convertFormToApi({
      ...baseForm,
      request_params_text: 'temperature=0.7\ntop_p=0.9\nmax_tokens=512\nstop=foo,bar',
    });
    expect(api.request_params).toMatchObject({
      temperature: 0.7,
      top_p: 0.9,
      max_tokens: 512,
      stop: ['foo', 'bar'],
    });
  });

  it('folds system_prompt into request_params', () => {
    const api = convertFormToApi({ ...baseForm, system_prompt: 'Be terse.' });
    expect(api.request_params?.system_prompt).toBe('Be terse.');
  });

  it('drops unknown / invalid request param keys', () => {
    const api = convertFormToApi({ ...baseForm, request_params_text: 'bogus=1\ntemperature=not-a-number' });
    expect(api.request_params).not.toHaveProperty('bogus');
    expect(api.request_params).not.toHaveProperty('temperature');
  });

  it('round-trips an existing alias back into editable text', () => {
    const apiData = {
      source: 'user',
      alias: 'my-alias',
      repo: 'org/repo-GGUF',
      filename: 'repo-Q4_K_M.gguf',
      snapshot: 'main',
      request_params: { temperature: 0.3, system_prompt: 'Hello' },
      context_params: ['--ctx-size 2048'],
    } as unknown as AliasResponse;

    const form = convertApiToForm(apiData as never);
    expect(form.system_prompt).toBe('Hello');
    expect(form.request_params_text).toContain('temperature=0.3');
    expect(form.request_params_text).not.toContain('system_prompt');
    expect(form.context_params).toBe('--ctx-size 2048');
  });
});
