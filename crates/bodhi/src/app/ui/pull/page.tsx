'use client';

import { useState } from 'react';
import { useDownloads } from '@/hooks/useQuery';
import { DataTable, Pagination } from '@/components/DataTable';
import { TableCell } from '@/components/ui/table';
import { DownloadRequest } from '@/types/api';
import { SortState } from '@/types/models';
import AppInitializer from '@/components/AppInitializer';
import { Badge } from '@/components/ui/badge';
import { PullForm } from '@/app/ui/pull/PullForm';
import { useLocalStorage } from '@/hooks/useLocalStorage';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { X } from 'lucide-react';

const columns = [
  { id: 'repo', name: 'Repo', sorted: true },
  { id: 'filename', name: 'Filename', sorted: true },
  { id: 'status', name: 'Status', sorted: true },
  { id: 'updated_at', name: 'Updated At', sorted: true },
];

type BadgeVariant = 'secondary' | 'destructive' | 'default';

function StatusBadge({ status }: { status: DownloadRequest['status'] }) {
  const variants: Record<DownloadRequest['status'], BadgeVariant> = {
    pending: 'secondary',
    completed: 'default',
    error: 'destructive',
  };

  return <Badge variant={variants[status]}>{status}</Badge>;
}

function PullPageContent() {
  const [hasDismissedBanner, setHasDismissedBanner] = useLocalStorage(
    'pull-banner-dismissed',
    false
  );
  const [page, setPage] = useState(1);
  const [pageSize] = useState(30);
  const [sort, setSort] = useState<SortState>({
    column: 'updated_at',
    direction: 'desc',
  });

  const { data, isLoading, error } = useDownloads(page, pageSize);

  const toggleSort = (column: string) => {
    setSort((prevSort) => ({
      column,
      direction:
        prevSort.column === column && prevSort.direction === 'asc'
          ? 'desc'
          : 'asc',
    }));
    setPage(1); // Reset to first page when sorting
  };

  const renderRow = (download: DownloadRequest) => (
    <>
      <TableCell>{download.repo}</TableCell>
      <TableCell>{download.filename}</TableCell>
      <TableCell>
        <StatusBadge status={download.status} />
      </TableCell>
      <TableCell>{new Date(download.updated_at).toLocaleString()}</TableCell>
    </>
  );

  const renderExpandedRow = (download: DownloadRequest) => {
    if (download.status === 'error' && download.error) {
      return (
        <div className="bg-muted p-4">
          <h4 className="font-semibold">Error:</h4>
          <p className="text-destructive">{download.error}</p>
        </div>
      );
    }
    return undefined;
  };

  if (error) {
    return <div className="text-destructive">Error loading downloads</div>;
  }

  return (
    <div className="container mx-auto space-y-8 px-4 py-8 sm:px-6 lg:px-8">
      {!hasDismissedBanner && (
        <Alert className="mb-4">
          <AlertDescription className="flex items-center justify-between gap-4">
            <span>
              Welcome to Pull! Here you can download model files from Hugging
              Face to your local storage, and monitor the status of your
              downloads.
            </span>
            <Button
              variant="ghost"
              size="sm"
              className="h-8 w-8 p-0 shrink-0"
              onClick={() => setHasDismissedBanner(true)}
              title="Dismiss"
            >
              <X className="h-4 w-4" />
            </Button>
          </AlertDescription>
        </Alert>
      )}

      <div>
        <PullForm />
      </div>
      <DataTable
        data={data?.data || []}
        columns={columns}
        loading={isLoading}
        sort={sort}
        onSortChange={toggleSort}
        renderRow={renderRow}
        renderExpandedRow={renderExpandedRow}
        getItemId={(item) => item.id}
      />
      <div className="mt-6 mb-4">
        <Pagination
          page={page}
          totalPages={
            data
              ? Math.ceil((data.total as number) / (data.page_size as number))
              : 1
          }
          onPageChange={setPage}
        />
      </div>
    </div>
  );
}

export default function PullPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <PullPageContent />
    </AppInitializer>
  );
}
