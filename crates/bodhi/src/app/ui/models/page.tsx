'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { DataTable, Pagination } from '@/components/DataTable';
import { TableCell } from '@/components/ui/table';
import { Model, SortState } from '@/types/models';
import { Button } from '@/components/ui/button';
import { Pencil, MessageSquare, ExternalLink, FilePlus2 } from 'lucide-react';
import { useModels } from '@/hooks/useQuery';
import AppInitializer from '@/components/AppInitializer';

const columns = [
  { id: 'alias', name: 'Name', sorted: true },
  { id: 'source', name: 'Source', sorted: true },
  { id: 'repo', name: 'Repo', sorted: true },
  { id: 'filename', name: 'Filename', sorted: true },
  { id: 'actions', name: '', sorted: false },
];

function ModelsPageContent() {
  const router = useRouter();
  const [page, setPage] = useState(1);
  const [pageSize] = useState(10);
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
  const handleNew = (model: Model) => {
    router.push(
      `/ui/models/new?repo=${model.repo}&filename=${model.filename}&snapshot=${model.snapshot}`
    );
  };
  const handleChat = (model: Model) => {
    router.push(`/ui/chat?alias=${model.alias}`);
  };
  const getHuggingFaceFileUrl = (repo: string, filename: string) => {
    return `https://huggingface.co/${repo}/blob/main/${filename}`;
  };
  const actionUi = (model: Model) => {
    const actions =
      model.source === 'model' ? (
        <Button
          variant="ghost"
          size="sm"
          onClick={() => handleNew(model)}
          title={`Create new model alias using this modelfile`}
          className="h-8 w-8 p-0"
        >
          <FilePlus2 className="h-4 w-4" />
        </Button>
      ) : (
        <Button
          variant="ghost"
          size="sm"
          onClick={() => handleEdit(model.alias)}
          title={`Edit ${model.alias}`}
          className="h-8 w-8 p-0"
        >
          <Pencil className="h-4 w-4" />
        </Button>
      );
    return (
      <div className="flex gap-2 justify-end">
        {actions}
        <Button
          variant="ghost"
          size="sm"
          onClick={() => handleChat(model)}
          title={`Chat with the model in playground`}
          className="h-8 w-8 p-0"
        >
          <MessageSquare className="h-4 w-4" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          className="h-8 w-8 p-0"
          onClick={() =>
            window.open(
              getHuggingFaceFileUrl(model.repo, model.filename),
              '_blank'
            )
          }
          title="Open in HuggingFace"
        >
          <ExternalLink className="h-4 w-4" />
        </Button>
      </div>
    );
  };

  const renderRow = (model: Model) => [
    <TableCell key="alias">{model.alias}</TableCell>,
    <TableCell key="source">{model.source}</TableCell>,
    <TableCell key="repo">{model.repo}</TableCell>,
    <TableCell key="filename">{model.filename}</TableCell>,
    <TableCell key="actions" className="w-10">
      {actionUi(model)}
    </TableCell>,
  ];

  if (error) {
    return (
      <div className="text-destructive text-center" role="alert">
        {error.response?.data?.error?.message ||
          error.message ||
          'An unexpected error occurred'}
      </div>
    );
  }

  return (
    <div data-testid="models-content" className="container mx-auto p-4">
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

export default function ModelsPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <ModelsPageContent />
    </AppInitializer>
  );
}
