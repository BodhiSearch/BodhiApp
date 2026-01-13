'use client';

import { useState } from 'react';

import { ApiToken } from '@bodhiapp/ts-client';
import { Shield } from 'lucide-react';

import { CreateTokenDialog } from '@/app/ui/tokens/CreateTokenDialog';
import AppInitializer from '@/components/AppInitializer';
import { DataTable, Pagination, SortState } from '@/components/DataTable';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Skeleton } from '@/components/ui/skeleton';
import { Switch } from '@/components/ui/switch';
import { TableCell } from '@/components/ui/table';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { useListTokens, useUpdateToken } from '@/hooks/useApiTokens';
import { useAppInfo } from '@/hooks/useInfo';
import { useQueryClient } from '@/hooks/useQuery';

const columns = [
  { id: 'name', name: 'Name', sorted: false },
  { id: 'scope', name: 'Scope', sorted: false },
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
  const [showDialog, setShowDialog] = useState(false);
  const { isLoading: appLoading } = useAppInfo();
  const [page, setPage] = useState(1);
  const [pageSize] = useState(10);
  const [sort] = useState<SortState>({
    column: 'created_at',
    direction: 'desc',
  });
  const { showSuccess, showError } = useToastMessages();
  const _queryClient = useQueryClient();

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

  const handleDialogClose = () => {
    setShowDialog(false);
  };

  if (appLoading) {
    return (
      <div className="container mx-auto p-4" data-testid="tokens-page" data-pagestatus="loading">
        <div className="space-y-4">
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-10 w-1/4" />
        </div>
      </div>
    );
  }

  const renderRow = (token: ApiToken) => (
    <>
      <TableCell data-testid={`token-name-${token.id}`}>{token.name || '-'}</TableCell>
      <TableCell data-testid={`token-scope-${token.id}`}>{token.scopes}</TableCell>
      <TableCell>
        <div className="flex items-center gap-2">
          <Switch
            checked={token.status === 'active'}
            onCheckedChange={(checked) => handleStatusChange(token, checked)}
            aria-label="Toggle token status"
            data-testid={`token-status-switch-${token.id}`}
          />
          <StatusBadge status={token.status} />
        </div>
      </TableCell>
      <TableCell className="hidden md:table-cell">{new Date(token.created_at).toLocaleString()}</TableCell>
      <TableCell className="hidden md:table-cell">{new Date(token.updated_at).toLocaleString()}</TableCell>
    </>
  );

  return (
    <div
      className="container mx-auto p-4"
      data-testid="tokens-page"
      data-pagestatus={tokensLoading ? 'loading' : 'ready'}
    >
      <div className="flex justify-end gap-2 mb-4">
        <Button onClick={() => setShowDialog(true)} data-testid="new-token-button">
          <Shield className="h-4 w-4 mr-2" />
          New API Token
        </Button>
      </div>
      <Alert>
        <Shield className="h-4 w-4" />
        <AlertDescription>
          API tokens provide full access to the API. Keep them secure. Tokens cannot be viewed again after creation.
        </AlertDescription>
      </Alert>
      <div className="mt-6" data-testid="tokens-table-container">
        <DataTable
          data={tokensData?.data || []}
          columns={columns}
          loading={tokensLoading}
          renderRow={renderRow}
          getItemId={(item) => item.id}
          sort={sort}
          onSortChange={() => {}}
          data-testid="tokens-table"
        />
        {tokensData && (
          <div className="mt-6 mb-4">
            <Pagination
              page={page}
              totalPages={tokensData ? Math.ceil((tokensData.total as number) / (tokensData.page_size as number)) : 1}
              onPageChange={setPage}
            />
          </div>
        )}
      </div>
      <CreateTokenDialog open={showDialog} onClose={handleDialogClose} />
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
