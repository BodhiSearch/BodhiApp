'use client';

import { PullForm } from '@/app/ui/pull/PullForm';
import AppInitializer from '@/components/AppInitializer';
import { DataTable, Pagination } from '@/components/DataTable';
import { Badge } from '@/components/ui/badge';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { TableCell } from '@/components/ui/table';
import { UserOnboarding } from '@/components/UserOnboarding';
import { useDownloads } from '@/hooks/useQuery';
import { DownloadRequest } from '@/types/api';
import { SortState } from '@/types/models';
import { useEffect, useState } from 'react';

const columns = [
  { id: 'repo', name: 'Repo', sorted: true },
  { id: 'filename', name: 'Filename', sorted: true },
  { id: 'status', name: 'Status', sorted: true },
  { id: 'progress', name: 'Progress', sorted: false },
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

function ProgressDisplay({ download }: { download: DownloadRequest }) {
  if (download.status !== 'pending') {
    return <span className="text-muted-foreground">-</span>;
  }

  // Compute progress percentage from bytes
  const computeProgress = (download: DownloadRequest) => {
    if (!download.total_bytes || download.total_bytes === 0) return 0;
    return (download.downloaded_bytes / download.total_bytes) * 100;
  };

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  };

  return (
    <div className="space-y-2">
      <div className="flex items-center space-x-2">
        <div className="w-full bg-muted rounded-full h-2">
          <div
            className="bg-primary h-2 rounded-full transition-all duration-300"
            style={{ width: `${Math.min(computeProgress(download), 100)}%` }}
          />
        </div>
        <span className="text-sm font-medium min-w-[3rem]">{computeProgress(download).toFixed(1)}%</span>
      </div>
      {download.total_bytes && (
        <div className="text-xs text-muted-foreground">
          {formatBytes(download.downloaded_bytes)} / {formatBytes(download.total_bytes)}
        </div>
      )}
    </div>
  );
}

function PullPageContent() {
  const [page, setPage] = useState(1);
  const [pageSize] = useState(30);
  const [sort, setSort] = useState<SortState>({
    column: 'updated_at',
    direction: 'desc',
  });
  const [enablePolling, setEnablePolling] = useState(false);
  const { data, isLoading, error } = useDownloads(page, pageSize, { enablePolling });

  // Update polling state based on pending downloads
  useEffect(() => {
    const hasPendingDownloads = data?.data.some((download) => download.status === 'pending') ?? false;
    setEnablePolling(hasPendingDownloads);
  }, [data]);

  const toggleSort = (column: string) => {
    setSort((prevSort) => ({
      column,
      direction: prevSort.column === column && prevSort.direction === 'asc' ? 'desc' : 'asc',
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
      <TableCell>
        <ProgressDisplay download={download} />
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
    return <ErrorPage message="Error loading downloads" />;
  }

  return (
    <div className="container mx-auto space-y-8 px-4 py-8 sm:px-6 lg:px-8">
      <UserOnboarding storageKey="pull-banner-dismissed">
        Welcome to Download Models! Here you can download model files from Huggingface to your local storage, and
        monitor the status of your downloads.
      </UserOnboarding>

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
          totalPages={data ? Math.ceil((data.total as number) / (data.page_size as number)) : 1}
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
