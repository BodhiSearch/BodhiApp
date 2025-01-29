'use client';

import { useState } from 'react';
import { DataTable, Pagination } from '@/components/DataTable';
import { TableCell } from '@/components/ui/table';
import { ModelFile, SortState } from '@/types/models';
import { useModelFiles } from '@/hooks/useQuery';
import AppInitializer from '@/components/AppInitializer';
import { ExternalLink, X } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useLocalStorage } from '@/hooks/useLocalStorage';
import { Alert, AlertDescription } from '@/components/ui/alert';

// Helper function to convert bytes to GB
const bytesToGB = (bytes: number | undefined): string => {
  if (bytes === undefined) return '';
  const gb = bytes / (1024 * 1024 * 1024);
  return `${gb.toFixed(2)} GB`;
};

const columns = [
  {
    id: 'repo',
    name: 'Repo',
    sorted: true,
    className: 'max-w-[180px] truncate',
  },
  {
    id: 'filename',
    name: 'Filename',
    sorted: true,
    className: 'hidden sm:table-cell',
  },
  { id: 'size', name: 'Size', sorted: true, className: 'w-20 text-right' },
  { id: 'actions', name: '', sorted: false, className: 'w-10' },
];

function ModelFilesContent() {
  const [hasDismissedBanner, setHasDismissedBanner] = useLocalStorage(
    'modelfiles-banner-dismissed',
    false
  );

  const [page, setPage] = useState(1);
  const [pageSize] = useState(10);
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

  const getHuggingFaceUrl = (repo: string) => {
    return `https://huggingface.co/${repo}`;
  };

  const renderRow = (modelFile: ModelFile) => (
    <>
      <TableCell className="max-w-[180px]">
        <div className="truncate">{modelFile.repo}</div>
        <div className="text-xs text-muted-foreground truncate sm:hidden mt-1">
          {modelFile.filename}
        </div>
      </TableCell>
      <TableCell className="hidden sm:table-cell">
        {modelFile.filename}
      </TableCell>
      <TableCell className="text-right">{bytesToGB(modelFile.size)}</TableCell>
      <TableCell>
        <Button
          variant="ghost"
          size="sm"
          className="h-8 w-8 p-0"
          onClick={() =>
            window.open(getHuggingFaceUrl(modelFile.repo), '_blank')
          }
          title="Open in HuggingFace"
        >
          <ExternalLink className="h-4 w-4" />
        </Button>
      </TableCell>
    </>
  );

  if (error) {
    const errorMessage =
      error.response?.data?.error?.message ||
      error.message ||
      'An unexpected error occurred. Please try again.';
    return (
      <div className="text-destructive text-center" role="alert">
        {errorMessage}
      </div>
    );
  }

  return (
    <div data-testid="modelfiles-content" className="container mx-auto p-4">
      {!hasDismissedBanner && (
        <Alert className="mb-4">
          <AlertDescription className="flex items-center justify-between gap-4">
            <span>
              Welcome to Model Management! Here you can view all your downloaded
              models and access their HuggingFace repositories.
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

      <DataTable
        data={data?.data || []}
        columns={columns}
        loading={isLoading}
        sort={sort}
        onSortChange={toggleSort}
        renderRow={renderRow}
        getItemId={getItemId}
      />
      <div className="mt-6">
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
