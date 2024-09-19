import React from 'react';
import { render, screen } from '@testing-library/react';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useUserContext, UserProvider } from './useUserContext';
import { QueryClient, QueryClientProvider } from 'react-query';

// Mock the useUser hook
const useUserMock = vi.fn();
vi.mock('@/hooks/useQuery', () => ({
  useUser: () => useUserMock(),
}));

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      <UserProvider>{children}</UserProvider>
    </QueryClientProvider>
  );
};

// Simple component that uses the useUserContext hook
const UserInfo: React.FC = () => {
  const { userInfo, isLoading, error } = useUserContext();

  if (isLoading) return <div>Loading...</div>;
  if (error) return <div>Error: {error.message}</div>;
  if (!userInfo) return <div>No user info available</div>;

  return (
    <div>
      <h1>User Info</h1>
      <p>Email: {userInfo.email}</p>
      <p>Logged in: {userInfo.logged_in ? 'Yes' : 'No'}</p>
    </div>
  );
};

describe('useUserContext', () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it('renders loading state', () => {
    useUserMock.mockReturnValue({ isLoading: true, data: undefined, error: null });

    render(<UserInfo />, { wrapper: createWrapper() });
    expect(screen.getByText('Loading...')).toBeInTheDocument();
    expect(useUserMock).toHaveBeenCalled();
  });

  it('renders error state', () => {
    useUserMock.mockReturnValue({ isLoading: false, data: undefined, error: new Error('Test error') });

    render(<UserInfo />, { wrapper: createWrapper() });
    expect(screen.getByText('Error: Test error')).toBeInTheDocument();
    expect(useUserMock).toHaveBeenCalled();
  });

  it('renders no user info state', () => {
    useUserMock.mockReturnValue({ isLoading: false, data: null, error: null });

    render(<UserInfo />, { wrapper: createWrapper() });
    expect(screen.getByText('No user info available')).toBeInTheDocument();
    expect(useUserMock).toHaveBeenCalled();
  });

  it('renders user info when available', () => {
    useUserMock.mockReturnValue({
      isLoading: false,
      data: { email: 'test@example.com', logged_in: true },
      error: null,
    });

    render(<UserInfo />, { wrapper: createWrapper() });
    expect(screen.getByText('User Info')).toBeInTheDocument();
    expect(screen.getByText('Email: test@example.com')).toBeInTheDocument();
    expect(screen.getByText('Logged in: Yes')).toBeInTheDocument();
    expect(useUserMock).toHaveBeenCalled();
  });

  it('throws error when used outside of UserProvider', () => {
    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => { });
    expect(() => render(<UserInfo />)).toThrow('useUserContext must be used within a UserProvider');
    consoleErrorSpy.mockRestore();
  });
});
