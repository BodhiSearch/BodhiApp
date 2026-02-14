'use client';

import { useState, useEffect } from 'react';

import { ChevronDown, ChevronRight, Loader2, CheckCircle, XCircle, Wrench } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { decodeToolName } from '@/lib/toolsets';
import { cn } from '@/lib/utils';
import { Message, ToolCall } from '@/types/chat';

export type ToolCallStatus = 'calling' | 'completed' | 'error';

interface ToolCallMessageProps {
  toolCall: ToolCall;
  /** The tool result message, if available */
  toolResult?: Message;
  /** Current status of the tool call */
  status: ToolCallStatus;
  /** Force the collapsible to be open (for "calling" state) */
  forceOpen?: boolean;
}

/**
 * Format JSON for display with pretty printing.
 */
function formatJson(value: string): string {
  try {
    const parsed = JSON.parse(value);
    return JSON.stringify(parsed, null, 2);
  } catch {
    return value;
  }
}

/**
 * Get status badge configuration based on status.
 */
function getStatusConfig(status: ToolCallStatus): {
  label: string;
  variant: 'default' | 'secondary' | 'destructive' | 'outline' | 'blue' | 'green' | 'orange' | 'gray';
  icon: React.ReactNode;
} {
  switch (status) {
    case 'calling':
      return {
        label: 'Calling...',
        variant: 'blue',
        icon: <Loader2 className="h-3 w-3 animate-spin" />,
      };
    case 'completed':
      return {
        label: 'Completed',
        variant: 'green',
        icon: <CheckCircle className="h-3 w-3" />,
      };
    case 'error':
      return {
        label: 'Error',
        variant: 'destructive',
        icon: <XCircle className="h-3 w-3" />,
      };
  }
}

export function ToolCallMessage({ toolCall, toolResult, status, forceOpen = false }: ToolCallMessageProps) {
  // Auto-expand when calling, always reset to collapsed otherwise
  const [isOpen, setIsOpen] = useState(status === 'calling' || forceOpen);

  // Update open state when status changes
  useEffect(() => {
    if (status === 'calling' || forceOpen) {
      setIsOpen(true);
    } else {
      // Reset to collapsed when not calling
      setIsOpen(false);
    }
  }, [status, forceOpen]);

  const decoded = decodeToolName(toolCall.function.name);
  const toolName = decoded?.method || toolCall.function.name;
  const toolsetSlug = decoded?.toolsetSlug || 'unknown';
  const statusConfig = getStatusConfig(status);

  // Parse result content for display
  const resultContent = toolResult?.content ? formatJson(toolResult.content) : null;
  const isErrorResult = resultContent && resultContent.includes('"error"');

  return (
    <div className="my-2 rounded-lg border bg-muted/30" data-testid="tool-call-message">
      <Collapsible open={isOpen} onOpenChange={setIsOpen}>
        <CollapsibleTrigger
          className="flex w-full items-center justify-between p-3 hover:bg-muted/50 transition-colors"
          data-testid="tool-call-expand"
        >
          <div className="flex items-center gap-2">
            <div className="flex h-6 w-6 items-center justify-center rounded bg-muted">
              <Wrench className="h-3.5 w-3.5 text-muted-foreground" />
            </div>
            <div className="flex flex-col items-start">
              <span className="text-sm font-medium">{toolName}</span>
              <span className="text-xs text-muted-foreground">{toolsetSlug}</span>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <Badge
              variant={statusConfig.variant}
              className="flex items-center gap-1 text-xs"
              data-testid="tool-call-status"
            >
              {statusConfig.icon}
              {statusConfig.label}
            </Badge>
            {isOpen ? (
              <ChevronDown className="h-4 w-4 text-muted-foreground" />
            ) : (
              <ChevronRight className="h-4 w-4 text-muted-foreground" />
            )}
          </div>
        </CollapsibleTrigger>

        <CollapsibleContent data-testid="tool-call-content">
          <div className="border-t px-3 py-2 space-y-3">
            {/* Arguments */}
            <div>
              <h5 className="text-xs font-medium text-muted-foreground mb-1">Arguments</h5>
              <pre className="text-xs bg-muted p-2 rounded overflow-x-auto max-h-40 overflow-y-auto">
                {formatJson(toolCall.function.arguments)}
              </pre>
            </div>

            {/* Result (if available) */}
            {resultContent && (
              <div>
                <h5
                  className={cn(
                    'text-xs font-medium mb-1',
                    isErrorResult ? 'text-destructive' : 'text-muted-foreground'
                  )}
                >
                  {isErrorResult ? 'Error' : 'Result'}
                </h5>
                <pre
                  className={cn(
                    'text-xs p-2 rounded overflow-x-auto max-h-40 overflow-y-auto',
                    isErrorResult ? 'bg-destructive/10' : 'bg-muted'
                  )}
                >
                  {resultContent}
                </pre>
              </div>
            )}
          </div>
        </CollapsibleContent>
      </Collapsible>
    </div>
  );
}

interface ToolCallsDisplayProps {
  /** Tool calls from the assistant message */
  toolCalls: ToolCall[];
  /** All messages to find tool results */
  messages?: Message[];
  /** Whether the tools are currently being executed */
  isExecuting?: boolean;
}

/**
 * Display multiple tool calls with their results.
 * Matches tool calls to their results by tool_call_id.
 */
export function ToolCallsDisplay({ toolCalls, messages = [], isExecuting = false }: ToolCallsDisplayProps) {
  // Create a map of tool_call_id to result message
  const resultMap = new Map<string, Message>();
  for (const msg of messages) {
    if (msg.role === 'tool' && msg.tool_call_id) {
      resultMap.set(msg.tool_call_id, msg);
    }
  }

  return (
    <div className="space-y-1">
      {toolCalls.map((toolCall) => {
        const result = resultMap.get(toolCall.id);
        let status: ToolCallStatus = 'calling';

        if (result) {
          // Check if result contains an error
          try {
            const parsed = JSON.parse(result.content);
            status = parsed.error ? 'error' : 'completed';
          } catch {
            status = 'completed';
          }
        } else if (!isExecuting) {
          // No result and not executing - likely completed without explicit result
          status = 'completed';
        }

        return (
          <ToolCallMessage
            key={toolCall.id}
            toolCall={toolCall}
            toolResult={result}
            status={status}
            forceOpen={isExecuting && !result}
          />
        );
      })}
    </div>
  );
}
