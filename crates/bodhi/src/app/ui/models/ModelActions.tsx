import { AliasResponse } from '@bodhiapp/ts-client';
import { Eye, ExternalLink, FilePlus2, MessageSquare, MoreHorizontal, Pencil, Trash2 } from 'lucide-react';

import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { formatPrefixedModel } from '@/schemas/apiModel';
import { hasLocalFileProperties, isApiAlias } from '@/lib/utils';

export interface ModelActionsProps {
  model: AliasResponse;
  testIdPrefix?: string;
  onPreview: (model: AliasResponse) => void;
  onEdit: (model: AliasResponse) => void;
  onDelete: (model: AliasResponse) => void;
  onShowMoreModels: (models: string[], modelId: string, prefix?: string | null) => void;
  onNew: (model: AliasResponse) => void;
  onChat: (model: AliasResponse) => void;
  getExternalUrl: (model: AliasResponse) => string;
  router: { push: (url: string) => void };
}

const ModelActions = ({
  model,
  testIdPrefix = '',
  onPreview,
  onEdit,
  onDelete,
  onShowMoreModels,
  onNew,
  onChat,
  getExternalUrl,
  router,
}: ModelActionsProps) => {
  if (isApiAlias(model)) {
    return (
      <div className="flex flex-nowrap items-center gap-1 md:gap-2">
        <Button
          variant="ghost"
          size="sm"
          onClick={() => onPreview(model)}
          title={`Preview model ${model.id}`}
          className="h-8 w-8 p-0"
          data-testid={`${testIdPrefix}preview-button-${model.id}`}
        >
          <Eye className="h-4 w-4" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => onEdit(model)}
          title={`Edit API model ${model.id}`}
          className="h-8 w-8 p-0"
          data-testid={`${testIdPrefix}edit-button-${model.id}`}
        >
          <Pencil className="h-4 w-4" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => onDelete(model)}
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
              onClick={() => onShowMoreModels(model.models, model.id, model.prefix)}
              title="Show more models"
              data-testid={`${testIdPrefix}more-models-button-${model.id}`}
            >
              +{model.models.length - 2} more...
            </Button>
          )}
        </div>
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
  }

  const actions =
    model.source === 'model' ? (
      <Button
        variant="ghost"
        size="sm"
        onClick={() => onNew(model)}
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
        onClick={() => onEdit(model)}
        title={`Edit ${model.alias}`}
        className="h-8 w-8 p-0"
        data-testid={`${testIdPrefix}edit-button-${model.alias}`}
      >
        <Pencil className="h-4 w-4" />
      </Button>
    );

  return (
    <div className="flex flex-nowrap items-center gap-1 md:gap-2">
      <Button
        variant="ghost"
        size="sm"
        onClick={() => onPreview(model)}
        title={`Preview model ${model.alias}`}
        className="h-8 w-8 p-0"
        data-testid={`${testIdPrefix}preview-button-${model.alias}`}
      >
        <Eye className="h-4 w-4" />
      </Button>
      {actions}
      <Button
        variant="ghost"
        size="sm"
        onClick={() => onChat(model)}
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
};

export { ModelActions };
