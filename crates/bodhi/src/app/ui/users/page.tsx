'use client';

import React, { useState } from 'react';

import { AlertCircle } from 'lucide-react';

import AppInitializer from '@/components/AppInitializer';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Card, CardContent, CardHeader } from '@/components/ui/card';
import { Skeleton } from '@/components/ui/skeleton';
import { UserManagementTabs } from '@/components/UserManagementTabs';
import { UsersTable } from '@/components/users/UsersTable';
import { useAllUsers, useAuthenticatedUser } from '@/hooks/useUsers';

function UsersContent() {
  const [page, setPage] = useState(1);
  const [pageSize] = useState(10);

  const { data: currentUserInfo, isLoading: isLoadingUser } = useAuthenticatedUser();
  const { data: usersData, isLoading: isLoadingUsers, error } = useAllUsers(page, pageSize);

  // Get current user's role and username for filtering
  const currentUserRole = typeof currentUserInfo?.role === 'string' ? currentUserInfo.role : '';
  const currentUsername = currentUserInfo?.username || '';

  // Show loading state if either users or current user info is loading
  if (isLoadingUsers || isLoadingUser) {
    return (
      <Card>
        <CardHeader>
          <Skeleton className="h-6 w-32" />
          <Skeleton className="h-4 w-48" />
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {Array.from({ length: 5 }).map((_, i) => (
              <Skeleton key={i} className="h-12 w-full" />
            ))}
          </div>
        </CardContent>
      </Card>
    );
  }

  if (error) {
    const errorMessage = error?.response?.data?.error?.message || 'Failed to load users. Please try again later.';
    return (
      <Alert variant="destructive">
        <AlertCircle className="h-4 w-4" />
        <AlertDescription>{errorMessage}</AlertDescription>
      </Alert>
    );
  }

  const users = usersData?.users || [];
  const total = usersData?.total_users || 0;

  return (
    <UsersTable
      users={users}
      total={total}
      page={page}
      pageSize={pageSize}
      currentUserRole={currentUserRole}
      currentUsername={currentUsername}
      currentUserInfo={currentUserInfo}
      isLoading={isLoadingUsers}
      onPageChange={setPage}
    />
  );
}

export default function UsersPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true} minRole="manager">
      <div className="container mx-auto p-4" data-testid="users-page">
        <UserManagementTabs />
        <UsersContent />
      </div>
    </AppInitializer>
  );
}
