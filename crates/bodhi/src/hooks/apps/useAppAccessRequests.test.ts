import type { BodhiErrorResponse } from '@bodhiapp/ts-client';
import { act, renderHook, waitFor } from '@testing-library/react';
import { AxiosError } from 'axios';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { useGetConsentContext, useSubmitConsent } from '@/hooks/apps';
import { mockConsentOk, mockConsentErrorInApp } from '@/test-fixtures/apps';
import {
  mockConsentContext,
  mockConsentContextError,
  mockSubmitConsent,
  mockSubmitConsentError,
} from '@/test-utils/msw-v2/handlers/apps';
import { setupMswV2, server } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

setupMswV2();

afterEach(() => server.resetHandlers());

describe('useGetConsentContext', () => {
  it('fetches the consent context appending the raw search string verbatim', async () => {
    let requestedUrl = '';
    server.use(...mockConsentContext(mockConsentOk(), { onUrl: (url) => (requestedUrl = url) }));

    // Pre-encoded search must reach the server unchanged (no re-encoding).
    const search = '?client_id=test-app-client&scope=openid%20scope_user_user';
    const { result } = renderHook(() => useGetConsentContext(search), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data?.result).toBe('ok');
    expect(new URL(requestedUrl).search).toBe(search);
  });

  it('returns the structured error union on a 200 error result', async () => {
    server.use(...mockConsentContext(mockConsentErrorInApp));

    const { result } = renderHook(() => useGetConsentContext('?client_id=x'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data).toMatchObject({
      result: 'error',
      error: 'invalid_request',
    });
  });

  it('handles an infrastructure error response', async () => {
    server.use(...mockConsentContextError({ message: 'Internal server error', status: 500 }));

    const { result } = renderHook(() => useGetConsentContext('?client_id=x'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    const error = result.current.error as AxiosError<BodhiErrorResponse>;
    expect(error.response?.status).toBe(500);
  });
});

describe('useSubmitConsent', () => {
  it('posts the consent decision and calls onSuccess with the redirect_url', async () => {
    const onSuccess = vi.fn();
    let capturedBody: unknown = null;
    server.use(
      ...mockSubmitConsent(
        { id: 'req-1', redirect_url: 'https://id.example.com/auth?scope=x' },
        { status: 201, onBody: (body) => (capturedBody = body) }
      )
    );

    const { result } = renderHook(() => useSubmitConsent({ onSuccess }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({
        query: 'client_id=test-app-client&response_type=code',
        decision: 'approve',
        approved_role: 'scope_user_user',
        approved: { version: '1' as const, mcps: [] },
      });
    });

    expect(capturedBody).toMatchObject({
      query: 'client_id=test-app-client&response_type=code',
      decision: 'approve',
      approved_role: 'scope_user_user',
    });
    expect(onSuccess).toHaveBeenCalledWith(
      expect.objectContaining({ id: 'req-1', redirect_url: 'https://id.example.com/auth?scope=x' })
    );
  });

  it('posts a deny decision without grant fields', async () => {
    const onSuccess = vi.fn();
    let capturedBody: unknown = null;
    server.use(
      ...mockSubmitConsent(
        { redirect_url: 'https://myapp.example.com/callback?error=access_denied' },
        { onBody: (body) => (capturedBody = body) }
      )
    );

    const { result } = renderHook(() => useSubmitConsent({ onSuccess }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({ query: 'client_id=test-app-client', decision: 'deny' });
    });

    expect(capturedBody).toMatchObject({ query: 'client_id=test-app-client', decision: 'deny' });
    expect(onSuccess).toHaveBeenCalledWith(
      expect.objectContaining({ redirect_url: 'https://myapp.example.com/callback?error=access_denied' })
    );
  });

  it('calls onError with the extracted message on failure', async () => {
    const onError = vi.fn();
    server.use(...mockSubmitConsentError({ message: 'privilege escalation', status: 403 }));

    const { result } = renderHook(() => useSubmitConsent({ onError }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      try {
        await result.current.mutateAsync({ query: 'client_id=x', decision: 'approve' });
      } catch {
        /* expected */
      }
    });

    expect(onError).toHaveBeenCalledWith('privilege escalation');
  });
});
