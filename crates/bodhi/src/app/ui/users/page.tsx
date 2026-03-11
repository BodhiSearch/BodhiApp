'use client';

import React, { useState } from 'react';

import { AlertCircle, Check, Copy } from 'lucide-react';

import AppInitializer from '@/components/AppInitializer';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Skeleton } from '@/components/ui/skeleton';
import { UserManagementTabs } from '@/components/UserManagementTabs';
import { UsersTable } from '@/components/users/UsersTable';
import { useAppInfo } from '@/hooks/useInfo';
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

function InviteLinkSection() {
  const { data: appInfo } = useAppInfo();
  const [copied, setCopied] = useState(false);

  if (appInfo?.deployment !== 'multi_tenant') {
    return null;
  }

  const inviteUrl = `${appInfo.url}/ui/login/?invite=${appInfo.client_id}`;

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(inviteUrl);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      setCopied(false);
    }
  };

  return (
    <div className="flex items-center gap-2">
      <Input readOnly value={inviteUrl} data-testid="invite-url-input" className="w-80 text-sm" />
      <Button
        variant="outline"
        size="icon"
        onClick={handleCopy}
        data-testid="invite-copy-button"
        title="Copy invite link"
      >
        {copied ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
      </Button>
      <span className="text-sm text-muted-foreground">Share this link to invite users to your workspace</span>
    </div>
  );
}

export default function UsersPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true} minRole="manager">
      <div className="container mx-auto p-4" data-testid="users-page">
        <div className="flex justify-between items-center mb-4">
          <UserManagementTabs />
          <InviteLinkSection />
        </div>
        <UsersContent />
      </div>
    </AppInitializer>
  );
}
