import { useMemo, useState } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { AliasResponse, ModelRouterRequest, ModelRouterResponse } from '@bodhiapp/ts-client';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { ShellIcon, useShellChrome, type ShellSlots } from '@/components/shell';
import { useCreateModelRouter, useListModels, useUpdateModelRouter } from '@/hooks/models';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { isApiAlias } from '@/lib/utils';
import { ChainItem } from '@/routes/models/-components/RoutingChainPreview';
import { aliasIdentity, aliasLabel, AliasCombobox } from './AliasCombobox';
import { apiMatchableModels } from './RouteToModelField';
import { RouterInfoRail, RouterRailHeader, type RailStatus } from './RouterInfoRail';
import { StepCard, type TargetRow } from './StepCard';
import { StepConnector } from './StepConnector';
import './router-form.css';

interface ModelRouterFormProps {
  mode: 'create' | 'edit';
  initialData?: ModelRouterResponse;
  /** Breadcrumb published to the shell alongside the rail (one publisher per screen). */
  breadcrumb?: ShellSlots['breadcrumb'];
}

const ROUTE_MODELS = '/models/';

// Defaults mirror the persisted server defaults (FallbackConfig::default).
const DEFAULT_COOLDOWN_SECS = 30;
const DEFAULT_MAX_ATTEMPTS = 0; // 0 = try all enabled targets
const DEFAULT_HONOR_RETRY_AFTER = true;
const MAX_COOLDOWN_SECS = 3600; // soft upper guard (1 hour)

export default function ModelRouterForm({ mode, initialData, breadcrumb }: ModelRouterFormProps) {
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
  // Only one alias combobox is open at a time (clicking another trigger closes the prior popover).
  const [openComboboxIdx, setOpenComboboxIdx] = useState<number | null>(null);

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

  // ── Published rail (live preview). Display-only; never gates submit. ──
  const chain: ChainItem[] = useMemo(
    () =>
      targets.map((t) => {
        const a = t.alias ? aliasByIdentity.get(t.alias) : undefined;
        const needsModel = t.enabled && a !== undefined && isApiAlias(a);
        return {
          alias: a ? aliasLabel(a) : undefined,
          model: t.model || undefined,
          enabled: t.enabled,
          missingModel: needsModel && !t.model,
        };
      }),
    [targets, aliasByIdentity]
  );
  const railStatus: RailStatus | undefined = resilienceInvalid
    ? { tone: 'warn', message: 'Fix the resilience settings to continue.' }
    : undefined;

  // Memoize on the minimal derived inputs so the publish doesn't thrash on unrelated re-renders.
  const railKey = JSON.stringify({ chain, honorRetryAfter, alias, status: railStatus?.message });
  const rail = useMemo(
    () => <RouterInfoRail alias={alias} chain={chain} honorRetryAfter={honorRetryAfter} status={railStatus} />,
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [railKey]
  );
  const railHeader = useMemo(() => <RouterRailHeader />, []);
  // One publisher per screen: breadcrumb + rail together (a second useShellChrome would clobber).
  useShellChrome({ breadcrumb, rail, railHeader, railDefaultOpen: true });

  return (
    <div className="rf-card" data-testid="model-router-form">
      <div className="rf-card-head">
        <h1 className="rf-card-title">{mode === 'edit' ? 'Edit Model Router' : 'New Model Router'}</h1>
        <p className="rf-card-sub">Chain targets into a priority order and route requests through them.</p>
      </div>

      <div className="rf-card-body">
        {/* ── IDENTITY ── */}
        <section className="rf-section">
          <div className="rf-section-title">Identity</div>
          <div className="rf-field">
            <Label htmlFor="router-alias">
              Name <span className="rf-req">*</span>
            </Label>
            <Input
              id="router-alias"
              data-testid="router-alias-input"
              className="mono"
              value={alias}
              onChange={(e) => setAlias(e.target.value.replace(/\s/g, ''))}
              placeholder="e.g. smart-fallback"
            />
            <p className="rf-hint">
              Becomes the <code className="rf-code">model</code> value clients send to your API. No spaces.
            </p>
          </div>
          <div className="rf-field">
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
        </section>

        <div className="rf-divider" />

        {/* ── RESILIENCE ── */}
        <section className="rf-section" data-testid="resilience-settings">
          <div className="rf-section-title">Resilience</div>
          <div className="rf-field-row">
            <div className="rf-field">
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
              <p className="rf-hint">How long a failed target is skipped before it is retried.</p>
              {cooldownError && (
                <p className="rf-err-text" data-testid="cooldown-secs-error">
                  Enter a whole number between 0 and {MAX_COOLDOWN_SECS}.
                </p>
              )}
            </div>
            <div className="rf-field">
              <Label htmlFor="max-attempts">Max attempts per request</Label>
              <Input
                id="max-attempts"
                data-testid="max-attempts-input"
                type="number"
                min={0}
                value={maxAttempts}
                onChange={(e) => setMaxAttempts(e.target.valueAsNumber)}
              />
              <p className="rf-hint">0 = try all enabled targets.</p>
              {maxAttemptsError && (
                <p className="rf-err-text" data-testid="max-attempts-error">
                  Enter a whole number of 0 or more.
                </p>
              )}
            </div>
          </div>
          <div className="rf-toggle-row">
            <div className="rf-toggle-body">
              <div className="rf-toggle-label">Honor upstream Retry-After</div>
              <div className="rf-toggle-desc">
                When a target returns a <code className="rf-code">Retry-After</code> header, use it instead of the fixed
                cooldown above.
              </div>
            </div>
            <Switch
              data-testid="honor-retry-after-switch"
              checked={honorRetryAfter}
              onCheckedChange={setHonorRetryAfter}
              aria-label="Honor upstream Retry-After"
            />
          </div>
        </section>

        <div className="rf-divider" />

        {/* ── TARGETS ── */}
        <section className="rf-section">
          <div className="rf-section-title">Targets (in priority order)</div>

          {targets.length === 0 && (
            <p className="rf-hint" data-testid="no-targets">
              No targets yet. A router with no enabled targets returns an error at request time.
            </p>
          )}

          <div className="rf-steps">
            {targets.map((target, idx) => (
              <div key={idx}>
                <StepCard
                  target={target}
                  index={idx}
                  total={targets.length}
                  selected={target.alias ? aliasByIdentity.get(target.alias) : undefined}
                  options={referenceableAliases}
                  byIdentity={aliasByIdentity}
                  onSelectAlias={onSelectAlias}
                  onChangeModel={(i, m) => updateTarget(i, { model: m })}
                  onToggleEnabled={(i, enabled) => updateTarget(i, { enabled })}
                  onMove={moveTarget}
                  onRemove={removeTarget}
                  comboboxOpen={openComboboxIdx === idx}
                  onComboboxOpenChange={(open) => setOpenComboboxIdx(open ? idx : null)}
                />
                {idx < targets.length - 1 && <StepConnector testId={`step-connector-${idx}`} />}
              </div>
            ))}
          </div>

          <button type="button" className="rf-add-step" data-testid="add-target" onClick={addTarget}>
            <ShellIcon name="plus-circle" size={14} /> Add step
          </button>
        </section>
      </div>

      {/* ── FOOTER ── */}
      <div className="rf-footer">
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
          <ShellIcon name="route" size={13} /> {mode === 'edit' ? 'Save' : 'Create Model Router'}
        </Button>
      </div>
    </div>
  );
}

function errorMessage(error: unknown): string {
  const e = error as { response?: { data?: { error?: { message?: string } } }; message?: string };
  return e?.response?.data?.error?.message || e?.message || 'An unexpected error occurred';
}
