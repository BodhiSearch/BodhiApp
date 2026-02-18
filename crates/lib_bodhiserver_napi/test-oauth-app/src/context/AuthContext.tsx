import React, { createContext, useContext, useState, useEffect } from 'react';
import { loadConfig, loadToken, saveConfig as storeConfig, saveToken as storeToken } from '@/lib/storage';

export interface OAuthConfig {
  bodhiServerUrl: string;
  authServerUrl: string;
  realm: string;
  clientId: string;
  isConfidential: boolean;
  clientSecret: string;
  redirectUri: string;
  scope: string;
  requested: string;
  codeVerifier?: string;
  state?: string;
  approvedScopes?: string[];
  accessRequestId?: string;
}

interface AuthContextValue {
  token: string | null;
  config: OAuthConfig | null;
  setToken: (token: string | null) => void;
  setConfig: (config: OAuthConfig | null) => void;
}

const AuthContext = createContext<AuthContextValue | undefined>(undefined);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [token, setTokenState] = useState<string | null>(() => loadToken());
  const [config, setConfigState] = useState<OAuthConfig | null>(() => loadConfig());

  const setToken = (newToken: string | null) => {
    setTokenState(newToken);
    if (newToken) {
      storeToken(newToken);
    }
  };

  const setConfig = (newConfig: OAuthConfig | null) => {
    setConfigState(newConfig);
    if (newConfig) {
      storeConfig(newConfig);
    }
  };

  return (
    <AuthContext.Provider value={{ token, config, setToken, setConfig }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthContextValue {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}
