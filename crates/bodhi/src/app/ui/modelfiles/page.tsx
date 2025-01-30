'use client';

import { useState } from 'react';
import { DataTable, Pagination } from '@/components/DataTable';
import { TableCell } from '@/components/ui/table';
import { ModelFile, SortState } from '@/types/models';
import { useModelFiles } from '@/hooks/useQuery';
import AppInitializer from '@/components/AppInitializer';
import { ExternalLink, Trash2, X } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useLocalStorage } from '@/hooks/useLocalStorage';
import { Alert, AlertDescription } from '@/components/ui/alert';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { ErrorPage } from '@/components/ui/ErrorPage';

// Helper function to convert bytes to GB
const bytesToGB = (bytes: number | undefined): string => {
  if (bytes === undefined) return '';
  const gb = bytes / (1024 * 1024 * 1024);
  return `${gb.toFixed(2)} GB`;
};

export const columns = [
  // Mobile view (combined column)
  { id: 'combined', name: 'Model Files', sorted: true, className: 'sm:hidden' },
  // Tablet/Desktop columns
  {
    id: 'repo',
    name: 'Repo',
    sorted: true,
    className: 'hidden sm:table-cell max-w-[180px]',
  },
  {
    id: 'filename',
    name: 'Filename',
    sorted: true,
    className: 'hidden sm:table-cell',
  },
  {
    id: 'size',
    name: 'Size',
    sorted: true,
    className: 'hidden sm:table-cell w-20 text-right',
  },
  {
    id: 'actions',
    name: '',
    sorted: false,
    className: 'hidden sm:table-cell w-20',
  },
];

function ModelFilesContent() {
  const [showDeleteDialog, setShowDeleteDialog] = useState(false);
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

  const renderRow = (modelFile: ModelFile) => [
    // Mobile view (combined column)
    <TableCell key="combined" className="sm:hidden" data-testid="combined-cell">
      <div className="flex flex-col gap-2">
        <span className="font-medium truncate">{modelFile.repo}</span>
        <span className="truncate text-sm">{modelFile.filename}</span>
        <span className="text-xs text-muted-foreground">
          {bytesToGB(modelFile.size)}
        </span>
        <div className="flex gap-2 justify-end pt-2 border-t">
          {renderActions(modelFile)}
        </div>
      </div>
    </TableCell>,
    // Tablet/Desktop columns
    <TableCell
      key="repo"
      className="hidden sm:table-cell max-w-[180px]"
      data-testid="repo-cell"
    >
      <div className="truncate">{modelFile.repo}</div>
    </TableCell>,
    <TableCell
      key="filename"
      className="hidden sm:table-cell"
      data-testid="filename-cell"
    >
      {modelFile.filename}
    </TableCell>,
    <TableCell
      key="size"
      className="hidden sm:table-cell text-right"
      data-testid="size-cell"
    >
      {bytesToGB(modelFile.size)}
    </TableCell>,
    <TableCell
      key="actions"
      className="hidden sm:table-cell"
      data-testid="actions-cell"
    >
      <div className="flex gap-2 justify-end">{renderActions(modelFile)}</div>
    </TableCell>,
  ];

  // Extract actions to a separate function for reuse
  const renderActions = (modelFile: ModelFile) => (
    <>
      <Button
        variant="ghost"
        size="sm"
        className="h-8 w-8 p-0"
        onClick={() => setShowDeleteDialog(true)}
        title="Delete modelfile"
      >
        <Trash2 className="h-4 w-4" />
      </Button>
      <Button
        variant="ghost"
        size="sm"
        className="h-8 w-8 p-0"
        onClick={() => window.open(getHuggingFaceUrl(modelFile.repo), '_blank')}
        title="Open in HuggingFace"
      >
        <ExternalLink className="h-4 w-4" />
      </Button>
    </>
  );

  if (error) {
    const errorMessage =
      error.response?.data?.error?.message ||
      error.message ||
      'An unexpected error occurred. Please try again.';
    return <ErrorPage message={errorMessage} />;
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

      <Dialog open={showDeleteDialog} onOpenChange={setShowDeleteDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Coming Soon</DialogTitle>
            <DialogDescription>
              Delete modelfile feature is not yet implemented. Stay tuned for
              our next update.
            </DialogDescription>
          </DialogHeader>
        </DialogContent>
      </Dialog>

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
