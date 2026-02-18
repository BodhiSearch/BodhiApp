'use client';

import React, { useEffect, useMemo, useState } from 'react';

import { AlertCircle, CheckCircle, Loader2, XCircle } from 'lucide-react';
import { useSearchParams } from 'next/navigation';

import AppInitializer from '@/components/AppInitializer';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Skeleton } from '@/components/ui/skeleton';
import { useToastMessages } from '@/hooks/use-toast-messages';
import {
  useAppAccessRequestReview,
  useApproveAppAccessRequest,
  useDenyAppAccessRequest,
} from '@/hooks/useAppAccessRequests';
import type {
  AccessRequestActionResponse,
  ApproveAccessRequestBody,
  McpServerReviewInfo,
  ToolTypeReviewInfo,
} from '@/hooks/useAppAccessRequests';

// ============================================================================
// Tool Type Card Component
// ============================================================================

const ToolTypeCard = ({
  toolInfo,
  selectedInstance,
  isApproved,
  onSelectInstance,
  onToggleApproval,
}: {
  toolInfo: ToolTypeReviewInfo;
  selectedInstance: string | undefined;
  isApproved: boolean;
  onSelectInstance: (toolType: string, instanceId: string) => void;
  onToggleApproval: (toolType: string, approved: boolean) => void;
}) => {
  const hasInstances = toolInfo.instances.length > 0;
  const validInstances = toolInfo.instances.filter((i) => i.enabled && i.has_api_key);

  return (
    <Card data-testid={`review-tool-${toolInfo.toolset_type}`} className="mb-3">
      <CardContent className="pt-4 pb-4">
        <div className="flex items-start gap-3">
          <Checkbox
            checked={isApproved}
            onCheckedChange={(checked) => onToggleApproval(toolInfo.toolset_type, checked === true)}
            title="Uncheck to deny this tool type"
            data-testid={`review-tool-checkbox-${toolInfo.toolset_type}`}
            className="mt-1"
          />
          <div className={`flex flex-col gap-2 flex-1 ${!isApproved ? 'opacity-50 pointer-events-none' : ''}`}>
            <div className="font-medium">{toolInfo.name}</div>
            <div className="text-sm text-muted-foreground">{toolInfo.description}</div>

            {!hasInstances ? (
              <Alert variant="destructive" data-testid={`review-no-instances-${toolInfo.toolset_type}`}>
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>
                  No instances configured for this tool type. You need to configure an instance before you can approve.
                </AlertDescription>
              </Alert>
            ) : (
              <Select
                value={selectedInstance ?? ''}
                onValueChange={(value) => onSelectInstance(toolInfo.toolset_type, value)}
                data-testid={`review-instance-select-${toolInfo.toolset_type}`}
              >
                <SelectTrigger data-testid={`review-instance-select-${toolInfo.toolset_type}`}>
                  <SelectValue placeholder="Select an instance..." />
                </SelectTrigger>
                <SelectContent>
                  {toolInfo.instances.map((instance) => {
                    const isDisabled = !instance.enabled || !instance.has_api_key;
                    const statusLabel = !instance.enabled ? '(disabled)' : !instance.has_api_key ? '(no API key)' : '';

                    return (
                      <SelectItem
                        key={instance.id}
                        value={instance.id}
                        disabled={isDisabled}
                        data-testid={`review-instance-option-${instance.id}`}
                      >
                        <span className="flex items-center gap-2">
                          {instance.slug}
                          {statusLabel && <span className="text-muted-foreground text-xs">{statusLabel}</span>}
                        </span>
                      </SelectItem>
                    );
                  })}
                </SelectContent>
              </Select>
            )}

            {hasInstances && validInstances.length === 0 && (
              <Alert variant="destructive" data-testid={`review-no-valid-instances-${toolInfo.toolset_type}`}>
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>
                  All instances are disabled or missing API keys. Configure an instance to approve.
                </AlertDescription>
              </Alert>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
};

// ============================================================================
// MCP Server Card Component
// ============================================================================

const McpServerCard = ({
  mcpInfo,
  selectedInstance,
  isApproved,
  onSelectInstance,
  onToggleApproval,
}: {
  mcpInfo: McpServerReviewInfo;
  selectedInstance: string | undefined;
  isApproved: boolean;
  onSelectInstance: (url: string, instanceId: string) => void;
  onToggleApproval: (url: string, approved: boolean) => void;
}) => {
  const hasInstances = mcpInfo.instances.length > 0;
  const validInstances = mcpInfo.instances.filter((i) => i.enabled);

  return (
    <Card data-testid={`review-mcp-${mcpInfo.url}`} className="mb-3">
      <CardContent className="pt-4 pb-4">
        <div className="flex items-start gap-3">
          <Checkbox
            checked={isApproved}
            onCheckedChange={(checked) => onToggleApproval(mcpInfo.url, !!checked)}
            data-testid={`review-mcp-toggle-${mcpInfo.url}`}
            className="mt-1"
          />
          <div className="flex-1">
            <div className="flex items-center gap-2 mb-1">
              <span className="font-medium text-sm">MCP Server</span>
              <Badge variant="outline" className="text-xs">
                {mcpInfo.url}
              </Badge>
            </div>

            {hasInstances && validInstances.length > 0 && isApproved && (
              <Select
                value={selectedInstance ?? ''}
                onValueChange={(value) => onSelectInstance(mcpInfo.url, value)}
                data-testid={`review-mcp-select-${mcpInfo.url}`}
              >
                <SelectTrigger className="w-full mt-2" data-testid={`review-mcp-select-trigger-${mcpInfo.url}`}>
                  <SelectValue placeholder="Select an MCP instance..." />
                </SelectTrigger>
                <SelectContent>
                  {validInstances.map((instance) => (
                    <SelectItem
                      key={instance.id}
                      value={instance.id}
                      data-testid={`review-mcp-instance-option-${instance.id}`}
                    >
                      {instance.name} ({instance.slug})
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            )}

            {!hasInstances && (
              <Alert variant="destructive" data-testid={`review-no-mcp-instances-${mcpInfo.url}`}>
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>
                  No MCP instances connected to this server. Create an instance first.
                </AlertDescription>
              </Alert>
            )}

            {hasInstances && validInstances.length === 0 && (
              <Alert variant="destructive" data-testid={`review-no-valid-mcp-instances-${mcpInfo.url}`}>
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>All MCP instances are disabled. Enable an instance to approve.</AlertDescription>
              </Alert>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
};

// ============================================================================
// Non-Draft Status Handler
// ============================================================================

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

// ============================================================================
// Review Content Component
// ============================================================================

const ReviewContent = () => {
  const searchParams = useSearchParams();
  const id = searchParams?.get('id');

  const { showError } = useToastMessages();
  const [selectedInstances, setSelectedInstances] = useState<Record<string, string>>({});
  const [approvedTools, setApprovedTools] = useState<Record<string, boolean>>({});
  const [selectedMcpInstances, setSelectedMcpInstances] = useState<Record<string, string>>({});
  const [approvedMcps, setApprovedMcps] = useState<Record<string, boolean>>({});
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [actionResult, setActionResult] = useState<AccessRequestActionResponse | null>(null);

  const { data: reviewData, isLoading, error } = useAppAccessRequestReview(id);

  const handleActionSuccess = (data: AccessRequestActionResponse) => {
    if (data.flow_type === 'popup') {
      window.close();
    } else if (data.flow_type === 'redirect' && data.redirect_url) {
      window.location.href = data.redirect_url;
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

  useEffect(() => {
    if (reviewData?.tools_info) {
      const initial: Record<string, boolean> = {};
      reviewData.tools_info.forEach((tool) => {
        initial[tool.toolset_type] = true;
      });
      setApprovedTools(initial);
    }
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
    const toolsValid = (reviewData.tools_info ?? []).every((tool) => {
      if (!approvedTools[tool.toolset_type]) return true;
      if (tool.instances.length === 0) return false;
      const validInstances = tool.instances.filter((i) => i.enabled && i.has_api_key);
      if (validInstances.length === 0) return false;
      return !!selectedInstances[tool.toolset_type];
    });
    const mcpsValid = (reviewData.mcps_info ?? []).every((mcp) => {
      if (!approvedMcps[mcp.url]) return true;
      const validInstances = mcp.instances.filter((i) => i.enabled);
      if (validInstances.length === 0) return false;
      return !!selectedMcpInstances[mcp.url];
    });
    return toolsValid && mcpsValid;
  }, [reviewData, selectedInstances, approvedTools, selectedMcpInstances, approvedMcps]);

  // Compute approve button label
  const approvedCount = useMemo(() => {
    const toolsApproved = (reviewData?.tools_info ?? []).filter((t) => approvedTools[t.toolset_type]).length;
    const mcpsApproved = (reviewData?.mcps_info ?? []).filter((m) => approvedMcps[m.url]).length;
    return toolsApproved + mcpsApproved;
  }, [reviewData, approvedTools, approvedMcps]);

  const totalCount = (reviewData?.tools_info?.length ?? 0) + (reviewData?.mcps_info?.length ?? 0);

  // No id query param
  if (!id) {
    return <ErrorPage message="Missing access request ID" />;
  }

  // Action completed: show immediate result without waiting for refetch
  if (actionResult) {
    return <NonDraftStatus status={actionResult.status} flowType={actionResult.flow_type} />;
  }

  // Loading state
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

  // Error state
  if (error || !reviewData) {
    return (
      <div data-testid="review-access-error">
        <ErrorPage message={error?.response?.data?.error?.message || 'Failed to load access request'} />
      </div>
    );
  }

  // Non-draft: already processed
  if (reviewData.status !== 'draft') {
    return <NonDraftStatus status={reviewData.status} flowType={reviewData.flow_type} />;
  }

  // Draft: show review form
  const handleSelectInstance = (toolType: string, instanceId: string) => {
    setSelectedInstances((prev) => ({ ...prev, [toolType]: instanceId }));
  };

  const handleToggleApproval = (toolType: string, approved: boolean) => {
    setApprovedTools((prev) => ({ ...prev, [toolType]: approved }));
  };

  const handleApprove = () => {
    setIsSubmitting(true);
    const body: ApproveAccessRequestBody = {
      approved: {
        toolsets: (reviewData.tools_info ?? []).map((tool) => ({
          toolset_type: tool.toolset_type,
          status: approvedTools[tool.toolset_type] ? 'approved' : 'denied',
          instance:
            approvedTools[tool.toolset_type] && selectedInstances[tool.toolset_type]
              ? { id: selectedInstances[tool.toolset_type] }
              : undefined,
        })),
        mcps: (reviewData.mcps_info ?? []).map((mcp) => ({
          url: mcp.url,
          status: approvedMcps[mcp.url] ? 'approved' : 'denied',
          instance:
            approvedMcps[mcp.url] && selectedMcpInstances[mcp.url] ? { id: selectedMcpInstances[mcp.url] } : undefined,
        })),
      },
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
      className="container mx-auto max-w-2xl p-4"
      data-testid="review-access-page"
      data-test-status={reviewData.status}
      data-test-flow-type={reviewData.flow_type}
    >
      <Card>
        <CardHeader>
          <CardTitle>Review Access Request</CardTitle>
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
          {reviewData.tools_info && reviewData.tools_info.length > 0 && (
            <div className="mb-4">
              <h3 className="text-sm font-medium mb-2">Requested Tools:</h3>
              {reviewData.tools_info.map((toolInfo) => (
                <ToolTypeCard
                  key={toolInfo.toolset_type}
                  toolInfo={toolInfo}
                  selectedInstance={selectedInstances[toolInfo.toolset_type]}
                  isApproved={approvedTools[toolInfo.toolset_type] ?? true}
                  onSelectInstance={handleSelectInstance}
                  onToggleApproval={handleToggleApproval}
                />
              ))}
            </div>
          )}

          {reviewData.mcps_info && reviewData.mcps_info.length > 0 && (
            <div className="mb-4">
              <h3 className="text-sm font-medium mb-2">Requested MCP Servers:</h3>
              {reviewData.mcps_info.map((mcpInfo) => (
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
            </div>
          )}

          <div className="flex justify-between gap-4">
            <Button variant="outline" onClick={handleDeny} disabled={isSubmitting} data-testid="review-deny-button">
              {denyMutation.isLoading ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Denying...
                </>
              ) : (
                'Deny'
              )}
            </Button>
            <Button onClick={handleApprove} disabled={!canApprove || isSubmitting} data-testid="review-approve-button">
              {approveMutation.isLoading ? (
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

// ============================================================================
// Page Component
// ============================================================================

export default function ReviewAccessRequestPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <ReviewContent />
    </AppInitializer>
  );
}
