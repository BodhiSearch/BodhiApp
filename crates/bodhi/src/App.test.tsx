import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { expect, test, vi } from 'vitest';
import App from './App';

// Mock all the page components
vi.mock('@/pages/HomePage', () => ({
  default: () => <div data-testid="home-page">Home Page</div>,
}));

vi.mock('@/pages/ChatPage', () => ({
  default: () => <div data-testid="chat-page">Chat Page</div>,
}));

vi.mock('@/pages/ModelsPage', () => ({
  default: () => <div data-testid="models-page">Models Page</div>,
}));

vi.mock('@/pages/ModelFilesPage', () => ({
  default: () => <div data-testid="modelfiles-page">ModelFiles Page</div>,
}));

vi.mock('@/pages/PullPage', () => ({
  default: () => <div data-testid="pull-page">Pull Page</div>,
}));

vi.mock('@/pages/LoginPage', () => ({
  default: () => <div data-testid="login-page">Login Page</div>,
}));

vi.mock('@/pages/SettingsPage', () => ({
  default: () => <div data-testid="settings-page">Settings Page</div>,
}));

vi.mock('@/pages/TokensPage', () => ({
  default: () => <div data-testid="tokens-page">Tokens Page</div>,
}));

vi.mock('@/pages/UsersPage', () => ({
  default: () => <div data-testid="users-page">Users Page</div>,
}));

vi.mock('@/pages/SetupPage', () => ({
  default: () => <div data-testid="setup-page">Setup Page</div>,
}));

vi.mock('@/pages/DocsPage', () => ({
  default: () => <div data-testid="docs-page">Docs Page</div>,
}));

vi.mock('@/pages/NotFoundPage', () => ({
  default: () => <div data-testid="not-found-page">Not Found Page</div>,
}));

// Mock the AppHeader component
vi.mock('@/components/navigation/AppHeader', () => ({
  AppHeader: () => <div data-testid="app-header">App Header</div>,
}));

test('App redirects root path to /ui', () => {
  render(
    <MemoryRouter initialEntries={['/']}>
      <App />
    </MemoryRouter>
  );

  expect(screen.getByTestId('home-page')).toBeInTheDocument();
});

test('App renders home page for /ui route', () => {
  render(
    <MemoryRouter initialEntries={['/ui']}>
      <App />
    </MemoryRouter>
  );

  expect(screen.getByTestId('home-page')).toBeInTheDocument();
});

test('App renders not found page for unknown routes', () => {
  render(
    <MemoryRouter initialEntries={['/unknown-route']}>
      <App />
    </MemoryRouter>
  );

  expect(screen.getByTestId('not-found-page')).toBeInTheDocument();
});
