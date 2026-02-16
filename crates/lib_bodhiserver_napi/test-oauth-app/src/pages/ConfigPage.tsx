import React from 'react';
import { useSearchParams } from 'react-router-dom';
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui';
import { ConfigForm } from '@/components/ConfigForm';

export function ConfigPage() {
  const [searchParams] = useSearchParams();
  const errorParam = searchParams.get('error');
  const errorDescription = searchParams.get('error_description');

  const initialError = errorParam
    ? `OAuth Error: ${errorParam}${errorDescription ? ' - ' + errorDescription : ''}`
    : null;

  return (
    <div className="w-full max-w-2xl py-8 px-4">
      <Card>
        <CardHeader>
          <CardTitle>OAuth2 Authentication Test</CardTitle>
          <CardDescription>Configure and test OAuth2 authentication flows</CardDescription>
        </CardHeader>
        <CardContent>
          <ConfigForm initialError={initialError} />
        </CardContent>
      </Card>
    </div>
  );
}
