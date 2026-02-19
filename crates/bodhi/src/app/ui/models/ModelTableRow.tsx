import { AliasResponse } from '@bodhiapp/ts-client';

import { CopyableContent } from '@/components/CopyableContent';
import { Badge } from '@/components/ui/badge';
import { TableCell } from '@/components/ui/table';
import { isApiAlias, isUserAlias } from '@/lib/utils';

import { ModelActions, type ModelActionsProps } from '@/app/ui/models/ModelActions';
import { SourceBadge } from '@/app/ui/models/SourceBadge';

export interface ModelTableRowProps {
  model: AliasResponse;
  getItemId: (model: AliasResponse) => string;
  getModelDisplayRepo: (model: AliasResponse) => string;
  getModelDisplayFilename: (model: AliasResponse) => string;
  actionsProps: Omit<ModelActionsProps, 'model'>;
}

const ModelTableRow = ({
  model,
  getItemId,
  getModelDisplayRepo,
  getModelDisplayFilename,
  actionsProps,
}: ModelTableRowProps) => [
  <TableCell key="combined" className="sm:hidden" data-testid={`combined-cell-${getItemId(model)}`}>
    <div className="flex flex-col gap-2">
      <CopyableContent text={isApiAlias(model) ? model.id : model.alias} className="font-medium" />
      {isApiAlias(model) && <div className="text-xs text-muted-foreground">Models: {model.models.join(', ')}</div>}

      <CopyableContent text={getModelDisplayRepo(model)} className="text-sm" />

      <CopyableContent text={getModelDisplayFilename(model)} className="text-xs text-muted-foreground" />

      <div className="w-fit">
        <SourceBadge model={model} testIdPrefix="m-" />
      </div>

      <div className="flex items-center gap-1 pt-2 border-t" data-testid={`actions-${getItemId(model)}`}>
        <ModelActions model={model} testIdPrefix="m-" {...actionsProps} />
      </div>
    </div>
  </TableCell>,
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
  <TableCell
    key="alias"
    className="max-w-[250px] hidden lg:table-cell"
    data-testid={`alias-cell-${getItemId(model)}`}
    data-model-id={isApiAlias(model) ? model.id : undefined}
    data-model-type={isApiAlias(model) ? 'api' : 'local'}
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
    key="prefix"
    className="max-w-[100px] hidden lg:table-cell"
    data-testid={`prefix-cell-${getItemId(model)}`}
  >
    {isApiAlias(model) ? (
      <CopyableContent text={model.prefix || '-'} className="text-sm" />
    ) : (
      <span className="text-muted-foreground">-</span>
    )}
  </TableCell>,
  <TableCell
    key="forward_all"
    className="max-w-[100px] hidden lg:table-cell"
    data-testid={`forward-all-cell-${getItemId(model)}`}
  >
    {isApiAlias(model) ? (
      model.forward_all_with_prefix ? (
        <Badge variant="outline" className="bg-green-500/10 text-green-600 border-green-200">
          Yes
        </Badge>
      ) : (
        <Badge variant="outline" className="bg-gray-500/10 text-gray-600 border-gray-200">
          No
        </Badge>
      )
    ) : (
      <span className="text-muted-foreground">-</span>
    )}
  </TableCell>,
  <TableCell
    key="actions"
    className="w-[140px] whitespace-nowrap hidden sm:table-cell"
    data-testid={`actions-cell-${getItemId(model)}`}
  >
    <ModelActions model={model} {...actionsProps} />
  </TableCell>,
];

export { ModelTableRow };
