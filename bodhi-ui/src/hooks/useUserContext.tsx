'use client';

import { createContext, useContext, ReactNode } from 'react';
import { useUser } from '@/hooks/useQuery';
import { UserInfo } from '@/types/models';

interface UserContextType {
  userInfo: UserInfo | null | undefined;
  isLoading: boolean;
  error: Error | null;
}

const UserContext = createContext<UserContextType | undefined>(undefined);

export const UserProvider: React.FC<{ children: ReactNode }> = ({
  children,
}) => {
  const { data: userInfo, isLoading, error } = useUser();

  return (
    <UserContext.Provider value={{ userInfo, isLoading, error }}>
      {children}
    </UserContext.Provider>
  );
};

export function useUserContext() {
  const context = useContext(UserContext);
  if (context === undefined) {
    throw new Error('useUserContext must be used within a UserProvider');
  }
  return context;
}
