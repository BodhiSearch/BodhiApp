'use client'

import { useState } from 'react';
import { useQuery } from 'react-query';
import axios from 'axios';
import AppHeader from '@/components/AppHeader';
import { DataTable, Pagination } from '@/components/DataTable';
import { TableCell } from "@/components/ui/table";
import { Model, ModelsResponse, SortState } from '@/types/models';

const columns = [
  { id: 'alias', name: 'Name' },
  { id: 'family', name: 'Family' },
  { id: 'repo', name: 'Repo' },
  { id: 'filename', name: 'Filename' },
  { id: 'features', name: 'Features' },
];

export default function ModelsPage() {
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(30);
  const [sort, setSort] = useState<SortState>({ column: 'alias', direction: 'asc' });

  const fetchModels = async () => {
    const response = await axios.get<ModelsResponse>(`/api/ui/models`, {
      params: {
        page,
        page_size: pageSize,
        sort: sort.column,
        sort_order: sort.direction
      }
    });
    return response.data;
  };

  const { data, isLoading, error } = useQuery(
    ['models', page, pageSize, sort],
    fetchModels,
    { keepPreviousData: true }
  );

  const toggleSort = (column: string) => {
    setSort(prevSort => ({
      column,
      direction: prevSort.column === column && prevSort.direction === 'asc' ? 'desc' : 'asc'
    }));
    setPage(1); // Reset to first page when sorting
  };

  const getItemId = (model: Model) => model.alias;

  const renderRow = (model: Model) => (
    <>
      <TableCell>{model.alias}</TableCell>
      <TableCell>{model.family || ''}</TableCell>
      <TableCell>{model.repo}</TableCell>
      <TableCell>{model.filename}</TableCell>
      <TableCell>{model.features.join(', ')}</TableCell>
    </>
  );

  const renderExpandedRow = (model: Model) => (
    <div className="p-4 bg-gray-50">
      <h4 className="font-semibold">Additional Details:</h4>
      <p>SHA: {model.snapshot}</p>
      <p>Template: {model.chat_template}</p>
      <h5 className="font-semibold mt-2">Parameters:</h5>
      <p>Model: {JSON.stringify(model.model_params)}</p>
      <p>Request: {JSON.stringify(model.request_params)}</p>
      <p>Context: {JSON.stringify(model.context_params)}</p>
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
