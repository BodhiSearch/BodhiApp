'use client';

import { useState } from 'react';
import { useQuery } from 'react-query';
import axios from 'axios';
import AppHeader from '@/components/AppHeader';
import { DataTable, Pagination } from '@/components/DataTable';
import { TableCell } from '@/components/ui/table';
import { ModelFile, ModelFilesResponse, SortState } from '@/types/models';

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

export default function ModelFilesPage() {
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(30);
  const [sort, setSort] = useState<SortState>({
    column: 'filename',
    direction: 'asc',
  });

  const fetchModelFiles = async () => {
    const response = await axios.get<ModelFilesResponse>(`/api/ui/modelfiles`, {
      params: {
        page,
        page_size: pageSize,
        sort: sort.column,
        sort_order: sort.direction,
      },
    });
    return response.data;
  };

  const { data, isLoading, error } = useQuery(
    ['modelFiles', page, pageSize, sort],
    fetchModelFiles,
    { keepPreviousData: true }
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
      <h4 className="font-semibold">Model Parameters:</h4>
      <p>{JSON.stringify(modelFile.model_params)}</p>
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

  if (error) return <div>An error occurred: {(error as Error).message}</div>;

  return (
    <div className="container mx-auto px-4 sm:px-6 lg:px-8">
      <AppHeader />
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
          totalPages={data ? Math.ceil(data.total / data.page_size) : 1}
          onPageChange={setPage}
        />
      </div>
    </div>
  );
}