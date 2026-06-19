import { AliasResponse, ApiAliasResponse } from '@bodhiapp/ts-client';

import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { isApiAlias } from '@/lib/utils';
import { formatPrefixedModel, getApiModelId } from '@/schemas/apiModel';

/** Prefixed, selectable model ids offered by a selected-subset API alias. */
export function apiMatchableModels(api: ApiAliasResponse): string[] {
  return (api.models || []).map((m) => formatPrefixedModel(getApiModelId(m, api.prefix), api.prefix));
}

interface RouteToModelFieldProps {
  alias?: AliasResponse;
  value: string;
  onChange: (model: string) => void;
  testId: string;
}

/**
 * The per-step model field. Local aliases pin their own model (read-only). API aliases route to
 * a model: a constrained dropdown in selected-subset mode, free-text in forward-all mode. Keeps
 * `target-model-${idx}` on whichever variant renders.
 */
export function RouteToModelField({ alias, value, onChange, testId }: RouteToModelFieldProps) {
  if (!alias) {
    return <Input disabled value="" placeholder="Select an alias first" />;
  }

  if (!isApiAlias(alias)) {
    // Local user/model alias: model is fixed to the alias's own name, read-only.
    return <Input data-testid={testId} value={value} disabled />;
  }

  const freeText = alias.forward_all_with_prefix === true;
  if (freeText) {
    return (
      <div className="rf-model-field">
        <div className="rf-sub-label">
          Route to model
          <span className="rf-model-pill rf-model-pill-all">any model · forward-all</span>
        </div>
        <Input
          data-testid={testId}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder={`${alias.prefix ?? ''}model-name`}
        />
      </div>
    );
  }

  const modelOptions = apiMatchableModels(alias);
  return (
    <div className="rf-model-field">
      <div className="rf-sub-label">
        Route to model
        <span className="rf-model-pill rf-model-pill-selected">pre-configured only</span>
      </div>
      <Select value={value} onValueChange={onChange}>
        <SelectTrigger data-testid={testId}>
          <SelectValue placeholder="Select a model" />
        </SelectTrigger>
        <SelectContent>
          {modelOptions.map((m) => (
            <SelectItem key={m} value={m}>
              {m}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}
