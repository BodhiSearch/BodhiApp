import React from 'react';

import { AlertCircle } from 'lucide-react';

import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import type { McpServerReviewInfo } from '@/hooks/apps';

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
    <Card data-testid={`review-mcp-${mcpInfo.url}`} className="rounded-[9px]">
      <CardContent className="p-3">
        <div className="flex flex-wrap items-center gap-2">
          <Checkbox
            checked={isApproved}
            onCheckedChange={(checked) => onToggleApproval(mcpInfo.url, !!checked)}
            data-testid={`review-mcp-toggle-${mcpInfo.url}`}
          />
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
            <SelectTrigger className="w-full mt-3" data-testid={`review-mcp-select-trigger-${mcpInfo.url}`}>
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
          <Alert variant="destructive" className="mt-3" data-testid={`review-no-mcp-instances-${mcpInfo.url}`}>
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>No MCP instances configured. Create one first.</AlertDescription>
          </Alert>
        )}

        {hasInstances && validInstances.length === 0 && (
          <Alert variant="destructive" className="mt-3" data-testid={`review-no-valid-mcp-instances-${mcpInfo.url}`}>
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>All MCP instances are disabled. Enable an instance to approve.</AlertDescription>
          </Alert>
        )}
      </CardContent>
    </Card>
  );
};

export default McpServerCard;
