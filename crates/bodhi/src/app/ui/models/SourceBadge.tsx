import { AliasResponse } from '@bodhiapp/ts-client';

import { Badge } from '@/components/ui/badge';
import { Cloud } from 'lucide-react';

import { isApiAlias } from '@/lib/utils';

const SourceBadge = ({ model, testIdPrefix = '' }: { model: AliasResponse; testIdPrefix?: string }) => {
  const prefix = testIdPrefix ? `${testIdPrefix}` : '';

  if (isApiAlias(model)) {
    return (
      <Badge
        variant="outline"
        className="bg-purple-500/10 text-purple-600 border-purple-200"
        data-testid={`${prefix}source-badge-${model.id}`}
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

export { SourceBadge };
