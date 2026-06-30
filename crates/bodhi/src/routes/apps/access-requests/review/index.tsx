import React, { useEffect, useMemo, useState } from 'react';

import type { UserScope } from '@bodhiapp/ts-client';
import { createFileRoute, useSearch } from '@tanstack/react-router';
import { AlertCircle, CheckCircle, Loader2, XCircle } from 'lucide-react';
import { z } from 'zod';

import { GrantBlock, type AccessMode } from '@/components/access-picker';
import AppInitializer from '@/components/AppInitializer';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader } from '@/components/ui/card';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Skeleton } from '@/components/ui/skeleton';
import { useGetAppAccessRequestReview, useApproveAppAccessRequest, useDenyAppAccessRequest } from '@/hooks/apps';
import type { AccessRequestActionResponse, ApproveAccessRequest } from '@/hooks/apps';
import { useListMcps } from '@/hooks/mcps';
import { useListModels } from '@/hooks/models';
import { toast } from '@/hooks/use-toast';
import { useGetUser } from '@/hooks/users';
import { useToastMessages } from '@/hooks/useToastMessages';
import { extractErrorMessage } from '@/lib/errorUtils';
import { grantableMcpItems, grantableModelItems } from '@/lib/grantItems';
import { safeNavigate } from '@/lib/safeNavigate';

import McpServerCard from './-components/McpServerCard';
import { toApproveBody } from './-shared/toApproveBody';
import '@/components/shell/api-keys.css';

export const Route = createFileRoute('/apps/access-requests/review/')({
  validateSearch: z.object({ id: z.string().optional() }),
  component: ReviewAccessRequestPage,
});

const NonDraftStatus = ({ status, flowType }: { status: string; flowType: string }) => {
  useEffect(() => {
    if (flowType === 'popup') {
      window.close();
    }
    // For redirect flow, the backend redirect_url is not available in the review response,
    // so we just show the status. The 3rd party app polls for status changes.
  }, [flowType]);

  const statusConfig: Record<
    string,
    { icon: React.ReactNode; label: string; variant: 'default' | 'secondary' | 'destructive' }
  > = {
    approved: { icon: <CheckCircle className="h-5 w-5 text-green-500" />, label: 'Approved', variant: 'default' },
    denied: { icon: <XCircle className="h-5 w-5 text-red-500" />, label: 'Denied', variant: 'destructive' },
    expired: { icon: <AlertCircle className="h-5 w-5 text-yellow-500" />, label: 'Expired', variant: 'secondary' },
  };

  const config = statusConfig[status] || {
    icon: <AlertCircle className="h-5 w-5" />,
    label: status,
    variant: 'secondary' as const,
  };

  return (
    <div
      className="flex min-h-[50vh] items-center justify-center"
      data-testid={`review-status-${status}`}
      data-test-status={status}
      data-test-flow-type={flowType}
    >
      <Card className="w-full max-w-md">
        <CardContent className="flex flex-col items-center gap-4 pt-6 pb-6">
          {config.icon}
          <div className="text-lg font-medium">Access Request {config.label}</div>
          <Badge variant={config.variant}>{config.label}</Badge>
          <p className="text-sm text-muted-foreground text-center">
            {status === 'approved' && 'This access request has already been approved.'}
            {status === 'denied' && 'This access request has been denied.'}
            {status === 'expired' && 'This access request has expired.'}
            {!['approved', 'denied', 'expired'].includes(status) && `Status: ${status}`}
          </p>
          {flowType === 'popup' && (
            <p className="text-xs text-muted-foreground">This window should close automatically.</p>
          )}
        </CardContent>
      </Card>
    </div>
  );
};

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

const ReviewContent = () => {
  const search = useSearch({ strict: false });
  const id = search.id as string | undefined;

  const { showError } = useToastMessages();
  const [selectedMcpInstances, setSelectedMcpInstances] = useState<Record<string, string>>({});
  const [approvedMcps, setApprovedMcps] = useState<Record<string, boolean>>({});
  const [approvedRole, setApprovedRole] = useState<UserScope | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [actionResult, setActionResult] = useState<AccessRequestActionResponse | null>(null);

  // Owner's model/MCP grant decisions (driven by the app's requested UI flags).
  // Both pickers default to least-privilege (Specific/none) — granting a 3rd-party
  // app is opt-in, matching the fail-closed backend default.
  const [listModels, setListModels] = useState(false);
  const [modelMode, setModelMode] = useState<AccessMode>('specific');
  const [models, setModels] = useState<string[]>([]);
  const [listMcps, setListMcps] = useState(false);
  const [mcpExtraMode, setMcpExtraMode] = useState<AccessMode>('specific');
  const [mcpsExtra, setMcpsExtra] = useState<string[]>([]);

  const { data: reviewData, isLoading, error } = useGetAppAccessRequestReview(id ?? null);
  const { data: userData } = useGetUser();
  const { data: modelsData } = useListModels(1, 100, 'alias', 'asc');
  const { data: mcpsData } = useListMcps();

  const modelItems = useMemo(() => grantableModelItems(modelsData?.data ?? []), [modelsData]);
  const mcpItems = useMemo(() => grantableMcpItems(mcpsData?.mcps ?? []), [mcpsData]);

  const toggleSelection = (current: string[], setter: (v: string[]) => void, itemId: string) => {
    setter(current.includes(itemId) ? current.filter((x) => x !== itemId) : [...current, itemId]);
  };

  const handleActionSuccess = (data: AccessRequestActionResponse) => {
    if (data.flow_type === 'popup') {
      window.close();
    } else if (data.flow_type === 'redirect' && data.redirect_url) {
      if (!safeNavigate(data.redirect_url)) {
        toast({
          title: 'Invalid redirect URL',
          description: `URL "${data.redirect_url}" must use http:// or https:// scheme`,
          variant: 'destructive',
        });
      }
    }
    setActionResult(data);
  };

  const approveMutation = useApproveAppAccessRequest({
    onSuccess: handleActionSuccess,
    onError: (message) => {
      setIsSubmitting(false);
      showError('Approval Failed', message);
    },
  });

  const denyMutation = useDenyAppAccessRequest({
    onSuccess: handleActionSuccess,
    onError: (message) => {
      setIsSubmitting(false);
      showError('Denial Failed', message);
    },
  });

  const roleOptions = useMemo(() => {
    if (!reviewData) return [];
    const userRole = userData?.auth_status === 'logged_in' ? (userData.role as string | null | undefined) : null;
    return computeRoleOptions(reviewData.requested_role, userRole);
  }, [reviewData, userData]);

  useEffect(() => {
    if (roleOptions.length > 0) {
      setApprovedRole(roleOptions[0].value as UserScope);
    }
  }, [roleOptions]);

  useEffect(() => {
    if (reviewData?.mcps_info) {
      const initial: Record<string, boolean> = {};
      reviewData.mcps_info.forEach((mcp) => {
        initial[mcp.url] = true;
      });
      setApprovedMcps(initial);
    }
  }, [reviewData]);

  const canApprove = useMemo(() => {
    if (!reviewData) return false;
    if (!approvedRole) return false;
    const mcpsValid = (reviewData.mcps_info ?? []).every((mcp) => {
      if (!approvedMcps[mcp.url]) return true;
      const validInstances = mcp.instances.filter((i) => i.enabled);
      if (validInstances.length === 0) return false;
      return !!selectedMcpInstances[mcp.url];
    });
    return mcpsValid;
  }, [reviewData, selectedMcpInstances, approvedMcps, approvedRole]);

  const approvedCount = useMemo(() => {
    const mcpsApproved = (reviewData?.mcps_info ?? []).filter((m) => approvedMcps[m.url]).length;
    return mcpsApproved;
  }, [reviewData, approvedMcps]);

  const totalCount = reviewData?.mcps_info?.length ?? 0;

  if (!id) {
    return <ErrorPage message="Missing access request ID" />;
  }

  // Show immediate result without waiting for refetch
  if (actionResult) {
    return <NonDraftStatus status={actionResult.status} flowType={actionResult.flow_type} />;
  }

  if (isLoading) {
    return (
      <div className="container mx-auto max-w-2xl p-4" data-testid="review-access-loading">
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

  if (error || !reviewData) {
    return (
      <div data-testid="review-access-error">
        <ErrorPage message={extractErrorMessage(error, 'Failed to load access request')} />
      </div>
    );
  }

  if (reviewData.status !== 'draft') {
    return <NonDraftStatus status={reviewData.status} flowType={reviewData.flow_type} />;
  }

  const req = reviewData.requested;

  const handleApprove = () => {
    setIsSubmitting(true);
    const body: ApproveAccessRequest = {
      approved_role: approvedRole!,
      approved: toApproveBody(req, reviewData.mcps_info ?? [], {
        listModels,
        modelMode,
        models,
        listMcps,
        mcpExtraMode,
        mcpsExtra,
        approvedMcps,
        selectedMcpInstances,
      }),
    };
    approveMutation.mutate({ id, body });
  };

  const handleDeny = () => {
    setIsSubmitting(true);
    denyMutation.mutate({ id });
  };

  const displayName = reviewData.app_name || reviewData.app_client_id;

  return (
    <div
      className="api-keys-screen container mx-auto max-w-2xl p-4"
      data-testid="review-access-page"
      data-test-status={reviewData.status}
      data-test-flow-type={reviewData.flow_type}
      data-test-state={modelsData && mcpsData ? 'ready' : 'loading'}
    >
      <div className="page-header">
        <div className="page-header-text">
          <div className="page-title">Review Access Request</div>
          <div className="page-subtitle">Decide which of your resources this 3rd-party app can use.</div>
        </div>
      </div>
      <Card>
        <CardHeader>
          <CardDescription>
            <span data-testid="review-app-name" className="font-medium">
              {displayName}
            </span>
            {' is requesting access to your resources.'}
          </CardDescription>
          {reviewData.app_description && (
            <p className="text-sm text-muted-foreground mt-1" data-testid="review-app-description">
              {reviewData.app_description}
            </p>
          )}
        </CardHeader>
        <CardContent>
          {(req.models_list || req.models_access) && (
            <section className="review-section" data-testid="review-models-section">
              <div className="review-section-title">AI Models</div>
              <GrantBlock
                noun="model"
                showListing={req.models_list}
                showAccess={req.models_access}
                listChecked={listModels}
                onListToggle={() => setListModels((v) => !v)}
                listLabel="Let the app see your full model list"
                listCode="/v1/models"
                listDescription="The app can see the names of all your models. It still can't use a model unless you allow it below."
                listTestId="review-list-models-toggle"
                mode={modelMode}
                onModeChange={setModelMode}
                items={modelItems}
                selectedIds={models}
                onToggle={(itemId) => toggleSelection(models, setModels, itemId)}
                panelTitle="Select Models"
                panelSubtitle="Choose which models this app can use"
                testIdPrefix="review-model-access"
                disabled={isSubmitting}
              />
            </section>
          )}

          {(req.mcps_list || req.mcps_access || (reviewData.mcps_info?.length ?? 0) > 0) && (
            <section className="review-section" data-testid="review-mcps-section">
              <div className="review-section-title">Connected Tools</div>
              {(req.mcps_list || req.mcps_access) && (
                <GrantBlock
                  noun="tool"
                  showListing={req.mcps_list}
                  showAccess={req.mcps_access}
                  listChecked={listMcps}
                  onListToggle={() => setListMcps((v) => !v)}
                  listLabel="Let the app see your full list of tools"
                  listCode="/v1/mcps"
                  listDescription="The app can see the names of all your connected tools. It still can't use a tool unless you allow it below."
                  listTestId="review-list-mcps-toggle"
                  mode={mcpExtraMode}
                  onModeChange={setMcpExtraMode}
                  items={mcpItems}
                  selectedIds={mcpsExtra}
                  onToggle={(itemId) => toggleSelection(mcpsExtra, setMcpsExtra, itemId)}
                  panelTitle="Give the app extra tools"
                  panelSubtitle="Tools the app didn't ask for, but you can add."
                  allLabel="All tools"
                  allDesc="Give access to every connected tool, including ones added later."
                  specificLabel="Specific tools"
                  specificDesc="Choose exactly which tools the app can use."
                  testIdPrefix="review-mcp-access"
                  disabled={isSubmitting}
                />
              )}
              {reviewData.mcps_info?.map((mcpInfo) => (
                <McpServerCard
                  key={mcpInfo.url}
                  mcpInfo={mcpInfo}
                  selectedInstance={selectedMcpInstances[mcpInfo.url]}
                  isApproved={approvedMcps[mcpInfo.url] ?? true}
                  onSelectInstance={(url, instanceId) =>
                    setSelectedMcpInstances((prev) => ({ ...prev, [url]: instanceId }))
                  }
                  onToggleApproval={(url, approved) => setApprovedMcps((prev) => ({ ...prev, [url]: approved }))}
                />
              ))}
            </section>
          )}

          {roleOptions.length > 0 && (
            <div className="mb-4" data-testid="review-approved-role-section">
              <Label className="text-sm font-medium mb-1 block">Approved Role</Label>
              <Select value={approvedRole ?? ''} onValueChange={(v) => setApprovedRole(v as UserScope)}>
                <SelectTrigger data-testid="review-approved-role-select">
                  <SelectValue placeholder="Select role" />
                </SelectTrigger>
                <SelectContent>
                  {roleOptions.map((opt) => (
                    <SelectItem
                      key={opt.value}
                      value={opt.value}
                      data-testid={`review-approved-role-option-${opt.value}`}
                    >
                      {opt.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          )}

          <div className="flex justify-between gap-4">
            <Button variant="outline" onClick={handleDeny} disabled={isSubmitting} data-testid="review-deny-button">
              {denyMutation.isPending ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Denying...
                </>
              ) : (
                'Deny'
              )}
            </Button>
            <Button onClick={handleApprove} disabled={!canApprove || isSubmitting} data-testid="review-approve-button">
              {approveMutation.isPending ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Approving...
                </>
              ) : approvedCount === totalCount ? (
                'Approve All'
              ) : (
                'Approve Selected'
              )}
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default function ReviewAccessRequestPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <ReviewContent />
    </AppInitializer>
  );
}
