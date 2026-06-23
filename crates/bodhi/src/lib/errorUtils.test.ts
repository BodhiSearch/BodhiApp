import { AxiosError } from 'axios';
import { describe, expect, it } from 'vitest';
import { extractErrorCode, extractErrorMessage } from './errorUtils';

function axiosError(data: unknown, message = 'Request failed'): AxiosError {
  const err = new AxiosError(message);
  // @ts-expect-error — partial response is enough for extraction
  err.response = { data };
  return err;
}

describe('extractErrorMessage', () => {
  it('prefers the structured Bodhi envelope message', () => {
    const err = axiosError({ error: { message: 'Name already taken', type: 'validation' } });
    expect(extractErrorMessage(err, 'fallback')).toBe('Name already taken');
  });

  it('falls back to the raw Error message when no envelope', () => {
    const err = axiosError(undefined, 'Network Error');
    expect(extractErrorMessage(err, 'fallback')).toBe('Network Error');
  });

  it('falls back to the supplied default for unknown shapes', () => {
    expect(extractErrorMessage({}, 'Something broke')).toBe('Something broke');
    expect(extractErrorMessage(null, 'Something broke')).toBe('Something broke');
    expect(extractErrorMessage('a string', 'Something broke')).toBe('Something broke');
  });

  it('uses the fallback (not the raw axios message) for a message-less envelope', () => {
    const err = axiosError({ error: { message: '', type: 'x' } }, 'Request failed with status code 500');
    expect(extractErrorMessage(err, 'Friendly fallback')).toBe('Friendly fallback');
  });
});

describe('extractErrorCode', () => {
  it('returns the envelope code when present', () => {
    const err = axiosError({ error: { message: 'm', type: 't', code: 'model_not_found' } });
    expect(extractErrorCode(err)).toBe('model_not_found');
  });

  it('returns undefined when code is absent or null', () => {
    expect(extractErrorCode(axiosError({ error: { message: 'm', type: 't' } }))).toBeUndefined();
    expect(extractErrorCode(axiosError({ error: { message: 'm', type: 't', code: null } }))).toBeUndefined();
    expect(extractErrorCode({})).toBeUndefined();
  });
});
