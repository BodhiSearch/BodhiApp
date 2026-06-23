import { renderHook, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';

import { useApiModelForm } from '@/components/api-models/hooks/useApiModelForm';
import { mockApiFormats } from '@/test-utils/msw-v2/handlers/api-models';
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

// The Explore catalog "Configure in Bodhi" bridge prefills create-mode via the `prefill` prop.
setupMswV2();

let Wrapper: ReturnType<typeof createWrapper>;

beforeEach(() => {
  Wrapper = createWrapper();
  server.use(
    ...mockAppInfoReady(),
    ...mockUserLoggedIn({ username: 'admin@example.com', role: 'resource_admin' }),
    ...mockApiFormats({ data: ['openai', 'anthropic', 'gemini'] }, { stub: true })
  );
});

describe('useApiModelForm — create-mode prefill (Configure bridge)', () => {
  it('seeds api_format, base_url, and the model from prefill', async () => {
    const { result } = renderHook(
      () =>
        useApiModelForm({
          mode: 'create',
          prefill: { api_format: 'anthropic', base_url: 'https://api.anthropic.com/v1', model: 'claude-sonnet-4.5' },
        }),
      { wrapper: Wrapper }
    );

    await waitFor(() => expect(result.current.watchedValues.api_format).toBe('anthropic'));
    expect(result.current.watchedValues.base_url).toBe('https://api.anthropic.com/v1');
    expect(result.current.watchedValues.models).toEqual(['claude-sonnet-4.5']);
    // API key is never prefilled.
    expect(result.current.watchedValues.api_key).toBe('');
  });

  it('falls back to OpenAI defaults when no prefill is given', async () => {
    const { result } = renderHook(() => useApiModelForm({ mode: 'create' }), { wrapper: Wrapper });
    await waitFor(() => expect(result.current.watchedValues.api_format).toBe('openai'));
    expect(result.current.watchedValues.base_url).toBe('https://api.openai.com/v1');
    expect(result.current.watchedValues.models).toEqual([]);
  });

  it('keeps the preset base_url when prefill omits base_url (bridge gave null)', async () => {
    const { result } = renderHook(
      () => useApiModelForm({ mode: 'create', prefill: { api_format: 'anthropic', model: 'claude-x' } }),
      { wrapper: Wrapper }
    );
    await waitFor(() => expect(result.current.watchedValues.api_format).toBe('anthropic'));
    // base_url not provided → falls back to the hardcoded default (form preset wiring takes over).
    expect(result.current.watchedValues.base_url).toBe('https://api.openai.com/v1');
    expect(result.current.watchedValues.models).toEqual(['claude-x']);
  });
});
