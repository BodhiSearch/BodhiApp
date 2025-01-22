'use client';

import { useState } from 'react';
import { DataTable, Pagination } from '@/components/DataTable';
import { TableCell } from '@/components/ui/table';
import { ModelFile, SortState } from '@/types/models';
import { useModelFiles } from '@/hooks/useQuery';
import AppInitializer from '@/components/AppInitializer';

// Helper function to convert bytes to GB
const bytesToGB = (bytes: number | undefined): string => {
  if (bytes === undefined) return '';
  const gb = bytes / (1024 * 1024 * 1024);
  return gb.toFixed(2) + ' GB';
};

const columns = [
  { id: 'repo', name: 'Repo', sorted: true },
  { id: 'filename', name: 'Filename', sorted: true },
  { id: 'size', name: 'Size (GB)', sorted: true },
  { id: 'updated_at', name: 'Updated At', sorted: true },
  { id: 'snapshot', name: 'Snapshot', sorted: true },
];

function ModelFilesContent() {
  const [page, setPage] = useState(1);
  const [pageSize] = useState(30);
  const [sort, setSort] = useState<SortState>({
    column: 'filename',
    direction: 'asc',
  });

  const { data, isLoading, error } = useModelFiles(
    page,
    pageSize,
    sort.column,
    sort.direction
  );

  const toggleSort = (column: string) => {
    setSort((prevSort) => ({
      column,
      direction:
        prevSort.column === column && prevSort.direction === 'asc'
          ? 'desc'
          : 'asc',
    }));
    setPage(1);
  };

  const getItemId = (modelFile: ModelFile) =>
    `${modelFile.repo}${modelFile.filename}${modelFile.snapshot}`;

  const renderRow = (modelFile: ModelFile) => (
    <>
      <TableCell>{modelFile.repo}</TableCell>
      <TableCell>{modelFile.filename}</TableCell>
      <TableCell>{bytesToGB(modelFile.size)}</TableCell>
      <TableCell>
        {modelFile.updated_at
          ? new Date(modelFile.updated_at).toLocaleString()
          : ''}
      </TableCell>
      <TableCell>{modelFile.snapshot.slice(0, 6)}</TableCell>
    </>
  );

  const renderExpandedRow = (modelFile: ModelFile) => (
    <div className="p-4 bg-gray-50">
      <h4 className="font-semibold mt-2">Full Snapshot:</h4>
      <p>{modelFile.snapshot}</p>
      {modelFile.size !== undefined && (
        <>
          <h4 className="font-semibold mt-2">Exact Size:</h4>
          <p>{modelFile.size.toLocaleString()} bytes</p>
        </>
      )}
    </div>
  );
  if (error) {
    const errorMessage =
      error.response?.data?.error?.message ||
      error.message ||
      'An unexpected error occurred. Please try again.';
    return <div>An error occurred: {errorMessage}</div>;
  }

  return (
    <div className="container mx-auto px-4 sm:px-6 lg:px-8">
      <DataTable
        data={data?.data || []}
        columns={columns}
        loading={isLoading}
        sort={sort}
        onSortChange={toggleSort}
        renderRow={renderRow}
        renderExpandedRow={renderExpandedRow}
        getItemId={getItemId}
      />
      <div className="mt-4 flex flex-col sm:flex-row justify-between items-center">
        <div className="mb-2 sm:mb-0">
          Displaying {data?.data.length || 0} items of {data?.total || 0}
        </div>
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

export default function ModelFilesPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <ModelFilesContent />
    </AppInitializer>
  );
}
