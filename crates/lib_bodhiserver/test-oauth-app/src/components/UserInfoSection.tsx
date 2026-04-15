import React, { useState } from 'react';
import { Card, CardHeader, CardTitle, CardContent, Button } from '@/components/ui';
import { loadConfig, loadToken } from '@/lib/storage';

export function UserInfoSection() {
  const [response, setResponse] = useState<unknown>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchUserInfo = async () => {
    setLoading(true);
    setError(null);
    setResponse(null);
    try {
      const config = loadConfig();
      const token = loadToken();
      if (!config) throw new Error('No config found');
      const res = await fetch(`${config.bodhiServerUrl}/bodhi/v1/user`, {
        headers: token ? { Authorization: `Bearer ${token}` } : {},
      });
      const data = await res.json();
      setResponse(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  const testState = loading ? 'loading' : error ? 'error' : response ? 'success' : 'idle';

  return (
    <Card data-testid="section-user-info" data-test-state={testState}>
      <CardHeader>
        <CardTitle>User Info</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <Button data-testid="btn-fetch-user" onClick={fetchUserInfo} disabled={loading} size="sm">
          {loading ? 'Fetching...' : 'Fetch User Info'}
        </Button>
        {error && <p data-testid="user-info-error" className="text-sm text-destructive">{error}</p>}
        {response && (
          <pre data-testid="user-info-response" className="code-block">
            {JSON.stringify(response, null, 2)}
          </pre>
        )}
      </CardContent>
    </Card>
  );
}
