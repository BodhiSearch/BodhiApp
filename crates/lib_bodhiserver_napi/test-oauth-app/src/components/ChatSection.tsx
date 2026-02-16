import React, { useState, useEffect } from 'react';
import { Card, CardHeader, CardTitle, CardContent, Button, Input, Select, Badge } from '@/components/ui';
import { useStreamingChat } from '@/hooks/useStreamingChat';
import { loadConfig, loadToken } from '@/lib/storage';

export function ChatSection() {
  const [models, setModels] = useState<string[]>([]);
  const [selectedModel, setSelectedModel] = useState('');
  const [input, setInput] = useState('');
  const { messages, status, error, sendMessage, clearMessages } = useStreamingChat();

  useEffect(() => {
    async function fetchModels() {
      try {
        const config = loadConfig();
        const token = loadToken();
        if (!config) return;
        const res = await fetch(`${config.bodhiServerUrl}/v1/models`, {
          headers: token ? { Authorization: `Bearer ${token}` } : {},
        });
        if (res.ok) {
          const data = await res.json();
          const modelIds = (data.data || []).map((m: { id: string }) => m.id);
          setModels(modelIds);
          if (modelIds.length > 0 && !selectedModel) {
            setSelectedModel(modelIds[0]);
          }
        }
      } catch (err) {
        console.error('Failed to fetch models:', err);
      }
    }
    fetchModels();
  }, []);

  const handleSend = () => {
    if (!input.trim() || !selectedModel || status === 'streaming') return;
    sendMessage(input.trim(), selectedModel);
    setInput('');
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <Card data-testid="section-chat" data-test-state={status}>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle>Chat</CardTitle>
          <Badge
            data-testid="chat-status"
            variant={status === 'streaming' ? 'warning' : status === 'error' ? 'destructive' : 'secondary'}
          >
            {status}
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="flex items-center gap-2">
          <Select
            data-testid="chat-model-select"
            data-test-state={models.length > 0 ? 'loaded' : 'empty'}
            value={selectedModel}
            onChange={(e) => setSelectedModel(e.target.value)}
            className="flex-1"
          >
            <option value="">Select a model...</option>
            {models.map((m) => (
              <option key={m} value={m}>{m}</option>
            ))}
          </Select>
          <Button variant="outline" size="sm" onClick={clearMessages}>Clear</Button>
        </div>

        <div data-testid="chat-messages" className="border rounded-md p-3 min-h-[200px] max-h-[400px] overflow-y-auto space-y-3">
          {messages.length === 0 && (
            <p className="text-sm text-muted-foreground text-center py-8">No messages yet</p>
          )}
          {messages.map((msg, i) => (
            <div key={i} className={`flex ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}>
              <div className={`max-w-[80%] rounded-lg px-3 py-2 text-sm ${
                msg.role === 'user'
                  ? 'bg-primary text-primary-foreground'
                  : 'bg-muted'
              }`}>
                <p className="text-xs font-semibold mb-1">{msg.role === 'user' ? 'You' : 'Assistant'}</p>
                <p className="whitespace-pre-wrap">{msg.content || (status === 'streaming' ? '...' : '')}</p>
              </div>
            </div>
          ))}
        </div>

        {error && <p data-testid="chat-error" className="text-sm text-destructive">{error}</p>}

        <div className="flex gap-2">
          <Input
            data-testid="chat-input"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Type a message..."
            disabled={status === 'streaming'}
          />
          <Button
            data-testid="btn-chat-send"
            onClick={handleSend}
            disabled={!input.trim() || !selectedModel || status === 'streaming'}
            size="sm"
          >
            Send
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
