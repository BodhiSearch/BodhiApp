'use client';

import React from 'react';

import { AlertCircle } from 'lucide-react';

import { Alert, AlertDescription } from '@/components/ui/alert';
import { Card, CardContent } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import type { ToolTypeReviewInfo } from '@/hooks/useAppAccessRequests';

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

export default ToolTypeCard;
