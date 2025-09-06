'use client';

import AppInitializer from '@/components/AppInitializer';
import { CopyableContent } from '@/components/CopyableContent';
import { DataTable, Pagination } from '@/components/DataTable';
import { DeleteConfirmDialog } from '@/components/DeleteConfirmDialog';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { TableCell } from '@/components/ui/table';
import { UserOnboarding } from '@/components/UserOnboarding';
import { useToast } from '@/hooks/use-toast';
import { useDeleteApiModel } from '@/hooks/useApiModels';
import { useModels } from '@/hooks/useQuery';
import { hasLocalFileProperties, isApiAlias } from '@/lib/utils';
import { SortState } from '@/types/models';
import { Alias } from '@bodhiapp/ts-client';
import { formatPrefixedModel } from '@/schemas/apiModel';
import {
  Cloud,
  ExternalLink,
  FilePlus2,
  Globe,
  MessageSquare,
  MoreHorizontal,
  Pencil,
  Plus,
  Trash2,
} from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useState } from 'react';

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

const SourceBadge = ({ model, testIdPrefix = '' }: { model: Alias; testIdPrefix?: string }) => {
  const prefix = testIdPrefix ? `${testIdPrefix}` : '';

  if (isApiAlias(model)) {
    return (
      <Badge
        variant="outline"
        className="bg-purple-500/10 text-purple-600 border-purple-200"
        data-testid={`${prefix}source-badge-api_${model.id}`}
      >
        <Cloud className="h-3 w-3 mr-1" />
        API
      </Badge>
    );
  }

  const source = model.source;
  const colorClass = source === 'model' ? 'bg-green-500/10 text-green-500' : 'bg-blue-500/10 text-blue-500';
  return (
    <span
      className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium w-fit ${colorClass}`}
      data-testid={`${prefix}source-badge-${model.alias}`}
    >
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
  const [deleteModel, setDeleteModel] = useState<{ id: string; name: string } | null>(null);
  const [moreModelsModal, setMoreModelsModal] = useState<{
    models: string[];
    modelId: string;
    prefix?: string | null;
  } | null>(null);

  const { toast } = useToast();
  const deleteApiModel = useDeleteApiModel();

  // Backend will provide combined data including API models, User aliases, and Model File aliases
  const { data, isLoading, error } = useModels(page, pageSize, sort.column, sort.direction);

  const toggleSort = (column: string) => {
    setSort((prevSort) => ({
      column,
      direction: prevSort.column === column && prevSort.direction === 'asc' ? 'desc' : 'asc',
    }));
    setPage(1); // Reset to first page when sorting
  };

  const getItemId = (model: Alias) => {
    return isApiAlias(model) ? `api_${model.id}` : model.alias;
  };

  const handleEdit = (model: Alias) => {
    if (isApiAlias(model)) {
      router.push(`/ui/api-models/edit?id=${model.id}`);
    } else {
      router.push(`/ui/models/edit?alias=${model.alias}`);
    }
  };

  const handleNew = (model: Alias) => {
    if (hasLocalFileProperties(model)) {
      router.push(`/ui/models/new?repo=${model.repo}&filename=${model.filename}&snapshot=${model.snapshot}`);
    }
  };

  const handleChat = (model: Alias) => {
    const modelIdentifier = isApiAlias(model) ? model.id : model.alias;
    router.push(`/ui/chat?model=${modelIdentifier}`);
  };

  const handleDelete = (model: Alias) => {
    if (isApiAlias(model)) {
      setDeleteModel({ id: model.id, name: model.id });
    }
  };

  const confirmDelete = async () => {
    if (!deleteModel) return;

    try {
      await deleteApiModel.mutateAsync(deleteModel.id);
      toast({
        title: 'Success',
        description: `API model ${deleteModel.name} deleted successfully`,
      });
      setDeleteModel(null);
    } catch (error) {
      toast({
        title: 'Error',
        description: 'Failed to delete API model',
        variant: 'destructive',
      });
    }
  };

  const handleShowMoreModels = (models: string[], modelId: string, prefix?: string | null) => {
    setMoreModelsModal({ models: models.slice(2), modelId, prefix });
  };

  const getHuggingFaceFileUrl = (repo: string, filename: string) => {
    return `https://huggingface.co/${repo}/blob/main/${filename}`;
  };

  const getExternalUrl = (model: Alias) => {
    if (isApiAlias(model)) {
      return model.base_url;
    } else {
      return getHuggingFaceFileUrl(model.repo, model.filename);
    }
  };

  const actionUi = (model: Alias, testIdPrefix = '') => {
    if (isApiAlias(model)) {
      // API model actions
      return (
        <div className="flex flex-nowrap items-center gap-1 md:gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => handleEdit(model)}
            title={`Edit API model ${model.id}`}
            className="h-8 w-8 p-0"
            data-testid={`${testIdPrefix}edit-button-${model.id}`}
          >
            <Pencil className="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => handleDelete(model)}
            title={`Delete API model ${model.id}`}
            className="h-8 w-8 p-0 text-destructive hover:text-destructive"
            data-testid={`${testIdPrefix}delete-button-${model.id}`}
          >
            <Trash2 className="h-4 w-4" />
          </Button>
          <div className="hidden sm:flex items-center gap-1">
            {model.models
              .map((modelName) => {
                const displayName = formatPrefixedModel(modelName, model.prefix);
                const chatModel = formatPrefixedModel(modelName, model.prefix);
                return (
                  <Button
                    key={`${model.id}-${modelName}`}
                    variant="ghost"
                    size="sm"
                    className="h-8 px-2 text-xs"
                    onClick={() => router.push(`/ui/chat?model=${chatModel}`)}
                    title={`Chat with ${displayName}`}
                    data-testid={`${testIdPrefix}model-chat-button-${chatModel}`}
                  >
                    {displayName}
                  </Button>
                );
              })
              .slice(0, 2)}
            {model.models.length > 2 && (
              <Button
                variant="ghost"
                size="sm"
                className="h-8 px-2 text-xs text-muted-foreground"
                onClick={() => handleShowMoreModels(model.models, model.id, model.prefix)}
                title="Show more models"
                data-testid={`${testIdPrefix}more-models-button-${model.id}`}
              >
                +{model.models.length - 2} more...
              </Button>
            )}
          </div>
          {/* Mobile view - dropdown for models */}
          <div className="sm:hidden">
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-8 w-8 p-0"
                  title="Chat with models"
                  data-testid={`${testIdPrefix}models-dropdown-${model.id}`}
                >
                  <MoreHorizontal className="h-4 w-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                {model.models.map((modelName) => {
                  const displayName = formatPrefixedModel(modelName, model.prefix);
                  const chatModel = formatPrefixedModel(modelName, model.prefix);
                  return (
                    <DropdownMenuItem key={modelName} onClick={() => router.push(`/ui/chat?model=${chatModel}`)}>
                      {displayName}
                    </DropdownMenuItem>
                  );
                })}
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
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
            data-testid={`${testIdPrefix}create-alias-from-model-${model.alias}`}
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
            data-testid={`${testIdPrefix}edit-button-${model.alias}`}
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
            data-testid={`${testIdPrefix}chat-button-${model.alias}`}
          >
            <MessageSquare className="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className="h-8 w-8 p-0"
            onClick={() => window.open(getExternalUrl(model), '_blank')}
            title="Open in HuggingFace"
            data-testid={`${testIdPrefix}external-button-${model.alias}`}
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

  const getModelDisplayRepo = (model: Alias): string => {
    if (isApiAlias(model)) {
      return model.provider;
    } else {
      return model.repo;
    }
  };

  const getModelDisplayFilename = (model: Alias): string => {
    if (isApiAlias(model)) {
      return model.base_url;
    } else {
      return model.filename;
    }
  };

  const renderRow = (model: Alias) => [
    // Mobile view (single column with all items stacked)
    <TableCell key="combined" className="sm:hidden" data-testid={`combined-cell-${getItemId(model)}`}>
      <div className="flex flex-col gap-2">
        {/* Name - for API models, show list of models */}
        <CopyableContent text={isApiAlias(model) ? model.id : model.alias} className="font-medium" />
        {isApiAlias(model) && <div className="text-xs text-muted-foreground">Models: {model.models.join(', ')}</div>}

        {/* Repo/Provider */}
        <CopyableContent text={getModelDisplayRepo(model)} className="text-sm" />

        {/* Filename/Base URL */}
        <CopyableContent text={getModelDisplayFilename(model)} className="text-xs text-muted-foreground" />

        {/* Source/Type */}
        <div className="w-fit">
          <SourceBadge model={model} testIdPrefix="m-" />
        </div>

        {/* Actions */}
        <div className="flex items-center gap-1 pt-2 border-t" data-testid={`actions-${getItemId(model)}`}>
          {actionUi(model, 'm-')}
        </div>
      </div>
    </TableCell>,
    // Tablet view (name+source column)
    <TableCell
      key="name_source"
      className="max-w-[250px] hidden sm:table-cell lg:hidden"
      data-testid={`name-source-cell-${getItemId(model)}`}
    >
      <div className="flex flex-col gap-1">
        <CopyableContent text={isApiAlias(model) ? model.id : model.alias} className="font-medium" />
        {isApiAlias(model) && (
          <div className="text-xs text-muted-foreground truncate">
            {model.models.slice(0, 2).join(', ')}
            {model.models.length > 2 ? '...' : ''}
          </div>
        )}
        <div className="w-fit">
          <SourceBadge model={model} testIdPrefix="tab-" />
        </div>
      </div>
    </TableCell>,
    // Tablet view (repo+filename column)
    <TableCell
      key="repo_filename"
      className="max-w-[300px] hidden sm:table-cell lg:hidden"
      data-testid={`repo-filename-cell-${getItemId(model)}`}
    >
      <div className="flex flex-col gap-1">
        <CopyableContent text={getModelDisplayRepo(model)} className="text-sm" />
        <CopyableContent text={getModelDisplayFilename(model)} className="text-xs text-muted-foreground truncate" />
      </div>
    </TableCell>,
    // Desktop view (separate columns)
    <TableCell
      key="alias"
      className="max-w-[250px] hidden lg:table-cell"
      data-testid={`alias-cell-${getItemId(model)}`}
    >
      <div className="flex flex-col gap-1">
        <CopyableContent text={isApiAlias(model) ? model.id : model.alias} />
      </div>
    </TableCell>,
    <TableCell
      key="repo"
      className="max-w-[200px] truncate hidden lg:table-cell"
      data-testid={`repo-cell-${getItemId(model)}`}
    >
      <CopyableContent text={getModelDisplayRepo(model)} />
    </TableCell>,
    <TableCell
      key="filename"
      className="max-w-[200px] hidden lg:table-cell"
      data-testid={`filename-cell-${getItemId(model)}`}
    >
      <CopyableContent text={getModelDisplayFilename(model)} className="truncate" />
    </TableCell>,
    <TableCell
      key="source"
      className="max-w-[100px] hidden lg:table-cell"
      data-testid={`source-cell-${getItemId(model)}`}
    >
      <div className="w-fit">
        <SourceBadge model={model} />
      </div>
    </TableCell>,
    <TableCell
      key="actions"
      className="w-[140px] whitespace-nowrap hidden sm:table-cell"
      data-testid={`actions-cell-${getItemId(model)}`}
    >
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
        <Button onClick={handleNewAlias} size="sm" data-testid="new-model-alias-button">
          <Plus className="h-4 w-4 mr-2" />
          New Model Alias
        </Button>
      </div>

      <div className="overflow-x-auto my-4" data-testid="table-list-models">
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
      {/* Delete Confirmation Dialog */}
      <DeleteConfirmDialog
        open={!!deleteModel}
        onOpenChange={(open) => !open && setDeleteModel(null)}
        title="Delete API Model"
        description={`Are you sure you want to delete the API model "${deleteModel?.name}"? This action cannot be undone.`}
        onConfirm={confirmDelete}
        loading={deleteApiModel.isLoading}
      />

      {/* More Models Modal */}
      <Dialog open={!!moreModelsModal} onOpenChange={(open) => !open && setMoreModelsModal(null)}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>Additional Models</DialogTitle>
          </DialogHeader>
          <div className="grid gap-2">
            {moreModelsModal?.models.map((modelName) => {
              const displayName = formatPrefixedModel(modelName, moreModelsModal.prefix);
              const chatModel = formatPrefixedModel(modelName, moreModelsModal.prefix);
              return (
                <Button
                  key={modelName}
                  variant="outline"
                  className="justify-start"
                  onClick={() => {
                    router.push(`/ui/chat?model=${chatModel}`);
                    setMoreModelsModal(null);
                  }}
                >
                  <MessageSquare className="h-4 w-4 mr-2" />
                  {displayName}
                </Button>
              );
            })}
          </div>
        </DialogContent>
      </Dialog>
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
