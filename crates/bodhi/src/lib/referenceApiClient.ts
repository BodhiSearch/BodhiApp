/**
 * Client for the external reference API (e.g. https://api.getbodhi.app/).
 *
 * Unlike `apiClient` (axios, same-origin BodhiApp backend), this is a thin `fetch` wrapper that
 * calls an EXTERNAL origin directly from the browser, sending the user's OIDC `id_token` as a
 * Bearer credential so the reference API can rate-limit / attribute per user. The base URL comes
 * from `AppInfo.reference_api_url` (configurable, env-overridable) — see `useReferenceApi`.
 *
 * `fetch` (not axios) is deliberate: MSW intercepts external absolute URLs cleanly, and it keeps
 * this client obviously separate from the same-origin `apiClient`.
 */

export class ReferenceApiError extends Error {
  constructor(
    public status: number,
    public body: string
  ) {
    super(`Reference API error ${status}: ${body}`);
    this.name = 'ReferenceApiError';
  }
}

export interface ReferenceApiClient {
  get<T>(path: string, init?: RequestInit): Promise<T>;
}

export function createReferenceApiClient(baseUrl: string, idToken: string | undefined): ReferenceApiClient {
  const request = async <T>(path: string, init?: RequestInit): Promise<T> => {
    const url = new URL(path, baseUrl).toString();
    const headers = new Headers(init?.headers);
    if (idToken) {
      headers.set('Authorization', `Bearer ${idToken}`);
    }
    const res = await fetch(url, { ...init, headers });
    if (!res.ok) {
      throw new ReferenceApiError(res.status, await res.text().catch(() => ''));
    }
    return (await res.json()) as T;
  };

  return {
    get: <T>(path: string, init?: RequestInit) => request<T>(path, { ...init, method: 'GET' }),
  };
}
