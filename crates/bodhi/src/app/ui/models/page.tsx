'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { DataTable, Pagination } from '@/components/DataTable';
import { TableCell } from '@/components/ui/table';
import { Model, SortState } from '@/types/models';
import { Button } from '@/components/ui/button';
import {
  Pencil,
  MessageSquare,
  ExternalLink,
  FilePlus2,
  Plus,
} from 'lucide-react';
import { useModels } from '@/hooks/useQuery';
import AppInitializer from '@/components/AppInitializer';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { UserOnboarding } from '@/components/UserOnboarding';
import { CopyButton } from '@/components/CopyButton';

const columns = [
  { id: 'combined', name: 'Models', sorted: true, className: 'sm:hidden' },
  {
    id: 'name_source',
    name: 'Name',
    sorted: true,
    className: 'hidden sm:table-cell lg:hidden',
  },
  {
    id: 'repo_filename',
    name: 'Repository',
    sorted: true,
    className: 'hidden sm:table-cell lg:hidden',
  },
  {
    id: 'alias',
    name: 'Name',
    sorted: true,
    className: 'hidden lg:table-cell',
  },
  { id: 'repo', name: 'Repo', sorted: true, className: 'hidden lg:table-cell' },
  {
    id: 'filename',
    name: 'Filename',
    sorted: true,
    className: 'hidden lg:table-cell',
  },
  {
    id: 'source',
    name: 'Source',
    sorted: true,
    className: 'hidden lg:table-cell',
  },
  { id: 'actions', name: '', sorted: false, className: 'hidden sm:table-cell' },
];

const SourceBadge = ({ source }: { source: string | undefined }) => {
  const colorClass =
    source === 'model'
      ? 'bg-green-500/10 text-green-500'
      : 'bg-blue-500/10 text-blue-500';

  return (
    <span
      className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium w-fit ${colorClass}`}
    >
      {source || ''}
    </span>
  );
};

// Add this component to handle the hover container
const CopyableContent = ({
  text,
  className = '',
}: {
  text: string;
  className?: string;
}) => {
  return (
    <div className={`flex items-center group ${className}`}>
      <span className="truncate">{text}</span>
      <div className="opacity-0 group-hover:opacity-100 transition-opacity">
        <CopyButton text={text} />
      </div>
    </div>
  );
};

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
      <div className="flex flex-nowrap items-center gap-1 md:gap-2">
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

  const handleNewAlias = () => {
    router.push('/ui/models/new');
  };

  const renderRow = (model: Model) => [
    // Mobile view (single column with all items stacked)
    <TableCell key="combined" className="sm:hidden" data-testid="combined-cell">
      <div className="flex flex-col gap-2">
        {/* Name */}
        <CopyableContent text={model.alias} className="font-medium" />

        {/* Repo */}
        <CopyableContent text={model.repo} className="text-sm" />

        {/* Filename */}
        <CopyableContent
          text={model.filename}
          className="text-xs text-muted-foreground"
        />

        {/* Source */}
        <div className="w-fit">
          <SourceBadge source={model.source} />
        </div>

        {/* Actions */}
        <div className="flex items-center gap-1 pt-2 border-t">
          {actionUi(model)}
        </div>
      </div>
    </TableCell>,
    // Tablet view (name+source column)
    <TableCell
      key="name_source"
      className="max-w-[250px] hidden sm:table-cell lg:hidden"
      data-testid="name-source-cell"
    >
      <div className="flex flex-col gap-1">
        <CopyableContent text={model.alias} className="font-medium" />
        <div className="w-fit">
          <SourceBadge source={model.source} />
        </div>
      </div>
    </TableCell>,
    // Tablet view (repo+filename column)
    <TableCell
      key="repo_filename"
      className="max-w-[300px] hidden sm:table-cell lg:hidden"
      data-testid="repo-filename-cell"
    >
      <div className="flex flex-col gap-1">
        <CopyableContent text={model.repo} className="text-sm" />
        <CopyableContent
          text={model.filename}
          className="text-xs text-muted-foreground"
        />
      </div>
    </TableCell>,
    // Desktop view (separate columns)
    <TableCell
      key="alias"
      className="max-w-[250px] truncate hidden lg:table-cell"
      data-testid="alias-cell"
    >
      <CopyableContent text={model.alias} />
    </TableCell>,
    <TableCell
      key="repo"
      className="max-w-[200px] truncate hidden lg:table-cell"
      data-testid="repo-cell"
    >
      <CopyableContent text={model.repo} />
    </TableCell>,
    <TableCell
      key="filename"
      className="max-w-[200px] truncate hidden lg:table-cell"
      data-testid="filename-cell"
    >
      <CopyableContent text={model.filename} />
    </TableCell>,
    <TableCell
      key="source"
      className="max-w-[100px] hidden lg:table-cell"
      data-testid="source-cell"
    >
      <div className="w-fit">
        <SourceBadge source={model.source} />
      </div>
    </TableCell>,
    <TableCell
      key="actions"
      className="w-[140px] whitespace-nowrap hidden sm:table-cell"
    >
      {actionUi(model)}
    </TableCell>,
  ];

  if (error) {
    const errorMessage =
      error.response?.data?.error?.message ||
      error.message ||
      'An unexpected error occurred';
    return <ErrorPage message={errorMessage} />;
  }

  return (
    <div data-testid="models-content" className="container mx-auto p-4">
      <UserOnboarding storageKey="models-banner-dismissed">
        Welcome to Models! Here you can manage your model aliases and access
        their configurations. Create new aliases or edit existing ones to
        customize your model settings.
      </UserOnboarding>

      <div className="flex justify-end m2-4">
        <Button onClick={handleNewAlias} size="sm">
          <Plus className="h-4 w-4 mr-2" />
          New Model Alias
        </Button>
      </div>

      <div className="overflow-x-auto my-4">
        <DataTable
          data={data?.data || []}
          columns={columns}
          loading={isLoading}
          sort={sort}
          onSortChange={toggleSort}
          renderRow={renderRow}
          getItemId={getItemId}
        />
      </div>
      <div className="mt-6 mb-4">
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
