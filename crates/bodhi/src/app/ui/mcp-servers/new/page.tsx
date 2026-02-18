'use client';

import { useState } from 'react';

import { useRouter } from 'next/navigation';

import AppInitializer from '@/components/AppInitializer';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { Textarea } from '@/components/ui/textarea';
import { toast } from '@/hooks/use-toast';
import { useCreateMcpServer } from '@/hooks/useMcps';

function NewMcpServerContent() {
  const router = useRouter();
  const [url, setUrl] = useState('');
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [enabled, setEnabled] = useState(true);
  const [errors, setErrors] = useState<Record<string, string>>({});

  const createMutation = useCreateMcpServer({
    onSuccess: () => {
      toast({ title: 'MCP server created' });
      router.push('/ui/mcp-servers');
    },
    onError: (message) => {
      toast({ title: 'Failed to create MCP server', description: message, variant: 'destructive' });
    },
  });

  const validate = () => {
    const newErrors: Record<string, string> = {};
    if (!name.trim()) newErrors.name = 'Name is required';
    if (name.length > 100) newErrors.name = 'Name cannot exceed 100 characters';
    if (!url.trim()) newErrors.url = 'URL is required';
    if (url.length > 2048) newErrors.url = 'URL cannot exceed 2048 characters';
    try {
      if (url.trim()) new URL(url.trim());
    } catch {
      newErrors.url = 'URL is not valid';
    }
    if (description.length > 255) newErrors.description = 'Description cannot exceed 255 characters';
    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!validate()) return;
    createMutation.mutate({
      url: url.trim(),
      name: name.trim(),
      description: description.trim() || undefined,
      enabled,
    });
  };

  return (
    <div className="container mx-auto p-4 max-w-2xl" data-testid="new-mcp-server-page">
      <Card>
        <CardHeader>
          <CardTitle>New MCP Server</CardTitle>
          <CardDescription>Register a new MCP server for users to connect to.</CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="url">URL *</Label>
              <Input
                id="url"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                placeholder="https://mcp.example.com/mcp"
                data-testid="mcp-server-url-input"
              />
              {errors.url && <p className="text-sm text-destructive">{errors.url}</p>}
            </div>

            <div className="space-y-2">
              <Label htmlFor="name">Name *</Label>
              <Input
                id="name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="My MCP Server"
                data-testid="mcp-server-name-input"
              />
              {errors.name && <p className="text-sm text-destructive">{errors.name}</p>}
            </div>

            <div className="space-y-2">
              <Label htmlFor="description">Description</Label>
              <Textarea
                id="description"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="Optional description"
                data-testid="mcp-server-description-input"
              />
              {errors.description && <p className="text-sm text-destructive">{errors.description}</p>}
            </div>

            <div className="flex items-center space-x-2">
              <Switch
                id="enabled"
                checked={enabled}
                onCheckedChange={setEnabled}
                data-testid="mcp-server-enabled-switch"
              />
              <Label htmlFor="enabled">Enabled</Label>
            </div>

            <div className="flex gap-2 justify-end">
              <Button type="button" variant="outline" onClick={() => router.push('/ui/mcp-servers')}>
                Cancel
              </Button>
              <Button type="submit" disabled={createMutation.isLoading} data-testid="mcp-server-save-button">
                {createMutation.isLoading ? 'Saving...' : 'Save'}
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}

export default function NewMcpServerPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <NewMcpServerContent />
    </AppInitializer>
  );
}
