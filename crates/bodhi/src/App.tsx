import { Routes, Route, Navigate } from 'react-router-dom';
import ClientProviders from '@/components/ClientProviders';
import { Toaster } from '@/components/ui/toaster';
import {
  NavigationProvider,
  defaultNavigationItems,
} from '@/hooks/use-navigation';
import { ThemeProvider } from '@/components/ThemeProvider';
import { AppHeader } from '@/components/navigation/AppHeader';

// Import page components
import HomePage from '@/pages/HomePage';
import ChatPage from '@/pages/ChatPage';
import ModelsPage from '@/pages/ModelsPage';
import ModelFilesPage from '@/pages/ModelFilesPage';
import PullPage from '@/pages/PullPage';
import LoginPage from '@/pages/LoginPage';
import SettingsPage from '@/pages/SettingsPage';
import TokensPage from '@/pages/TokensPage';
import UsersPage from '@/pages/UsersPage';
import SetupPage from '@/pages/SetupPage';
import DocsPage from '@/pages/DocsPage';
import NotFoundPage from '@/pages/NotFoundPage';

function App() {
  return (
    <div
      className="min-h-screen bg-background font-sans antialiased"
    >
      <ThemeProvider defaultTheme="system" storageKey="bodhi-ui-theme">
        <ClientProviders>
          <NavigationProvider items={defaultNavigationItems}>
            <div
              className="flex min-h-screen flex-col"
              data-testid="root-layout"
            >
              <AppHeader />
              <main className="flex-1" data-testid="app-main">
                <Routes>
                  <Route path="/" element={<Navigate to="/ui" replace />} />
                  <Route path="/ui" element={<HomePage />} />
                  <Route path="/ui/home" element={<HomePage />} />
                  <Route path="/ui/chat" element={<ChatPage />} />
                  <Route path="/ui/models" element={<ModelsPage />} />
                  <Route path="/ui/models/new" element={<ModelsPage />} />
                  <Route path="/ui/models/edit" element={<ModelsPage />} />
                  <Route path="/ui/modelfiles" element={<ModelFilesPage />} />
                  <Route path="/ui/pull" element={<PullPage />} />
                  <Route path="/ui/login" element={<LoginPage />} />
                  <Route path="/ui/settings" element={<SettingsPage />} />
                  <Route path="/ui/tokens" element={<TokensPage />} />
                  <Route path="/ui/users" element={<UsersPage />} />
                  <Route path="/ui/setup/*" element={<SetupPage />} />
                  <Route path="/docs/*" element={<DocsPage />} />
                  <Route path="*" element={<NotFoundPage />} />
                </Routes>
              </main>
              <Toaster />
            </div>
          </NavigationProvider>
        </ClientProviders>
      </ThemeProvider>
    </div>
  );
}

export default App;
