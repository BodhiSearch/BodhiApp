'use client';

import { useState } from 'react';
import { AxiosError } from 'axios';
import { useRouter } from 'next/navigation';
import { DataTable, Pagination } from '@/components/DataTable';
import { TableCell } from '@/components/ui/table';
import { ApiError, Model, SortState } from '@/types/models';
import { Button } from '@/components/ui/button';
import { Pencil } from 'lucide-react';
import { useModels } from '@/hooks/useQuery';
import AppInitializer from '@/components/AppInitializer';
import { MainLayout } from '@/components/layout/MainLayout';

const columns = [
  { id: 'alias', name: 'Name', sorted: true },
  { id: 'source', name: 'Source', sorted: true },
  { id: 'repo', name: 'Repo', sorted: true },
  { id: 'filename', name: 'Filename', sorted: true },
  { id: 'actions', name: 'Actions', sorted: false },
];

function ModelsPageContent() {
  const router = useRouter();
  const [page, setPage] = useState(1);
  const [pageSize] = useState(30);
  const [sort, setSort] = useState<SortState>({
    column: 'alias',
    direction: 'asc',
  });

  const { data, isLoading, error } = useModels(
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
    setPage(1); // Reset to first page when sorting
  };

  const getItemId = (model: Model) => model.alias;

  const handleEdit = (alias: string) => {
    router.push(`/ui/models/edit?alias=${alias}`);
  };

  const actionUi = (model: Model) => {
    if (model.source === 'model') {
      return <></>;
    } else {
      return (
        <>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => handleEdit(model.alias)}
            title={`Edit ${model.alias}`}
          >
            <Pencil className="h-4 w-4" />
          </Button>
        </>
      );
    }
  };
  const renderRow = (model: Model) => (
    <>
      <TableCell>{model.alias}</TableCell>
      <TableCell>{model.source}</TableCell>
      <TableCell>{model.repo}</TableCell>
      <TableCell>{model.filename}</TableCell>
      <TableCell>{actionUi(model)}</TableCell>
    </>
  );

  const renderExpandedRow = (model: Model) => (
    <div className="p-4 bg-gray-50">
      <h4 className="font-semibold">Additional Details:</h4>
      <p>SHA: {model.snapshot}</p>
      <p>Template: {model.chat_template}</p>
      <h5 className="font-semibold mt-2">Parameters:</h5>
      <p>Request: {JSON.stringify(model.request_params)}</p>
      <p>Context: {JSON.stringify(model.context_params)}</p>
    </div>
  );

  if (error) {
    if (error instanceof AxiosError) {
      return (
        <div>
          An error occurred:{' '}
          {(error.response?.data as ApiError)?.message || error.message}
        </div>
      );
    } else {
      return <div>An error occurred: {(error as Error)?.message}</div>;
    }
  }

  return (
    <MainLayout>
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
    </MainLayout>
  );
}

export default function ModelsPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <ModelsPageContent />
    </AppInitializer>
  );
}
