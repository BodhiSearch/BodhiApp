import { useMemo, useState } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { AliasResponse, ApiAliasResponse, ModelRouterRequest, ModelRouterResponse } from '@bodhiapp/ts-client';

import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { useCreateModelRouter, useListModels, useUpdateModelRouter } from '@/hooks/models';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { isApiAlias } from '@/lib/utils';
import { formatPrefixedModel, getApiModelId } from '@/schemas/apiModel';

interface TargetRow {
  alias: string;
  model: string;
  enabled: boolean;
}

interface ModelRouterFormProps {
  mode: 'create' | 'edit';
  initialData?: ModelRouterResponse;
}

const ROUTE_MODELS = '/models/';

// Defaults mirror the persisted server defaults (FallbackConfig::default).
const DEFAULT_COOLDOWN_SECS = 30;
const DEFAULT_MAX_ATTEMPTS = 0; // 0 = try all enabled targets
const DEFAULT_HONOR_RETRY_AFTER = true;
const MAX_COOLDOWN_SECS = 3600; // soft upper guard (1 hour)

/** Identity used to reference an alias from a target: id for api, name for local. */
function aliasIdentity(alias: AliasResponse): string {
  return isApiAlias(alias) ? alias.id : alias.alias;
}

/** Prefixed, selectable model ids offered by a selected-subset API alias. */
function apiMatchableModels(api: ApiAliasResponse): string[] {
  return (api.models || []).map((m) => formatPrefixedModel(getApiModelId(m, api.prefix), api.prefix));
}

export default function ModelRouterForm({ mode, initialData }: ModelRouterFormProps) {
  const navigate = useNavigate();
  const { showSuccess, showError } = useToastMessages();

  const { data: aliasPage } = useListModels(1, 100, 'name', 'asc');
  // Selectable referenced aliases: everything except other routers.
  const referenceableAliases: AliasResponse[] = useMemo(
    () => (aliasPage?.data || []).filter((a) => a.source !== 'model_router'),
    [aliasPage]
  );
  const aliasByIdentity = useMemo(() => {
    const map = new Map<string, AliasResponse>();
    referenceableAliases.forEach((a) => map.set(aliasIdentity(a), a));
    return map;
  }, [referenceableAliases]);

  const [alias, setAlias] = useState(initialData?.alias ?? '');
  const [targets, setTargets] = useState<TargetRow[]>(
    (initialData?.targets ?? []).map((t) => ({ alias: t.alias, model: t.model, enabled: t.enabled ?? true }))
  );

  // Resilience knobs (persisted since Phase 1; surfaced here in Phase 3).
  const initialStrategy = initialData?.strategy;
  const [cooldownSecs, setCooldownSecs] = useState<number>(initialStrategy?.cooldown_secs ?? DEFAULT_COOLDOWN_SECS);
  const [maxAttempts, setMaxAttempts] = useState<number>(initialStrategy?.max_attempts ?? DEFAULT_MAX_ATTEMPTS);
  const [honorRetryAfter, setHonorRetryAfter] = useState<boolean>(
    initialStrategy?.honor_retry_after ?? DEFAULT_HONOR_RETRY_AFTER
  );

  const cooldownError =
    Number.isNaN(cooldownSecs) ||
    !Number.isInteger(cooldownSecs) ||
    cooldownSecs < 0 ||
    cooldownSecs > MAX_COOLDOWN_SECS;
  const maxAttemptsError = Number.isNaN(maxAttempts) || !Number.isInteger(maxAttempts) || maxAttempts < 0;
  const resilienceInvalid = cooldownError || maxAttemptsError;

  const createMutation = useCreateModelRouter({
    onSuccess: () => {
      showSuccess('Model router created', `Router '${alias}' was created.`);
      navigate({ to: ROUTE_MODELS });
    },
    onError: (error) => showError('Create failed', errorMessage(error)),
  });
  const updateMutation = useUpdateModelRouter({
    onSuccess: () => {
      showSuccess('Model router updated', `Router '${alias}' was updated.`);
      navigate({ to: ROUTE_MODELS });
    },
    onError: (error) => showError('Update failed', errorMessage(error)),
  });

  const updateTarget = (idx: number, patch: Partial<TargetRow>) => {
    setTargets((prev) => prev.map((t, i) => (i === idx ? { ...t, ...patch } : t)));
  };

  const onSelectAlias = (idx: number, identity: string) => {
    const selected = aliasByIdentity.get(identity);
    // Default the pinned model based on the referenced alias's shape.
    let model = '';
    if (selected) {
      if (!isApiAlias(selected)) {
        model = selected.alias; // local: fixed to its own model
      } else if (!selected.forward_all_with_prefix) {
        model = apiMatchableModels(selected)[0] ?? '';
      }
    }
    updateTarget(idx, { alias: identity, model });
  };

  const addTarget = () => setTargets((prev) => [...prev, { alias: '', model: '', enabled: true }]);
  const removeTarget = (idx: number) => setTargets((prev) => prev.filter((_, i) => i !== idx));
  const moveTarget = (idx: number, dir: -1 | 1) => {
    setTargets((prev) => {
      const next = [...prev];
      const j = idx + dir;
      if (j < 0 || j >= next.length) return prev;
      [next[idx], next[j]] = [next[j], next[idx]];
      return next;
    });
  };

  const handleSubmit = () => {
    if (resilienceInvalid) return;
    const body: ModelRouterRequest = {
      alias,
      targets: targets.map((t) => ({ alias: t.alias, model: t.model, enabled: t.enabled })),
      strategy: {
        strategy: 'fallback',
        cooldown_secs: cooldownSecs,
        max_attempts: maxAttempts,
        honor_retry_after: honorRetryAfter,
      },
    };
    if (mode === 'edit' && initialData) {
      updateMutation.mutate({ id: initialData.id, data: body });
    } else {
      createMutation.mutate(body);
    }
  };

  const submitting = createMutation.isPending || updateMutation.isPending;

  return (
    <Card data-testid="model-router-form" className="max-w-3xl mx-auto my-6">
      <CardHeader>
        <CardTitle>{mode === 'edit' ? 'Edit Model Router' : 'New Model Router'}</CardTitle>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="space-y-2">
          <Label htmlFor="router-alias">Name</Label>
          <Input
            id="router-alias"
            data-testid="router-alias-input"
            value={alias}
            onChange={(e) => setAlias(e.target.value)}
            placeholder="e.g. my-stack"
          />
        </div>

        <div className="space-y-2">
          <Label>Strategy</Label>
          {/* Only fallback in v1; the selector exists so future strategies appear here. */}
          <Select value="fallback" disabled>
            <SelectTrigger data-testid="router-strategy-select">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="fallback">Fallback</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-3" data-testid="resilience-settings">
          <Label>Resilience</Label>
          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-1">
              <Label htmlFor="cooldown-secs">Cooldown (seconds)</Label>
              <Input
                id="cooldown-secs"
                data-testid="cooldown-secs-input"
                type="number"
                min={0}
                max={MAX_COOLDOWN_SECS}
                value={cooldownSecs}
                onChange={(e) => setCooldownSecs(e.target.valueAsNumber)}
              />
              <p className="text-xs text-muted-foreground">How long a failed target is skipped before it is retried.</p>
              {cooldownError && (
                <p className="text-xs text-destructive" data-testid="cooldown-secs-error">
                  Enter a whole number between 0 and {MAX_COOLDOWN_SECS}.
                </p>
              )}
            </div>

            <div className="space-y-1">
              <Label htmlFor="max-attempts">Max attempts per request</Label>
              <Input
                id="max-attempts"
                data-testid="max-attempts-input"
                type="number"
                min={0}
                value={maxAttempts}
                onChange={(e) => setMaxAttempts(e.target.valueAsNumber)}
              />
              <p className="text-xs text-muted-foreground">0 = try all enabled targets.</p>
              {maxAttemptsError && (
                <p className="text-xs text-destructive" data-testid="max-attempts-error">
                  Enter a whole number of 0 or more.
                </p>
              )}
            </div>
          </div>

          <div className="flex items-center gap-2">
            <Switch
              data-testid="honor-retry-after-switch"
              checked={honorRetryAfter}
              onCheckedChange={setHonorRetryAfter}
            />
            <span className="text-sm">Honor upstream Retry-After</span>
          </div>
        </div>

        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <Label>Targets (in priority order)</Label>
            <Button type="button" variant="outline" size="sm" data-testid="add-target" onClick={addTarget}>
              Add target
            </Button>
          </div>

          {targets.length === 0 && (
            <p className="text-sm text-muted-foreground" data-testid="no-targets">
              No targets yet. A router with no enabled targets returns an error at request time.
            </p>
          )}

          {targets.map((target, idx) => {
            const selected = aliasByIdentity.get(target.alias);
            const isApi = selected ? isApiAlias(selected) : false;
            const apiSelected = isApi ? (selected as ApiAliasResponse) : undefined;
            const freeText = apiSelected?.forward_all_with_prefix === true;
            const modelOptions = apiSelected && !freeText ? apiMatchableModels(apiSelected) : [];
            return (
              <div key={idx} data-testid={`target-row-${idx}`} className="border rounded-md p-3 space-y-3">
                <div className="grid grid-cols-2 gap-3">
                  <div className="space-y-1">
                    <Label>Alias</Label>
                    <Select value={target.alias} onValueChange={(v) => onSelectAlias(idx, v)}>
                      <SelectTrigger data-testid={`target-alias-${idx}`}>
                        <SelectValue placeholder="Select an alias" />
                      </SelectTrigger>
                      <SelectContent>
                        {referenceableAliases.map((a) => {
                          const id = aliasIdentity(a);
                          return (
                            <SelectItem key={id} value={id}>
                              {id}
                            </SelectItem>
                          );
                        })}
                      </SelectContent>
                    </Select>
                  </div>

                  <div className="space-y-1">
                    <Label>Model</Label>
                    {!selected && <Input disabled value="" placeholder="Select an alias first" />}
                    {selected && !isApi && <Input data-testid={`target-model-${idx}`} value={target.model} disabled />}
                    {selected && isApi && freeText && (
                      <Input
                        data-testid={`target-model-${idx}`}
                        value={target.model}
                        onChange={(e) => updateTarget(idx, { model: e.target.value })}
                        placeholder={`${apiSelected?.prefix ?? ''}model-name`}
                      />
                    )}
                    {selected && isApi && !freeText && (
                      <Select value={target.model} onValueChange={(v) => updateTarget(idx, { model: v })}>
                        <SelectTrigger data-testid={`target-model-${idx}`}>
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
                    )}
                  </div>
                </div>

                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Switch
                      data-testid={`target-enabled-${idx}`}
                      checked={target.enabled}
                      onCheckedChange={(checked) => updateTarget(idx, { enabled: checked })}
                    />
                    <span className="text-sm">{target.enabled ? 'Enabled' : 'Disabled'}</span>
                  </div>
                  <div className="flex gap-2">
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      data-testid={`target-up-${idx}`}
                      disabled={idx === 0}
                      onClick={() => moveTarget(idx, -1)}
                    >
                      Up
                    </Button>
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      data-testid={`target-down-${idx}`}
                      disabled={idx === targets.length - 1}
                      onClick={() => moveTarget(idx, 1)}
                    >
                      Down
                    </Button>
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      data-testid={`target-remove-${idx}`}
                      onClick={() => removeTarget(idx)}
                    >
                      Remove
                    </Button>
                  </div>
                </div>
              </div>
            );
          })}
        </div>

        <div className="flex justify-end gap-2">
          <Button
            type="button"
            variant="outline"
            data-testid="router-cancel"
            onClick={() => navigate({ to: ROUTE_MODELS })}
          >
            Cancel
          </Button>
          <Button
            type="button"
            data-testid="router-submit"
            disabled={submitting || resilienceInvalid}
            onClick={handleSubmit}
          >
            {mode === 'edit' ? 'Save' : 'Create'}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}

function errorMessage(error: unknown): string {
  const e = error as { response?: { data?: { error?: { message?: string } } }; message?: string };
  return e?.response?.data?.error?.message || e?.message || 'An unexpected error occurred';
}
