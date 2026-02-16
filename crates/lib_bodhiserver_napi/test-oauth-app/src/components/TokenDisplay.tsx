import React from 'react';

interface TokenDisplayProps {
  token: string;
}

function decodeJwtPart(part: string): string {
  try {
    const padded = part.replace(/-/g, '+').replace(/_/g, '/');
    const decoded = atob(padded);
    const parsed = JSON.parse(decoded);
    return JSON.stringify(parsed, null, 2);
  } catch {
    return 'Failed to decode';
  }
}

export function TokenDisplay({ token }: TokenDisplayProps) {
  const parts = token.split('.');
  const header = parts[0] ? decodeJwtPart(parts[0]) : '';
  const payload = parts[1] ? decodeJwtPart(parts[1]) : '';

  return (
    <div className="space-y-4">
      <div>
        <h4 className="text-sm font-medium mb-2">Raw Access Token</h4>
        <div
          data-testid="access-token"
          className="code-block break-all max-h-[200px]"
        >
          {token}
        </div>
      </div>

      <div>
        <h4 className="text-sm font-medium mb-2">JWT Header</h4>
        <pre className="code-block">
          {header}
        </pre>
      </div>

      <div>
        <h4 className="text-sm font-medium mb-2">JWT Payload</h4>
        <pre className="code-block">
          {payload}
        </pre>
      </div>
    </div>
  );
}
