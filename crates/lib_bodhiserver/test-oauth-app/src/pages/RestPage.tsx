import React, { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { RestClientSection } from '@/components/RestClientSection';
import { useAuth } from '@/context/AuthContext';
import { loadToken } from '@/lib/storage';

export function RestPage() {
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
    <div data-testid="page-rest" data-test-state="loaded" className="w-full max-w-3xl py-6 px-4 space-y-5">
      <RestClientSection />
    </div>
  );
}
