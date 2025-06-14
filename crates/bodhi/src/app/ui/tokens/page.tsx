'use client';

import { TokenDialog } from '@/app/ui/tokens/TokenDialog';
import { TokenForm } from '@/app/ui/tokens/TokenForm';
import AppInitializer from '@/components/AppInitializer';
import { DataTable, Pagination, SortState } from '@/components/DataTable';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Skeleton } from '@/components/ui/skeleton';
import { Switch } from '@/components/ui/switch';
import { TableCell } from '@/components/ui/table';
import { ApiToken, TokenResponse, useListTokens, useUpdateToken } from '@/hooks/useApiTokens';
import { useAppInfo } from '@/hooks/useQuery';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { Shield } from 'lucide-react';
import { useState } from 'react';

const columns = [
  { id: 'name', name: 'Name', sorted: false },
  { id: 'status', name: 'Status', sorted: false },
  {
    id: 'created_at',
    name: 'Created At',
    sorted: true,
    className: 'hidden md:table-cell',
  },
  {
    id: 'updated_at',
    name: 'Updated At',
    sorted: false,
    className: 'hidden md:table-cell',
  },
];

function StatusBadge({ status }: { status: string }) {
  const variant = status === 'active' ? 'default' : 'secondary';
  return <Badge variant={variant}>{status}</Badge>;
}

export function TokenPageContent() {
  const [token, setToken] = useState<TokenResponse | null>(null);
  const { data: appInfo, isLoading: appLoading } = useAppInfo();
  const [page, setPage] = useState(1);
  const [pageSize] = useState(10);
  const [sort] = useState<SortState>({
    column: 'created_at',
    direction: 'desc',
  });
  const { showSuccess, showError } = useToastMessages();

  const { mutate: updateToken } = useUpdateToken({
    onSuccess: (token) => {
      showSuccess('Token Updated', `Token status changed to ${token.status}`);
    },
    onError: (message) => {
      showError('Error', message);
    },
  });

  const { data: tokensData, isLoading: tokensLoading } = useListTokens(page, pageSize, {
    enabled: !appLoading,
  });

  const handleStatusChange = (token: ApiToken, checked: boolean) => {
    updateToken({
      id: token.id,
      name: token.name,
      status: checked ? 'active' : 'inactive',
    });
  };

  const handleTokenCreated = (newToken: TokenResponse) => {
    setToken(newToken);
  };

  const handleDialogClose = () => {
    setToken(null);
  };

  if (appLoading) {
    return (
      <div className="container mx-auto px-4 sm:px-6 lg:px-8 py-6" data-testid="token-page-loading">
        <Card>
          <CardHeader>
            <div className="flex items-center gap-2">
              <Skeleton className="h-5 w-5" />
              <Skeleton className="h-8 w-32" />
            </div>
            <Skeleton className="h-4 w-3/4 mt-2" />
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <Skeleton className="h-10 w-full" />
              <Skeleton className="h-10 w-1/4" />
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  const renderRow = (token: ApiToken) => (
    <>
      <TableCell>{token.name || '-'}</TableCell>
      <TableCell>
        <div className="flex items-center gap-2">
          <Switch
            checked={token.status === 'active'}
            onCheckedChange={(checked) => handleStatusChange(token, checked)}
            aria-label="Toggle token status"
          />
          <StatusBadge status={token.status} />
        </div>
      </TableCell>
      <TableCell className="hidden md:table-cell">{new Date(token.created_at).toLocaleString()}</TableCell>
      <TableCell className="hidden md:table-cell">{new Date(token.updated_at).toLocaleString()}</TableCell>
    </>
  );

  return (
    <div className="container mx-auto px-4 sm:px-6 lg:px-8 py-6">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            API Tokens
          </CardTitle>
          <CardDescription>Generate and manage API tokens for programmatic access to the API</CardDescription>
        </CardHeader>
        <CardContent>
          <Alert>
            <Shield className="h-4 w-4" />
            <AlertDescription>
              API tokens provide full access to the API. Keep them secure. Tokens cannot be viewed again after creation.
            </AlertDescription>
          </Alert>
          <div className="mt-6">
            <TokenForm onTokenCreated={handleTokenCreated} />
          </div>
          <div className="mt-8">
            <DataTable
              data={tokensData?.data || []}
              columns={columns}
              loading={tokensLoading}
              renderRow={renderRow}
              getItemId={(item) => item.id}
              sort={sort}
              onSortChange={() => {}}
            />
            {tokensData && (
              <div className="mt-6 mb-4">
                <Pagination
                  page={page}
                  totalPages={
                    tokensData ? Math.ceil((tokensData.total as number) / (tokensData.page_size as number)) : 1
                  }
                  onPageChange={setPage}
                />
              </div>
            )}
          </div>
        </CardContent>
      </Card>
      {token && <TokenDialog token={token} onClose={handleDialogClose} open={!!token} />}
    </div>
  );
}

export default function TokenPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <TokenPageContent />
    </AppInitializer>
  );
}
