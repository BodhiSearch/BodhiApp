import { useCallback, useMemo, useRef, useState } from 'react';

import { Mcp, McpExecuteResponse } from '@bodhiapp/ts-client';

import { CompletionResult, useChatCompletion } from './useChatCompletions';
import { useChatDB } from './useChatDb';
import { useChatSettings } from './useChatSettings';
import { useToastMessages } from '@/hooks/use-toast-messages';
import apiClient from '@/lib/apiClient';
import { encodeMcpToolName, decodeMcpToolName } from '@/lib/mcps';
import { nanoid } from '@/lib/utils';
import { Message, ToolCall } from '@/types/chat';

// System message injected when max tool iterations is reached
const MAX_ITERATIONS_MESSAGE =
  'You have reached the maximum number of tool call iterations. Please provide a final response to the user without making additional tool calls.';

/**
 * Execute a single MCP tool call via the backend API.
 */
async function executeMcpToolCall(
  toolCall: ToolCall,
  signal: AbortSignal,
  headers: Record<string, string>,
  mcpSlugToId: Map<string, string>
): Promise<Message> {
  const decoded = decodeMcpToolName(toolCall.function.name);
  if (!decoded) {
    return {
      role: 'tool' as const,
      content: JSON.stringify({ error: `Invalid MCP tool name format: ${toolCall.function.name}` }),
      tool_call_id: toolCall.id,
    };
  }

  const { mcpSlug, toolName } = decoded;
  const mcpId = mcpSlugToId.get(mcpSlug);
  if (!mcpId) {
    return {
      role: 'tool' as const,
      content: JSON.stringify({ error: `Unknown MCP: ${mcpSlug}` }),
      tool_call_id: toolCall.id,
    };
  }

  const baseUrl =
    apiClient.defaults.baseURL || (typeof window !== 'undefined' ? window.location.origin : 'http://localhost');
  const url = `${baseUrl}/bodhi/v1/mcps/${mcpId}/tools/${toolName}/execute`;

  try {
    const response = await fetch(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...headers,
      },
      body: JSON.stringify({
        params: JSON.parse(toolCall.function.arguments),
      }),
      signal,
    });

    const result: McpExecuteResponse = await response.json();

    return {
      role: 'tool' as const,
      content: result.error ? JSON.stringify({ error: result.error }) : JSON.stringify(result.result),
      tool_call_id: toolCall.id,
    };
  } catch (error) {
    if (error instanceof Error && error.name === 'AbortError') {
      throw error;
    }

    return {
      role: 'tool' as const,
      content: JSON.stringify({
        error: error instanceof Error ? error.message : 'MCP tool execution failed',
      }),
      tool_call_id: toolCall.id,
    };
  }
}

/**
 * Execute multiple MCP tool calls in parallel, continuing even if some fail.
 */
async function executeToolCalls(
  toolCalls: ToolCall[],
  signal: AbortSignal,
  headers: Record<string, string>,
  mcpSlugToId: Map<string, string>
): Promise<Message[]> {
  const results = await Promise.allSettled(toolCalls.map((tc) => executeMcpToolCall(tc, signal, headers, mcpSlugToId)));

  return results.map((result, index) => {
    if (result.status === 'fulfilled') {
      return result.value;
    }
    // For rejected promises (e.g., abort), return error message
    return {
      role: 'tool' as const,
      content: JSON.stringify({
        error: result.reason instanceof Error ? result.reason.message : 'Tool execution failed',
      }),
      tool_call_id: toolCalls[index].id,
    };
  });
}

interface McpToolDefinition {
  type: 'function';
  function: {
    name: string;
    description: string;
    parameters: Record<string, unknown>;
  };
}

/**
 * Build tools array for API request from enabled MCP tools.
 * Only includes tools from available MCPs (server enabled, instance enabled, tools cached, filter not empty).
 * Applies tools_filter ceiling, encodes names as mcp__{slug}__{toolName}.
 */
function buildMcpToolsArray(enabledMcpTools: Record<string, string[]>, mcps: Mcp[]): McpToolDefinition[] {
  const result: McpToolDefinition[] = [];
  for (const mcp of mcps) {
    if (
      !mcp.mcp_server.enabled ||
      !mcp.enabled ||
      !mcp.tools_cache ||
      mcp.tools_cache.length === 0 ||
      (mcp.tools_filter != null && mcp.tools_filter.length === 0)
    ) {
      continue;
    }

    const enabledToolNames = enabledMcpTools[mcp.id] || [];
    // Apply tools_filter ceiling
    const visibleTools =
      mcp.tools_filter == null ? mcp.tools_cache : mcp.tools_cache.filter((t) => mcp.tools_filter!.includes(t.name));

    for (const tool of visibleTools) {
      if (enabledToolNames.includes(tool.name)) {
        result.push({
          type: 'function',
          function: {
            name: encodeMcpToolName(mcp.slug, tool.name),
            description: tool.description ?? '',
            parameters: (tool.input_schema ?? {}) as Record<string, unknown>,
          },
        });
      }
    }
  }
  return result;
}

export interface UseChatOptions {
  enabledMcpTools?: Record<string, string[]>;
  mcps?: Mcp[];
}

export function useChat(options?: UseChatOptions) {
  const { enabledMcpTools = {}, mcps = [] } = options || {};

  // Build MCP slug→UUID mapping for tool execution
  const mcpSlugToId = useMemo(() => {
    const map = new Map<string, string>();
    mcps.forEach((m) => map.set(m.slug, m.id));
    return map;
  }, [mcps]);

  const [input, setInput] = useState('');
  const [abortController, setAbortController] = useState<AbortController | null>(null);
  const [userMessage, setUserMessage] = useState<Message>({
    role: 'user',
    content: '',
  });
  const [assistantMessage, setAssistantMessage] = useState<Message>({
    role: 'assistant',
    content: '',
  });
  // Track pending tool calls for UI display
  const [pendingToolCalls, setPendingToolCalls] = useState<ToolCall[]>([]);

  const { showError } = useToastMessages();
  const { append, isLoading } = useChatCompletion();
  const { currentChat, createOrUpdateChat, setCurrentChatId } = useChatDB();
  const chatSettings = useChatSettings();

  // Use ref to track abort state across async operations
  const abortedRef = useRef(false);

  // Reset state to before user submission
  const resetToPreSubmissionState = useCallback((userContent: string) => {
    setInput(userContent);
    setUserMessage({ role: 'user', content: '' });
    setAssistantMessage({ role: 'assistant', content: '' });
    setPendingToolCalls([]);
  }, []);

  // Helper function to extract error message
  const extractErrorMessage = (error: unknown): string => {
    if (typeof error === 'string') return error;

    if (error && typeof error === 'object') {
      if ('error' in error && error.error && typeof error.error === 'object') {
        return (error.error as { message?: string }).message || 'Error sending message to AI assistant.';
      }
      return (error as { message?: string }).message || 'Error sending message to AI assistant.';
    }

    return 'Error sending message to AI assistant.';
  };

  const processCompletion = useCallback(
    async (
      initialMessages: Message[],
      controller: AbortController,
      chatIdRef: { id: string; createdAt: number }
    ): Promise<void> => {
      let currentAssistantContent = '';
      const userContent = initialMessages.find((m) => m.role === 'user')?.content || '';
      let conversationMessages = [...initialMessages];
      let iteration = 0;
      const maxIterations = chatSettings.maxToolIterations_enabled ? (chatSettings.maxToolIterations ?? 5) : 5;

      // Build tools array from enabled MCP tools
      const tools = Object.keys(enabledMcpTools).length > 0 ? buildMcpToolsArray(enabledMcpTools, mcps) : [];

      const headers: Record<string, string> = {};
      if (chatSettings.api_token_enabled) {
        headers.Authorization = `Bearer ${chatSettings.api_token || ''}`;
      }

      const makeCompletionRequest = async (messages: Message[]): Promise<CompletionResult | null> => {
        return new Promise((resolve, reject) => {
          // Build request messages with system prompt
          const requestMessages =
            chatSettings.systemPrompt_enabled && chatSettings.systemPrompt
              ? [{ role: 'system' as const, content: chatSettings.systemPrompt }, ...messages]
              : messages;

          // Add max iterations warning if needed
          const finalRequestMessages =
            iteration >= maxIterations
              ? [...requestMessages, { role: 'system' as const, content: MAX_ITERATIONS_MESSAGE }]
              : requestMessages;

          append({
            request: {
              ...chatSettings.getRequestSettings(),
              messages: finalRequestMessages,
              ...(tools.length > 0 && {
                tools: tools.map((t) => ({ type: 'function' as const, function: t.function })),
              }),
            },
            headers,
            signal: controller.signal,
            onDelta: (chunk) => {
              currentAssistantContent += chunk;
              setAssistantMessage((prev) => ({
                ...prev,
                content: prev.content + chunk,
              }));
            },
            onToolCallDelta: (toolCalls) => {
              setPendingToolCalls(toolCalls);
              setAssistantMessage((prev) => ({
                ...prev,
                tool_calls: toolCalls,
              }));
            },
            onMessage: (message) => {
              setAssistantMessage({
                role: 'assistant' as const,
                content: message.content,
                tool_calls: message.tool_calls,
              });
            },
            onFinish: (result) => {
              resolve(result);
            },
            onError: (error) => {
              reject(error);
            },
          }).catch(reject);
        });
      };

      try {
        // Agentic loop
        while (!abortedRef.current) {
          const result = await makeCompletionRequest(conversationMessages);
          if (!result || abortedRef.current) break;

          const { message, finishReason, toolCalls } = result;

          // Build the assistant message with accumulated content
          const assistantMsg: Message = {
            role: 'assistant' as const,
            content: currentAssistantContent || message.content,
            metadata: message.metadata,
          };
          if (toolCalls && toolCalls.length > 0) {
            assistantMsg.tool_calls = toolCalls;
          }

          // Add assistant message to conversation
          conversationMessages = [...conversationMessages, assistantMsg];

          // Check if we need to execute tool calls
          if (finishReason === 'tool_calls' && toolCalls && toolCalls.length > 0) {
            iteration++;

            // Execute tool calls in parallel
            const toolResults = await executeToolCalls(toolCalls, controller.signal, headers, mcpSlugToId);

            if (abortedRef.current) break;

            // Add tool results to conversation
            conversationMessages = [...conversationMessages, ...toolResults];

            // Reset assistant content for next iteration
            currentAssistantContent = '';
            setAssistantMessage({ role: 'assistant', content: '' });
            setPendingToolCalls([]);

            // Continue the loop for another LLM call
            continue;
          }

          // finish_reason is 'stop' or other - we're done
          break;
        }

        // Save the conversation if not aborted
        if (!abortedRef.current) {
          createOrUpdateChat({
            id: chatIdRef.id,
            title: conversationMessages.find((m) => m.role === 'user')?.content.slice(0, 20) || 'New Chat',
            messages: conversationMessages,
            createdAt: chatIdRef.createdAt,
            updatedAt: Date.now(),
            enabledMcpTools: Object.keys(enabledMcpTools).length > 0 ? enabledMcpTools : undefined,
          });

          if (!currentChat) {
            setCurrentChatId(chatIdRef.id);
          }
        }

        // Reset UI state
        setAssistantMessage({ role: 'assistant' as const, content: '' });
        setUserMessage({ role: 'user' as const, content: '' });
        setPendingToolCalls([]);
        setInput('');
      } catch (error) {
        // Handle abort
        if (error instanceof Error && error.name === 'AbortError') {
          return;
        }

        const errorMessage = extractErrorMessage(error);
        showError('Error', errorMessage);

        // Only reset if we haven't started receiving assistant's response
        if (!currentAssistantContent) {
          resetToPreSubmissionState(userContent);
        }
      }
    },
    [
      chatSettings,
      currentChat,
      append,
      createOrUpdateChat,
      showError,
      setCurrentChatId,
      resetToPreSubmissionState,
      enabledMcpTools,
      mcps,
      mcpSlugToId,
    ]
  );

  const appendMessage = useCallback(
    async (content: string) => {
      abortedRef.current = false;
      setAssistantMessage({ role: 'assistant' as const, content: '' });
      setUserMessage({ role: 'user' as const, content });
      setPendingToolCalls([]);

      const existingMessages = currentChat?.messages || [];
      const newMessages = [...existingMessages, { role: 'user' as const, content }];

      const controller = new AbortController();
      setAbortController(controller);

      // Create or use existing chat ID
      const chatIdRef = {
        id: currentChat?.id || nanoid(),
        createdAt: currentChat?.createdAt || Date.now(),
      };

      try {
        await processCompletion(newMessages, controller, chatIdRef);
      } finally {
        setAbortController(null);
      }
    },
    [currentChat, processCompletion]
  );

  const stop = useCallback(() => {
    abortedRef.current = true;
    if (abortController) {
      abortController.abort();
      setAbortController(null);
    }
  }, [abortController]);

  return {
    input,
    setInput,
    isLoading,
    append: appendMessage,
    stop,
    userMessage,
    assistantMessage,
    pendingToolCalls,
  };
}
