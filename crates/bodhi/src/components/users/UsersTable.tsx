'use client';

import React from 'react';

import { UserInfo } from '@bodhiapp/ts-client';
import { Users } from 'lucide-react';

import { DataTable, Pagination } from '@/components/DataTable';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { AuthenticatedUser } from '@/hooks/useUsers';
import { SortState } from '@/types/models';

import { UserRow } from './UserRow';

interface UsersTableProps {
  users: UserInfo[];
  total: number;
  page: number;
  pageSize: number;
  currentUserRole: string;
  currentUsername: string;
  currentUserInfo?: AuthenticatedUser;
  isLoading: boolean;
  onPageChange: (page: number) => void;
}

export function UsersTable({
  users,
  total,
  page,
  pageSize,
  currentUserRole,
  currentUsername,
  currentUserInfo,
  isLoading,
  onPageChange,
}: UsersTableProps) {
  // Dummy sort values - no actual sorting functionality
  const dummySort: SortState = { column: '', direction: 'asc' };
  const noOpSortChange = () => {};
  const getItemId = (user: UserInfo) => user.username;

  const columns = [
    { id: 'username', name: 'Username', sorted: false },
    { id: 'role', name: 'Role', sorted: false },
    { id: 'actions', name: 'Actions', sorted: false },
  ];

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Users className="h-5 w-5" />
            All Users
          </CardTitle>
          <CardDescription>Manage user access and roles</CardDescription>
        </CardHeader>
        <CardContent>
          {users.length === 0 ? (
            <div className="text-center py-8">
              <Users className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
              <h3 className="text-lg font-semibold mb-2">No Users</h3>
              <p className="text-muted-foreground">No users found</p>
            </div>
          ) : (
            <>
              <DataTable
                columns={columns}
                data={users}
                renderRow={(user) => (
                  <UserRow
                    user={user}
                    currentUserRole={currentUserRole}
                    currentUsername={currentUsername}
                    currentUserInfo={currentUserInfo}
                  />
                )}
                loading={isLoading}
                sort={dummySort}
                onSortChange={noOpSortChange}
                getItemId={getItemId}
              />
              {total > pageSize && (
                <div className="mt-4">
                  <Pagination page={page} totalPages={Math.ceil(total / pageSize)} onPageChange={onPageChange} />
                </div>
              )}
            </>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
