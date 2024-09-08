'use client'

import { useState, useEffect } from 'react';
import AppHeader from '@/components/AppHeader';
import { DataTable, SortState, Pagination } from '@/components/DataTable';
import { TableCell } from "@/components/ui/table";

interface ModelFile {
  repo: string;
  filename: string;
  size?: number; // Mark as optional
  updated_at?: string;
  snapshot: string;
  model_params: Record<string, any>;
}

interface ModelFilesResponse {
  data: ModelFile[];
  total: number;
  page: number;
  page_size: number;
}

// Helper function to convert bytes to GB
const bytesToGB = (bytes: number | undefined): string => {
  if (bytes === undefined) return '';
  const gb = bytes / (1024 * 1024 * 1024);
  return gb.toFixed(2) + ' GB';
};

const columns = [
  { id: 'repo', name: 'Repo' },
  { id: 'filename', name: 'Filename' },
  { id: 'size', name: 'Size (GB)' },
  { id: 'updated_at', name: 'Updated At' },
  { id: 'snapshot', name: 'Snapshot' },
];

export default function ModelFilesPage() {
  const [modelFiles, setModelFiles] = useState<ModelFile[]>([]);
  const [loading, setLoading] = useState(true);
  const [page, setPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const [totalItems, setTotalItems] = useState(0);
  const [pageSize, setPageSize] = useState(30);
  const [sort, setSort] = useState<SortState>({ column: 'filename', direction: 'asc' });

  useEffect(() => {
    const fetchModelFiles = async () => {
      setLoading(true);
      try {
        const response = await fetch(`/api/ui/modelfiles?page=${page}&page_size=${pageSize}&sort=${sort.column}&sort_order=${sort.direction}`);
        const data: ModelFilesResponse = await response.json();
        setModelFiles(data.data);
        setTotalPages(Math.ceil(data.total / data.page_size));
        setTotalItems(data.total);
        setPageSize(data.page_size);
      } catch (error) {
        console.error('Error fetching model files:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchModelFiles();
  }, [page, pageSize, sort]);

  const toggleSort = (column: string) => {
    setSort(prevSort => ({
      column,
      direction: prevSort.column === column && prevSort.direction === 'asc' ? 'desc' : 'asc'
    }));
    setPage(1);
  };

  const getItemId = (modelFile: ModelFile) => `${modelFile.repo}${modelFile.filename}${modelFile.snapshot}`;

  const renderRow = (modelFile: ModelFile) => (
    <>
      <TableCell>{modelFile.repo}</TableCell>
      <TableCell>{modelFile.filename}</TableCell>
      <TableCell>{bytesToGB(modelFile.size)}</TableCell>
      <TableCell>{modelFile.updated_at ? new Date(modelFile.updated_at).toLocaleString() : ''}</TableCell>
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

  return (
    <div className="container mx-auto px-4 sm:px-6 lg:px-8">
      <AppHeader />
      <DataTable
        data={modelFiles}
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
          Displaying {modelFiles.length} items of {totalItems}
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
