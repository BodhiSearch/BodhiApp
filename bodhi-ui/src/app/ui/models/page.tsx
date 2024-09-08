'use client'

import { useState, useEffect } from 'react';
import AppHeader from '@/components/AppHeader';
import { DataTable, SortState, Pagination } from '@/components/DataTable';
import { TableCell } from "@/components/ui/table";

interface Model {
  alias: string;
  family?: string;
  repo: string;
  filename: string;
  snapshot: string;
  features: string[];
  chat_template: string;
  model_params: Record<string, any>;
  request_params: Record<string, any>;
  context_params: Record<string, any>;
}

interface ModelsResponse {
  data: Model[];
  total: number;
  page: number;
  page_size: number;
}

const columns = [
  { id: 'alias', name: 'Name' },
  { id: 'family', name: 'Family' },
  { id: 'repo', name: 'Repo' },
  { id: 'filename', name: 'Filename' },
  { id: 'features', name: 'Features' },
];

export default function ModelsPage() {
  const [models, setModels] = useState<Model[]>([]);
  const [loading, setLoading] = useState(true);
  const [page, setPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const [totalItems, setTotalItems] = useState(0);
  const [pageSize, setPageSize] = useState(30);
  const [expandedRow, setExpandedRow] = useState<string | null>(null);
  const [sort, setSort] = useState<SortState>({ column: 'alias', direction: 'asc' });

  useEffect(() => {
    const fetchModels = async () => {
      setLoading(true);
      try {
        const response = await fetch(`/api/ui/models?page=${page}&page_size=${pageSize}&sort=${sort.column}&sort_order=${sort.direction}`);
        const data: ModelsResponse = await response.json();
        setModels(data.data);
        setTotalPages(Math.ceil(data.total / data.page_size));
        setTotalItems(data.total);
        setPageSize(data.page_size);
      } catch (error) {
        console.error('Error fetching models:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchModels();
  }, [page, pageSize, sort]);

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

  return (
    <div className="container mx-auto px-4 sm:px-6 lg:px-8">
      <AppHeader />
      <DataTable
        data={models}
        columns={columns}
        loading={loading}
        sort={sort}
        onSortChange={toggleSort}
        renderRow={renderRow}
        renderExpandedRow={renderExpandedRow}
        getItemId={getItemId}
      />
      <div className="mt-4 flex flex-col sm:flex-row justify-between items-center">
        <div className="mb-2 sm:mb-0">
          Displaying {models.length} items of {totalItems}
        </div>
        <Pagination
          page={page}
          totalPages={totalPages}
          onPageChange={setPage}
        />
      </div>
    </div>
  );
}
