import { useState, useCallback } from 'react';
import { loadToken } from '@/lib/storage';

interface ApiResponse {
  status: number;
  statusText: string;
  headers: Record<string, string>;
  body: unknown;
  raw: string;
}

export function useApi(baseUrl?: string) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [response, setResponse] = useState<ApiResponse | null>(null);

  const execute = useCallback(
    async (
      url: string,
      options: {
        method?: string;
        headers?: Record<string, string>;
        body?: string;
        useAuth?: boolean;
      } = {}
    ) => {
      setLoading(true);
      setError(null);
      setResponse(null);

      try {
        const headers: Record<string, string> = {
          ...options.headers,
        };

        if (options.useAuth !== false) {
          const token = loadToken();
          if (token) {
            headers['Authorization'] = `Bearer ${token}`;
          }
        }

        const fullUrl = baseUrl ? `${baseUrl}${url}` : url;
        const res = await fetch(fullUrl, {
          method: options.method || 'GET',
          headers,
          body: options.body,
        });

        const responseHeaders: Record<string, string> = {};
        res.headers.forEach((value, key) => {
          responseHeaders[key] = value;
        });

        const raw = await res.text();
        let body: unknown;
        try {
          body = JSON.parse(raw);
        } catch {
          body = raw;
        }

        const apiResponse: ApiResponse = {
          status: res.status,
          statusText: res.statusText,
          headers: responseHeaders,
          body,
          raw,
        };

        setResponse(apiResponse);
        setLoading(false);
        return apiResponse;
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        setError(message);
        setLoading(false);
        throw err;
      }
    },
    [baseUrl]
  );

  return { execute, loading, error, response };
}
