import React from 'react';
import { Badge } from '@/components/ui';

interface ScopeDisplayProps {
  resourceScope: string | null;
  accessRequestScope: string | null;
}

export function ScopeDisplay({ resourceScope, accessRequestScope }: ScopeDisplayProps) {
  const resourceScopes = resourceScope ? resourceScope.split(' ').filter(Boolean) : [];
  const accessScopes = accessRequestScope ? accessRequestScope.split(' ').filter(Boolean) : [];

  return (
    <div
      className="rounded-md border border-success/30 bg-success/5 p-4 space-y-2"
      data-test-resource-scope={resourceScope || ''}
      data-test-access-request-scope={accessRequestScope || ''}
    >
      <div className="flex items-center gap-2 flex-wrap">
        <span className="text-sm font-medium">Resource:</span>
        {resourceScopes.length > 0 ? (
          resourceScopes.map((s) => (
            <Badge key={s} variant="success">{s}</Badge>
          ))
        ) : (
          <span className="text-sm italic text-muted-foreground">none</span>
        )}
      </div>
      <div className="flex items-center gap-2 flex-wrap">
        <span className="text-sm font-medium">Access Request:</span>
        {accessScopes.length > 0 ? (
          accessScopes.map((s) => (
            <Badge key={s} variant="success">{s}</Badge>
          ))
        ) : (
          <span className="text-sm italic text-muted-foreground">none</span>
        )}
      </div>
    </div>
  );
}
