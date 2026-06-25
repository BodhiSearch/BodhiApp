import { renderHook, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';

import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import { mockSearchRepos } from '@/test-utils/msw-v2/handlers/reference-models';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

import { useSearchRepos } from './useSearchRepos';

setupMswV2();

let Wrapper: ReturnType<typeof createWrapper>;

beforeEach(() => {
  Wrapper = createWrapper();
  server.use(
    ...mockAppInfoReady(),
    // Anonymous read — a logged-in user is present but the repo search must not send the id_token.
    ...mockUserLoggedIn({ username: 'admin@example.com', role: 'resource_admin', id_token: 'test-id-token' })
  );
});

describe('useSearchRepos', () => {
  it('is disabled on empty search (no request)', async () => {
    let seen: { url: URL; authorization: string | null } | null = null;
    server.use(...mockSearchRepos({ onRequest: (info) => (seen = info) }));

    const { result } = renderHook(() => useSearchRepos({ search: '   ', filter: 'gguf' }), { wrapper: Wrapper });

    // Nothing fires; query stays idle.
    await new Promise((r) => setTimeout(r, 50));
    expect(seen).toBeNull();
    expect(result.current.data).toBeUndefined();
  });

  it('sends search + filter=gguf, no Bearer, and returns the suggestion items', async () => {
    let seen: { url: URL; authorization: string | null } | null = null;
    server.use(
      ...mockSearchRepos({
        ids: ['Qwen/Qwen3-Coder-32B-GGUF', 'Qwen/Qwen2.5-7B-Instruct-GGUF'],
        onRequest: (info) => (seen = info),
      })
    );

    const { result } = renderHook(() => useSearchRepos({ search: 'qwen', filter: 'gguf', limit: 10 }), {
      wrapper: Wrapper,
    });

    await waitFor(() => expect(result.current.data).toBeDefined());
    expect(result.current.data).toEqual([{ id: 'Qwen/Qwen3-Coder-32B-GGUF' }, { id: 'Qwen/Qwen2.5-7B-Instruct-GGUF' }]);
    expect(seen!.url.searchParams.get('search')).toBe('qwen');
    expect(seen!.url.searchParams.getAll('filter')).toEqual(['gguf']);
    expect(seen!.url.searchParams.get('limit')).toBe('10');
    expect(seen!.authorization).toBeNull();
  });
});
