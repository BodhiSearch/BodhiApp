'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { DataTable, Pagination } from '@/components/DataTable';
import { TableCell } from '@/components/ui/table';
import { UnifiedModelResponse } from '@bodhiapp/ts-client';
import { SortState } from '@/types/models';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Pencil, MessageSquare, ExternalLink, FilePlus2, Plus, Cloud, Globe } from 'lucide-react';
import { useModels } from '@/hooks/useQuery';
import AppInitializer from '@/components/AppInitializer';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { UserOnboarding } from '@/components/UserOnboarding';
import { CopyableContent } from '@/components/CopyableContent';

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
    name: 'Provider/Repository',
    sorted: true,
    className: 'hidden sm:table-cell lg:hidden',
  },
  {
    id: 'alias',
    name: 'Name',
    sorted: true,
    className: 'hidden lg:table-cell',
  },
  { id: 'repo', name: 'Provider/Repo', sorted: true, className: 'hidden lg:table-cell' },
  {
    id: 'filename',
    name: 'File/Endpoint',
    sorted: true,
    className: 'hidden lg:table-cell',
  },
  {
    id: 'source',
    name: 'Type',
    sorted: true,
    className: 'hidden lg:table-cell',
  },
  { id: 'actions', name: '', sorted: false, className: 'hidden sm:table-cell' },
];

const SourceBadge = ({ model }: { model: UnifiedModelResponse }) => {
  if (model.model_type === 'api') {
    return (
      <Badge variant="outline" className="bg-purple-500/10 text-purple-600 border-purple-200">
        <Cloud className="h-3 w-3 mr-1" />
        API
      </Badge>
    );
  }

  const source = model.source;
  const colorClass = source === 'model' ? 'bg-green-500/10 text-green-500' : 'bg-blue-500/10 text-blue-500';
  return (
    <span className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium w-fit ${colorClass}`}>
      {source || ''}
    </span>
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

  // Backend will provide combined data including API models, User aliases, and Model File aliases
  const { data, isLoading, error } = useModels(page, pageSize, sort.column, sort.direction);

  const toggleSort = (column: string) => {
    setSort((prevSort) => ({
      column,
      direction: prevSort.column === column && prevSort.direction === 'asc' ? 'desc' : 'asc',
    }));
    setPage(1); // Reset to first page when sorting
  };

  const getItemId = (model: UnifiedModelResponse) => {
    return model.model_type === 'api' ? `api_${model.id}` : model.alias;
  };

  const handleEdit = (model: UnifiedModelResponse) => {
    if (model.model_type === 'api') {
      router.push(`/ui/api-models/edit?id=${model.id}`);
    } else {
      router.push(`/ui/models/edit?alias=${model.alias}`);
    }
  };

  const handleNew = (model: UnifiedModelResponse) => {
    if (model.model_type === 'local') {
      router.push(`/ui/models/new?repo=${model.repo}&filename=${model.filename}&snapshot=${model.snapshot}`);
    }
  };

  const handleChat = (model: UnifiedModelResponse) => {
    const modelIdentifier = model.model_type === 'api' ? model.id : model.alias;
    router.push(`/ui/chat?model=${modelIdentifier}`);
  };

  const getHuggingFaceFileUrl = (repo: string, filename: string) => {
    return `https://huggingface.co/${repo}/blob/main/${filename}`;
  };

  const getExternalUrl = (model: UnifiedModelResponse) => {
    if (model.model_type === 'api') {
      return model.base_url;
    } else {
      return getHuggingFaceFileUrl(model.repo, model.filename);
    }
  };

  const actionUi = (model: UnifiedModelResponse) => {
    if (model.model_type === 'api') {
      // API model actions
      return (
        <div className="flex flex-nowrap items-center gap-1 md:gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => handleEdit(model)}
            title={`Edit API model ${model.id}`}
            className="h-8 w-8 p-0"
          >
            <Pencil className="h-4 w-4" />
          </Button>
          {model.models
            .map((modelName) => (
              <Button
                key={`${model.id}-${modelName}`}
                variant="ghost"
                size="sm"
                className="h-8 px-2 text-xs"
                onClick={() => router.push(`/ui/chat?model=${modelName}`)}
                title={`Chat with ${modelName}`}
              >
                {modelName}
              </Button>
            ))
            .slice(0, 2)}{' '}
          {/* Show only first 2 models */}
          {model.models.length > 2 && (
            <span className="text-xs text-muted-foreground">+{model.models.length - 2} more</span>
          )}
        </div>
      );
    } else {
      // Regular model actions
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
            onClick={() => handleEdit(model)}
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
            onClick={() => window.open(getExternalUrl(model), '_blank')}
            title="Open in HuggingFace"
          >
            <ExternalLink className="h-4 w-4" />
          </Button>
        </div>
      );
    }
  };

  const handleNewAlias = () => {
    router.push('/ui/models/new');
  };

  const handleNewApiModel = () => {
    router.push('/ui/api-models/new');
  };

  const getModelDisplayRepo = (model: UnifiedModelResponse): string => {
    if (model.model_type === 'api') {
      return model.provider;
    } else {
      return model.repo;
    }
  };

  const getModelDisplayFilename = (model: UnifiedModelResponse): string => {
    if (model.model_type === 'api') {
      return model.base_url;
    } else {
      return model.filename;
    }
  };

  const renderRow = (model: UnifiedModelResponse) => [
    // Mobile view (single column with all items stacked)
    <TableCell key="combined" className="sm:hidden" data-testid="combined-cell">
      <div className="flex flex-col gap-2">
        {/* Name - for API models, show list of models */}
        <CopyableContent text={model.model_type === 'api' ? model.id : model.alias} className="font-medium" />
        {model.model_type === 'api' && (
          <div className="text-xs text-muted-foreground">Models: {model.models.join(', ')}</div>
        )}

        {/* Repo/Provider */}
        <CopyableContent text={getModelDisplayRepo(model)} className="text-sm" />

        {/* Filename/Base URL */}
        <CopyableContent text={getModelDisplayFilename(model)} className="text-xs text-muted-foreground" />

        {/* Source/Type */}
        <div className="w-fit">
          <SourceBadge model={model} />
        </div>

        {/* Actions */}
        <div className="flex items-center gap-1 pt-2 border-t">{actionUi(model)}</div>
      </div>
    </TableCell>,
    // Tablet view (name+source column)
    <TableCell
      key="name_source"
      className="max-w-[250px] hidden sm:table-cell lg:hidden"
      data-testid="name-source-cell"
    >
      <div className="flex flex-col gap-1">
        <CopyableContent text={model.model_type === 'api' ? model.id : model.alias} className="font-medium" />
        {model.model_type === 'api' && (
          <div className="text-xs text-muted-foreground truncate">
            {model.models.slice(0, 2).join(', ')}
            {model.models.length > 2 ? '...' : ''}
          </div>
        )}
        <div className="w-fit">
          <SourceBadge model={model} />
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
        <CopyableContent text={getModelDisplayRepo(model)} className="text-sm" />
        <CopyableContent text={getModelDisplayFilename(model)} className="text-xs text-muted-foreground truncate" />
      </div>
    </TableCell>,
    // Desktop view (separate columns)
    <TableCell key="alias" className="max-w-[250px] hidden lg:table-cell" data-testid="alias-cell">
      <div className="flex flex-col gap-1">
        <CopyableContent text={model.model_type === 'api' ? model.id : model.alias} />
      </div>
    </TableCell>,
    <TableCell key="repo" className="max-w-[200px] truncate hidden lg:table-cell" data-testid="repo-cell">
      <CopyableContent text={getModelDisplayRepo(model)} />
    </TableCell>,
    <TableCell key="filename" className="max-w-[200px] hidden lg:table-cell" data-testid="filename-cell">
      <CopyableContent text={getModelDisplayFilename(model)} className="truncate" />
    </TableCell>,
    <TableCell key="source" className="max-w-[100px] hidden lg:table-cell" data-testid="source-cell">
      <div className="w-fit">
        <SourceBadge model={model} />
      </div>
    </TableCell>,
    <TableCell key="actions" className="w-[140px] whitespace-nowrap hidden sm:table-cell">
      {actionUi(model)}
    </TableCell>,
  ];

  if (error) {
    const errorMessage = error.response?.data?.error?.message || error.message || 'An unexpected error occurred';
    return <ErrorPage message={errorMessage} />;
  }

  return (
    <div data-testid="models-content" className="container mx-auto p-4">
      <UserOnboarding storageKey="models-banner-dismissed">
        Welcome to Models! Here you can manage your model aliases and API models. Create new aliases for local models or
        configure external AI APIs to expand your available models.
      </UserOnboarding>

      <div className="flex justify-end gap-2 m2-4">
        <Button onClick={handleNewApiModel} size="sm" variant="outline">
          <Globe className="h-4 w-4 mr-2" />
          New API Model
        </Button>
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
          totalPages={data ? Math.ceil((data.total as number) / (data.page_size as number)) : 1}
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
