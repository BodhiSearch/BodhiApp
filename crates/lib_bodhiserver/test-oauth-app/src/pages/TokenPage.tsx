import React, { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui';
import { TokenDisplay } from '@/components/TokenDisplay';
import { UserInfoSection } from '@/components/UserInfoSection';
import { useAuth } from '@/context/AuthContext';
import { loadToken } from '@/lib/storage';

export function TokenPage() {
  const navigate = useNavigate();
  const { token } = useAuth();

  const effectiveToken = token || loadToken();

  useEffect(() => {
    if (!effectiveToken) {
      navigate('/', { replace: true });
    }
  }, [effectiveToken, navigate]);

  if (!effectiveToken) {
    return null;
  }

  return (
    <div data-testid="page-dashboard" data-test-state="loaded" className="w-full max-w-3xl py-6 px-4 space-y-5">
      <Card data-testid="section-token">
        <CardHeader>
          <CardTitle>Access Token</CardTitle>
          <CardDescription>Your OAuth2 access token and decoded JWT claims</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <TokenDisplay token={effectiveToken} />
        </CardContent>
      </Card>

      <UserInfoSection />
    </div>
  );
}
