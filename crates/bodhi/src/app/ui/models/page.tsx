'use client';

import { useState, useEffect } from 'react';

import { AliasResponse } from '@bodhiapp/ts-client';
import { Globe, MessageSquare, Plus } from 'lucide-react';
import { useRouter } from 'next/navigation';

import AppInitializer from '@/components/AppInitializer';
import { DataTable, Pagination } from '@/components/DataTable';
import { DeleteConfirmDialog } from '@/components/DeleteConfirmDialog';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { UserOnboarding } from '@/components/UserOnboarding';
import { useToast } from '@/hooks/use-toast';
import { useDeleteApiModel } from '@/hooks/useApiModels';
import { useModels } from '@/hooks/useModels';
import { hasLocalFileProperties, isApiAlias, isUserAlias } from '@/lib/utils';
import { formatPrefixedModel } from '@/schemas/apiModel';
import { SortState } from '@/types/models';

import { ModelTableRow } from '@/app/ui/models/ModelTableRow';
import { ModelPreviewModal } from '@/app/ui/models/components/ModelPreviewModal';

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
    name: 'API Format/Repository',
    sorted: true,
    className: 'hidden sm:table-cell lg:hidden',
  },
  {
    id: 'alias',
    name: 'Name',
    sorted: true,
    className: 'hidden lg:table-cell',
  },
  { id: 'repo', name: 'API Format/Repo', sorted: true, className: 'hidden lg:table-cell' },
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
  { id: 'prefix', name: 'Prefix', sorted: true, className: 'hidden lg:table-cell' },
  { id: 'forward_all', name: 'Forward All', sorted: false, className: 'hidden lg:table-cell' },
  { id: 'actions', name: '', sorted: false, className: 'hidden sm:table-cell' },
];

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
  const [previewModel, setPreviewModel] = useState<AliasResponse | null>(null);

  const { toast } = useToast();
  const deleteApiModel = useDeleteApiModel();

  // Backend will provide combined data including API models, User aliases, and Model File aliases
  const { data, isLoading, error } = useModels(page, pageSize, sort.column, sort.direction);

  // Update preview model when query data changes (after metadata refresh)
  useEffect(() => {
    if (previewModel && data?.data) {
      const modelId = isApiAlias(previewModel) ? previewModel.id : previewModel.alias;
      const updatedModel = data.data.find((m) => {
        const currentId = isApiAlias(m) ? m.id : m.alias;
        return currentId === modelId;
      });
      if (updatedModel) {
        setPreviewModel(updatedModel);
      }
    }
  }, [data?.data, previewModel]);

  const toggleSort = (column: string) => {
    setSort((prevSort) => ({
      column,
      direction: prevSort.column === column && prevSort.direction === 'asc' ? 'desc' : 'asc',
    }));
    setPage(1); // Reset to first page when sorting
  };

  const getItemId = (model: AliasResponse) => {
    return isApiAlias(model) ? model.id : model.alias;
  };

  const handleEdit = (model: AliasResponse) => {
    if (isApiAlias(model)) {
      router.push(`/ui/api-models/edit?id=${model.id}`);
    } else if (isUserAlias(model)) {
      router.push(`/ui/models/edit?id=${model.id}`);
    }
  };

  const handleNew = (model: AliasResponse) => {
    if (hasLocalFileProperties(model)) {
      router.push(`/ui/models/new?repo=${model.repo}&filename=${model.filename}&snapshot=${model.snapshot}`);
    }
  };

  const handleChat = (model: AliasResponse) => {
    const modelIdentifier = isApiAlias(model) ? model.id : model.alias;
    router.push(`/ui/chat?model=${modelIdentifier}`);
  };

  const handleDelete = (model: AliasResponse) => {
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

  const getExternalUrl = (model: AliasResponse) => {
    if (isApiAlias(model)) {
      return model.base_url;
    } else {
      return getHuggingFaceFileUrl(model.repo, model.filename);
    }
  };

  const handleNewAlias = () => {
    router.push('/ui/models/new');
  };

  const handleNewApiModel = () => {
    router.push('/ui/api-models/new');
  };

  const getModelDisplayRepo = (model: AliasResponse): string => {
    if (isApiAlias(model)) {
      return model.api_format;
    } else {
      return model.repo;
    }
  };

  const getModelDisplayFilename = (model: AliasResponse): string => {
    if (isApiAlias(model)) {
      return model.base_url;
    } else {
      return model.filename;
    }
  };

  const modelActionsProps = {
    onPreview: setPreviewModel,
    onEdit: handleEdit,
    onDelete: handleDelete,
    onShowMoreModels: handleShowMoreModels,
    onNew: handleNew,
    onChat: handleChat,
    getExternalUrl,
    router,
  };

  const renderRow = (model: AliasResponse) => (
    <ModelTableRow
      model={model}
      getItemId={getItemId}
      getModelDisplayRepo={getModelDisplayRepo}
      getModelDisplayFilename={getModelDisplayFilename}
      actionsProps={modelActionsProps}
    />
  );

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
          getRowProps={(model: AliasResponse) => ({
            'data-test-model-id': isUserAlias(model) ? model.id : isApiAlias(model) ? model.id : undefined,
          })}
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

      {/* Model Preview Modal */}
      {previewModel && (
        <ModelPreviewModal
          open={!!previewModel}
          onOpenChange={(open) => !open && setPreviewModel(null)}
          model={previewModel}
        />
      )}
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
