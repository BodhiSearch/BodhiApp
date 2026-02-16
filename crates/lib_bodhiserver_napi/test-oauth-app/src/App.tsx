import React from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { AuthProvider } from '@/context/AuthContext';
import { AppLayout } from '@/components/AppLayout';
import { ConfigPage } from '@/pages/ConfigPage';
import { AccessCallbackPage } from '@/pages/AccessCallbackPage';
import { OAuthCallbackPage } from '@/pages/OAuthCallbackPage';
import { TokenPage } from '@/pages/TokenPage';
import { ChatPage } from '@/pages/ChatPage';
import { RestPage } from '@/pages/RestPage';

export default function App() {
  return (
    <AuthProvider>
      <BrowserRouter>
        <AppLayout>
          <Routes>
            <Route path="/" element={<ConfigPage />} />
            <Route path="/access-callback" element={<AccessCallbackPage />} />
            <Route path="/callback" element={<OAuthCallbackPage />} />
            <Route path="/dashboard" element={<TokenPage />} />
            <Route path="/chat" element={<ChatPage />} />
            <Route path="/rest" element={<RestPage />} />
          </Routes>
        </AppLayout>
      </BrowserRouter>
    </AuthProvider>
  );
}
