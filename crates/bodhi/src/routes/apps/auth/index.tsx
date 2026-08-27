import { useCallback, useEffect, useMemo, useState } from 'react';

import type { ConsentPriorGrant, ConsentScopeInfo, UserScope } from '@bodhiapp/ts-client';
import { createFileRoute } from '@tanstack/react-router';
import { Loader2 } from 'lucide-react';
import { z } from 'zod';

import { GrantBlock, type AccessMode } from '@/components/access-picker';
import AppInitializer from '@/components/AppInitializer';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader } from '@/components/ui/card';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Skeleton } from '@/components/ui/skeleton';
import { useGetConsentContext, useSubmitConsent } from '@/hooks/apps';
import { useListMcps } from '@/hooks/mcps';
import { useListModels } from '@/hooks/models';
import { useGetUser } from '@/hooks/users';
import { useToastMessages } from '@/hooks/useToastMessages';
import { extractErrorMessage } from '@/lib/errorUtils';
import { grantableMcpItems, grantableModelItems } from '@/lib/grantItems';
import { safeNavigate } from '@/lib/safeNavigate';

import { previousGrantToState } from './-shared/previousGrantToState';
import { toApproveBody } from './-shared/toApproveBody';
import '@/components/shell/api-keys.css';

export const Route = createFileRoute('/apps/auth/')({
  // The backend re-parses and validates the raw query string; the page passes it through.
  validateSearch: z.object({}).passthrough(),
  component: AppsAuthPage,
});

const Redirecting = () => (
  <div className="flex min-h-[50vh] items-center justify-center" data-testid="consent-redirecting">
    <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
  </div>
);

const SCOPE_ORDER = ['scope_user_power_user', 'scope_user_user'] as const;
type UserScopeValue = (typeof SCOPE_ORDER)[number];

const SCOPE_LABELS: Record<UserScopeValue, string> = {
  scope_user_power_user: 'Power User',
  scope_user_user: 'User',
};

function computeRoleOptions(
  requestedRole: string,
  userRole: string | null | undefined
): { value: string; label: string }[] {
  const requestedIndex = SCOPE_ORDER.indexOf(requestedRole as UserScopeValue);
  if (requestedIndex === -1) return [];

  // resource_power_user, resource_manager, resource_admin can grant scope_user_power_user
  const maxGrantable =
    userRole && ['resource_power_user', 'resource_manager', 'resource_admin'].includes(userRole)
      ? 'scope_user_power_user'
      : 'scope_user_user';
  const maxGrantableIndex = SCOPE_ORDER.indexOf(maxGrantable as UserScopeValue);

  // Higher index in SCOPE_ORDER = lower/more-restrictive scope; cap at min(requested, maxGrantable)
  const startIndex = Math.max(requestedIndex, maxGrantableIndex);
  return SCOPE_ORDER.slice(startIndex).map((scope) => ({
    value: scope,
    label: SCOPE_LABELS[scope],
  }));
}

const AppIdentity = ({ name, description }: { name: string; description: string }) => (
  <CardHeader>
    <CardDescription>
      <span data-testid="consent-app-name" className="font-medium">
        {name}
      </span>
      {' is requesting access to your resources.'}
    </CardDescription>
    {description && (
      <p className="text-sm text-muted-foreground mt-1" data-testid="consent-app-description">
        {description}
      </p>
    )}
  </CardHeader>
);

const ConsentContent = () => {
  // Raw, already-encoded query string — the backend is the validator and re-parses it.
  const rawSearch = window.location.search;
  const query = rawSearch.startsWith('?') ? rawSearch.slice(1) : rawSearch;

  const { showError } = useToastMessages();
  const [approvedRole, setApprovedRole] = useState<UserScope | null>(null);
  const [pendingDecision, setPendingDecision] = useState<'approve' | 'deny' | null>(null);
  const isSubmitting = pendingDecision !== null;
  const [redirecting, setRedirecting] = useState(false);
  const [restored, setRestored] = useState(false);

  // Owner's model/MCP grant decisions; both pickers default to least-privilege (Specific/none) —
  // granting a 3rd-party app is opt-in, matching the fail-closed backend default.
  const [listModels, setListModels] = useState(false);
  const [modelMode, setModelMode] = useState<AccessMode>('specific');
  const [models, setModels] = useState<string[]>([]);
  const [listMcps, setListMcps] = useState(false);
  const [mcpExtraMode, setMcpExtraMode] = useState<AccessMode>('specific');
  const [mcpsExtra, setMcpsExtra] = useState<string[]>([]);

  const { data: consent, isLoading, error } = useGetConsentContext(rawSearch);
  const ok = consent?.result === 'ok' ? consent : undefined;

  const { data: userData } = useGetUser();
  // Picker candidates are only needed for sections the form actually renders
  // (never for blocked/role-only states).
  const needsModels = !!ok && ok.can_approve && ok.scope.llms;
  const needsMcps = !!ok && ok.can_approve && ok.scope.mcps;
  const { data: modelsData } = useListModels(1, 100, 'alias', 'asc', undefined, { enabled: needsModels });
  const { data: mcpsData } = useListMcps({ enabled: needsMcps });

  const consentReady = !!ok && (!needsModels || !!modelsData) && (!needsMcps || !!mcpsData);

  const modelItems = useMemo(() => grantableModelItems(modelsData?.data ?? []), [modelsData]);
  const mcpItems = useMemo(() => grantableMcpItems(mcpsData?.mcps ?? []), [mcpsData]);

  const toggleSelection = (current: string[], setter: (v: string[]) => void, itemId: string) => {
    setter(current.includes(itemId) ? current.filter((x) => x !== itemId) : [...current, itemId]);
  };

  const submitMutation = useSubmitConsent({
    onSuccess: (data) => {
      setRedirecting(true);
      safeNavigate(data.redirect_url);
    },
    onError: (message) => {
      setPendingDecision(null);
      showError('Consent Failed', message);
    },
  });

  const roleOptions = useMemo(() => {
    if (!ok) return [];
    const userRole = userData?.auth_status === 'logged_in' ? (userData.role as string | null | undefined) : null;
    return computeRoleOptions(ok.scope.role, userRole);
  }, [ok, userData]);

  // Default to the highest grantable role (the requested ceiling, capped by the approver's role).
  useEffect(() => {
    if (roleOptions.length > 0) {
      setApprovedRole(roleOptions[0].value as UserScope);
    }
  }, [roleOptions]);

  // Prefill only touches rendered sections — scope-suppressed sections carry nothing forward.
  const applyPriorGrant = useCallback((prior: ConsentPriorGrant, scope: ConsentScopeInfo) => {
    const s = previousGrantToState(prior);
    if (scope.llms) {
      setListModels(s.listModels);
      setModelMode(s.modelMode);
      setModels(s.models);
    }
    if (scope.mcps) {
      setListMcps(s.listMcps);
      setMcpExtraMode(s.mcpExtraMode);
      setMcpsExtra(s.mcpsExtra);
    }
  }, []);

  // Explicit reauthorization prefills immediately; a 'latest' prior grant waits for Restore.
  useEffect(() => {
    if (ok?.prior_grant?.source === 'explicit') {
      applyPriorGrant(ok.prior_grant, ok.scope);
    }
  }, [ok, applyPriorGrant]);

  const errorRedirectUrl = consent?.result === 'error' ? (consent.redirect_url ?? undefined) : undefined;

  useEffect(() => {
    if (errorRedirectUrl && !redirecting) {
      setRedirecting(true);
      safeNavigate(errorRedirectUrl);
    }
  }, [errorRedirectUrl, redirecting]);

  const handleDeny = () => {
    setPendingDecision('deny');
    submitMutation.mutate({ query, decision: 'deny' });
  };

  if (redirecting || errorRedirectUrl) {
    return <Redirecting />;
  }

  if (isLoading) {
    return (
      <div className="container mx-auto max-w-2xl p-4" data-testid="consent-loading">
        <Card>
          <CardHeader>
            <Skeleton className="h-6 w-48" />
            <Skeleton className="h-4 w-64 mt-2" />
          </CardHeader>
          <CardContent>
            <Skeleton className="h-24 w-full" />
            <Skeleton className="h-24 w-full mt-4" />
          </CardContent>
        </Card>
      </div>
    );
  }

  if (error || !consent) {
    return (
      <div data-testid="consent-error">
        <ErrorPage message={extractErrorMessage(error, 'Failed to load the authorization request')} />
      </div>
    );
  }

  if (consent.result === 'error') {
    // No redirect_url means the error must render in-app (e.g. redirect_uri mismatch).
    return (
      <div data-testid="consent-error">
        <ErrorPage message={consent.error_description} />
      </div>
    );
  }

  const { app, scope, prior_grant: priorGrant, can_approve: canApprove } = consent;

  if (!canApprove) {
    return (
      <div
        className="api-keys-screen container mx-auto max-w-2xl p-4"
        data-testid="consent-blocked"
        data-test-state={consentReady ? 'ready' : 'loading'}
      >
        <Card>
          <AppIdentity name={app.name} description={app.description} />
          <CardContent>
            <p className="text-sm text-muted-foreground mb-4">
              You don&apos;t have access to this Bodhi instance yet. Ask an administrator for access, then try
              connecting again.
            </p>
            <Button variant="outline" onClick={handleDeny} disabled={isSubmitting} data-testid="consent-return-button">
              {submitMutation.isPending ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Returning...
                </>
              ) : (
                `Return to ${app.name}`
              )}
            </Button>
          </CardContent>
        </Card>
      </div>
    );
  }

  const roleOnly = !scope.llms && !scope.mcps;
  const roleLabel = SCOPE_LABELS[(approvedRole ?? scope.role) as UserScopeValue];

  const handleApprove = () => {
    setPendingDecision('approve');
    submitMutation.mutate({
      query,
      decision: 'approve',
      approved_role: approvedRole!,
      approved: toApproveBody(
        {
          version: '1',
          models_list: scope.llms,
          models_access: scope.llms,
          mcps_list: scope.mcps,
          mcps_access: scope.mcps,
        },
        { listModels, modelMode, models, listMcps, mcpExtraMode, mcpsExtra }
      ),
    });
  };

  return (
    <div
      className="api-keys-screen container mx-auto max-w-2xl p-4"
      data-testid="consent-page"
      data-test-state={consentReady ? 'ready' : 'loading'}
    >
      <div className="page-header">
        <div className="page-header-text">
          <div className="page-title">Review Access Request</div>
          <div className="page-subtitle">Decide which of your resources this 3rd-party app can use.</div>
        </div>
      </div>
      <Card>
        <AppIdentity name={app.name} description={app.description} />
        <CardContent>
          {priorGrant?.source === 'explicit' && (
            <p className="text-sm text-muted-foreground mb-4" data-testid="consent-reauth-banner">
              Reauthorizing an existing grant — your previous selections are pre-filled.
            </p>
          )}
          {priorGrant?.source === 'latest' && (
            <div className="mb-4 flex items-center justify-between gap-3" data-testid="consent-restore-banner">
              <p className="text-sm text-muted-foreground">You previously granted this app access.</p>
              {!restored && (
                <Button
                  variant="outline"
                  size="sm"
                  data-testid="consent-restore-button"
                  onClick={() => {
                    applyPriorGrant(priorGrant, scope);
                    setRestored(true);
                  }}
                >
                  Restore previous selections
                </Button>
              )}
            </div>
          )}

          {scope.llms && (
            <section className="review-section" data-testid="consent-models-section">
              <div className="review-section-title">AI Models</div>
              <GrantBlock
                noun="model"
                listChecked={listModels}
                onListToggle={() => setListModels((v) => !v)}
                listLabel="Let the app see your full model list"
                listCode="/v1/models"
                listDescription="The app can see the names of all your models. It still can't use a model unless you allow it below."
                listTestId="consent-list-models-toggle"
                mode={modelMode}
                onModeChange={setModelMode}
                items={modelItems}
                selectedIds={models}
                onToggle={(itemId) => toggleSelection(models, setModels, itemId)}
                panelTitle="Select Models"
                panelSubtitle="Choose which models this app can use"
                testIdPrefix="consent-model-access"
                disabled={isSubmitting}
              />
            </section>
          )}

          {scope.mcps && (
            <section className="review-section" data-testid="consent-mcps-section">
              <div className="review-section-title">Connected Tools</div>
              <GrantBlock
                noun="tool"
                listChecked={listMcps}
                onListToggle={() => setListMcps((v) => !v)}
                listLabel="Let the app see your full list of tools"
                listCode="/v1/mcps"
                listDescription="The app can see the names of all your connected tools. It still can't use a tool unless you allow it below."
                listTestId="consent-list-mcps-toggle"
                mode={mcpExtraMode}
                onModeChange={setMcpExtraMode}
                items={mcpItems}
                selectedIds={mcpsExtra}
                onToggle={(itemId) => toggleSelection(mcpsExtra, setMcpsExtra, itemId)}
                panelTitle="Select Tools"
                panelSubtitle="Choose which tools this app can use"
                allLabel="All tools"
                allDesc="Give access to every connected tool, including ones added later."
                specificLabel="Specific tools"
                specificDesc="Choose exactly which tools the app can use."
                testIdPrefix="consent-mcp-access"
                disabled={isSubmitting}
              />
            </section>
          )}

          {roleOnly && (
            <p className="text-sm mb-4" data-testid="consent-role-only-summary">
              {`${app.name} will be able to access the Bodhi APIs as ${roleLabel} — no model or tool access.`}
            </p>
          )}

          <div className="mb-4" data-testid="consent-approved-role-section">
            <Label className="text-sm font-medium mb-1 block">Approved Role</Label>
            {scope.role === 'scope_user_user' ? (
              <p className="text-sm" data-testid="consent-approved-role-fixed">
                {SCOPE_LABELS.scope_user_user}
              </p>
            ) : (
              <Select value={approvedRole ?? ''} onValueChange={(v) => setApprovedRole(v as UserScope)}>
                <SelectTrigger data-testid="consent-approved-role-select">
                  <SelectValue placeholder="Select role" />
                </SelectTrigger>
                <SelectContent>
                  {roleOptions.map((opt) => (
                    <SelectItem
                      key={opt.value}
                      value={opt.value}
                      data-testid={`consent-approved-role-option-${opt.value}`}
                    >
                      {opt.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            )}
          </div>

          <div className="flex justify-between gap-4">
            <Button variant="outline" onClick={handleDeny} disabled={isSubmitting} data-testid="consent-deny-button">
              {pendingDecision === 'deny' ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Denying...
                </>
              ) : (
                'Deny'
              )}
            </Button>
            <Button
              onClick={handleApprove}
              disabled={!approvedRole || isSubmitting}
              data-testid="consent-approve-button"
            >
              {pendingDecision === 'approve' ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Approving...
                </>
              ) : (
                'Approve'
              )}
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default function AppsAuthPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true} skipRoleGate>
      <ConsentContent />
    </AppInitializer>
  );
}
