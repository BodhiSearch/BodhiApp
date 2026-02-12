'use client';

import React, { useEffect, useMemo, useState } from 'react';

import { AlertCircle, CheckCircle, Loader2, XCircle } from 'lucide-react';
import { useSearchParams } from 'next/navigation';

import AppInitializer from '@/components/AppInitializer';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Skeleton } from '@/components/ui/skeleton';
import { useToastMessages } from '@/hooks/use-toast-messages';
import {
  useAppAccessRequestReview,
  useApproveAppAccessRequest,
  useDenyAppAccessRequest,
} from '@/hooks/useAppAccessRequests';
import type { ApproveAccessRequestBody, ToolTypeReviewInfo } from '@/hooks/useAppAccessRequests';

// ============================================================================
// Tool Type Card Component
// ============================================================================

const ToolTypeCard = ({
  toolInfo,
  selectedInstance,
  onSelectInstance,
}: {
  toolInfo: ToolTypeReviewInfo;
  selectedInstance: string | undefined;
  onSelectInstance: (toolType: string, instanceId: string) => void;
}) => {
  const hasInstances = toolInfo.instances.length > 0;
  const validInstances = toolInfo.instances.filter((i) => i.enabled && i.has_api_key);

  return (
    <Card data-testid={`review-tool-${toolInfo.tool_type}`} className="mb-3">
      <CardContent className="pt-4 pb-4">
        <div className="flex flex-col gap-2">
          <div className="font-medium">{toolInfo.name}</div>
          <div className="text-sm text-muted-foreground">{toolInfo.description}</div>

          {!hasInstances ? (
            <Alert variant="destructive" data-testid={`review-no-instances-${toolInfo.tool_type}`}>
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>
                No instances configured for this tool type. You need to configure an instance before you can approve.
              </AlertDescription>
            </Alert>
          ) : (
            <Select
              value={selectedInstance ?? ''}
              onValueChange={(value) => onSelectInstance(toolInfo.tool_type, value)}
              data-testid={`review-instance-select-${toolInfo.tool_type}`}
            >
              <SelectTrigger data-testid={`review-instance-select-${toolInfo.tool_type}`}>
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
                        {instance.name}
                        {statusLabel && <span className="text-muted-foreground text-xs">{statusLabel}</span>}
                      </span>
                    </SelectItem>
                  );
                })}
              </SelectContent>
            </Select>
          )}

          {hasInstances && validInstances.length === 0 && (
            <Alert variant="destructive" data-testid={`review-no-valid-instances-${toolInfo.tool_type}`}>
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>
                All instances are disabled or missing API keys. Configure an instance to approve.
              </AlertDescription>
            </Alert>
          )}
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
  const [isSubmitting, setIsSubmitting] = useState(false);

  const { data: reviewData, isLoading, error } = useAppAccessRequestReview(id);

  const approveMutation = useApproveAppAccessRequest({
    onSuccess: () => {
      if (reviewData?.flow_type === 'popup') {
        window.close();
      }
    },
    onError: (message) => {
      setIsSubmitting(false);
      showError('Approval Failed', message);
    },
  });

  const denyMutation = useDenyAppAccessRequest({
    onSuccess: () => {
      if (reviewData?.flow_type === 'popup') {
        window.close();
      }
    },
    onError: (message) => {
      setIsSubmitting(false);
      showError('Denial Failed', message);
    },
  });

  // Check if all tool types have a valid instance selected
  const allToolTypesSelected = useMemo(() => {
    if (!reviewData?.tools_info) return false;
    return reviewData.tools_info.every((tool) => {
      // If no instances at all, can't approve
      if (tool.instances.length === 0) return false;
      // If no valid instances, can't approve
      const validInstances = tool.instances.filter((i) => i.enabled && i.has_api_key);
      if (validInstances.length === 0) return false;
      // Must have a selection
      return !!selectedInstances[tool.tool_type];
    });
  }, [reviewData, selectedInstances]);

  // No id query param
  if (!id) {
    return <ErrorPage message="Missing access request ID" />;
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

  const handleApprove = () => {
    setIsSubmitting(true);
    const body: ApproveAccessRequestBody = {
      approved: {
        toolset_types: reviewData.tools_info.map((tool) => ({
          tool_type: tool.tool_type,
          status: selectedInstances[tool.tool_type] ? 'approved' : 'denied',
          instance_id: selectedInstances[tool.tool_type],
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
            {' is requesting access to your tools.'}
          </CardDescription>
          {reviewData.app_description && (
            <p className="text-sm text-muted-foreground mt-1" data-testid="review-app-description">
              {reviewData.app_description}
            </p>
          )}
        </CardHeader>
        <CardContent>
          <div className="mb-4">
            <h3 className="text-sm font-medium mb-2">Requested Tools:</h3>
            {reviewData.tools_info.map((toolInfo) => (
              <ToolTypeCard
                key={toolInfo.tool_type}
                toolInfo={toolInfo}
                selectedInstance={selectedInstances[toolInfo.tool_type]}
                onSelectInstance={handleSelectInstance}
              />
            ))}
          </div>

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
            <Button
              onClick={handleApprove}
              disabled={!allToolTypesSelected || isSubmitting}
              data-testid="review-approve-button"
            >
              {approveMutation.isLoading ? (
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
