'use client';

import { useState } from 'react';
import { useDownloads } from '@/hooks/useQuery';
import { DataTable, Pagination } from '@/components/DataTable';
import { TableCell } from '@/components/ui/table';
import { DownloadRequest } from '@/types/api';
import { SortState } from '@/types/models';
import AppInitializer from '@/components/AppInitializer';
import { Badge } from '@/components/ui/badge';
import { PullForm } from '@/components/PullForm';

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

  const renderExpandedRow = (download: DownloadRequest) => (
    <div className="p-4 bg-gray-50">
      {download.status === 'error' && download.error && (
        <>
          <h4 className="font-semibold">Error:</h4>
          <p className="text-red-600">{download.error}</p>
        </>
      )}
    </div>
  );

  if (error) {
    return <div>Error loading downloads</div>;
  }

  return (
    <div className="container mx-auto px-4 sm:px-6 lg:px-8">
      <div className="mb-8">
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
      <div className="mt-4 flex flex-col sm:flex-row justify-between items-center">
        <div className="mb-2 sm:mb-0">
          Displaying {data?.data.length || 0} items of {data?.total || 0}
        </div>
        <Pagination
          page={page}
          totalPages={data ? Math.ceil(data.total / data.page_size) : 1}
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
