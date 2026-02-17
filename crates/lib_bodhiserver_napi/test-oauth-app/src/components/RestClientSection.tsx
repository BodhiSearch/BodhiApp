import React, { useState } from 'react';
import { Card, CardHeader, CardTitle, CardContent, Button, Input, Label, Textarea, Select, Checkbox } from '@/components/ui';
import { loadToken, loadConfig } from '@/lib/storage';
import { useAuth } from '@/context/AuthContext';

interface RestResponse {
  status: number;
  statusText: string;
  body: unknown;
  raw: string;
}

export function RestClientSection() {
  // Get bodhiServerUrl from context, with localStorage fallback via lazy useState
  const { config: contextConfig } = useAuth();
  const [fallbackConfig] = useState(() => loadConfig());
  const bodhiServerUrl = contextConfig?.bodhiServerUrl || fallbackConfig?.bodhiServerUrl || '';

  const [method, setMethod] = useState('GET');
  const [url, setUrl] = useState('');
  const [headers, setHeaders] = useState('');
  const [body, setBody] = useState('');
  const [includeAuth, setIncludeAuth] = useState(true);
  const [useJson, setUseJson] = useState(true);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [response, setResponse] = useState<RestResponse | null>(null);

  const parseHeaders = (headerText: string): Record<string, string> => {
    const result: Record<string, string> = {};
    if (!headerText.trim()) return result;
    const lines = headerText.split('\n');
    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed) continue;
      const colonIndex = trimmed.indexOf(':');
      if (colonIndex === -1) continue;
      const key = trimmed.substring(0, colonIndex).trim();
      const value = trimmed.substring(colonIndex + 1).trim();
      if (key) {
        result[key] = value;
      }
    }
    return result;
  };

  const sendRequest = async () => {
    setLoading(true);
    setError(null);
    setResponse(null);
    try {
      if (!url) throw new Error('URL is required');

      // Validate path-only URL
      if (url.startsWith('http')) {
        throw new Error('URL must be a path starting with / (e.g., /bodhi/v1/user)');
      }
      if (!url.startsWith('/')) {
        throw new Error('URL must start with / (e.g., /bodhi/v1/user)');
      }

      const fetchHeaders: Record<string, string> = parseHeaders(headers);

      // Add Content-Type if useJson is checked
      if (useJson) {
        fetchHeaders['Content-Type'] = 'application/json';
      }

      if (includeAuth) {
        const token = loadToken();
        if (token) {
          fetchHeaders['Authorization'] = `Bearer ${token}`;
        }
      }

      const fetchOptions: RequestInit = {
        method,
        headers: fetchHeaders,
      };

      if (method !== 'GET' && body.trim()) {
        fetchOptions.body = body;
      }

      // Construct full URL with server prefix
      const fullUrl = `${bodhiServerUrl}${url}`;
      const res = await fetch(fullUrl, fetchOptions);
      const raw = await res.text();
      let parsedBody: unknown;
      try {
        parsedBody = JSON.parse(raw);
      } catch {
        parsedBody = raw;
      }

      setResponse({
        status: res.status,
        statusText: res.statusText,
        body: parsedBody,
        raw,
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  const testState = loading ? 'loading' : error ? 'error' : response ? 'success' : 'idle';

  return (
    <Card data-testid="section-rest-client" data-test-state={testState}>
      <CardHeader>
        <CardTitle>REST Client</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-1">
          <Label>Server URL</Label>
          <p data-testid="rest-server-url" className="text-sm font-mono bg-muted px-2 py-1 rounded">
            {bodhiServerUrl || '(not configured)'}
          </p>
        </div>

        <div className="space-y-1">
          <Label>Request</Label>
          <div className="flex gap-2 items-center">
            <div className="w-24">
              <Select
                data-testid="select-rest-method"
                value={method}
                onChange={(e) => setMethod(e.target.value)}
              >
                <option value="GET">GET</option>
                <option value="POST">POST</option>
                <option value="PUT">PUT</option>
                <option value="DELETE">DELETE</option>
              </Select>
            </div>
            <span className="text-sm text-muted-foreground font-mono">{bodhiServerUrl}</span>
            <div className="flex-1">
              <Input
                data-testid="input-rest-url"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                placeholder="/bodhi/v1/user"
              />
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <Checkbox
            id="rest-auth"
            data-testid="toggle-rest-auth"
            checked={includeAuth}
            onChange={(e) => setIncludeAuth(e.target.checked)}
          />
          <Label htmlFor="rest-auth">Use OAuth Token</Label>
        </div>

        <div className="flex items-center gap-2">
          <Checkbox
            id="rest-json"
            data-testid="toggle-rest-json"
            checked={useJson}
            onChange={(e) => setUseJson(e.target.checked)}
          />
          <Label htmlFor="rest-json">Content-Type: application/json</Label>
        </div>

        <div className="space-y-1">
          <Label htmlFor="rest-headers">Headers (one per line, key: value)</Label>
          <Textarea
            id="rest-headers"
            data-testid="input-rest-headers"
            value={headers}
            onChange={(e) => setHeaders(e.target.value)}
            placeholder="Custom-Header: value"
            rows={2}
          />
        </div>

        <div className="space-y-1">
          <Label htmlFor="rest-body">Body (JSON)</Label>
          <Textarea
            id="rest-body"
            data-testid="input-rest-body"
            value={body}
            onChange={(e) => setBody(e.target.value)}
            placeholder='{"key": "value"}'
            rows={3}
          />
        </div>

        <Button data-testid="btn-rest-send" onClick={sendRequest} disabled={loading} size="sm">
          {loading ? 'Sending...' : 'Send'}
        </Button>

        {error && <p data-testid="rest-error" className="text-sm text-destructive">{error}</p>}

        {response && (
          <div className="space-y-2">
            <p data-testid="rest-response-status" className="text-sm font-medium">
              Status: {response.status} {response.statusText}
            </p>
            <pre data-testid="rest-response" className="code-block">
              {typeof response.body === 'string' ? response.body : JSON.stringify(response.body, null, 2)}
            </pre>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
