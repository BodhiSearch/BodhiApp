# UI Pages - Tools Feature

> Layer: `crates/bodhi` (Next.js) | Status: Planning

## Navigation

Add "Tools" item to sidebar navigation (top-level).

### Sidebar Entry

```tsx
// crates/bodhi/src/components/sidebar.tsx (or equivalent)
{
  name: 'Tools',
  href: '/ui/tools',
  icon: WrenchIcon,  // or similar
}
```

## Pages

### /ui/tools - Tools List

Lists all available tools with configuration status.

```tsx
// crates/bodhi/src/app/ui/tools/page.tsx

export default function ToolsPage() {
  const { data: tools } = useQuery({
    queryKey: ['available-tools'],
    queryFn: () => fetch('/bodhi/v1/tools/available').then(r => r.json()),
  });

  return (
    <div>
      <h1>Tools</h1>
      <p>Configure tools to enhance AI capabilities.</p>

      <div className="grid gap-4">
        {tools?.tools.map((tool) => (
          <ToolCard
            key={tool.tool_id}
            tool={tool}
            href={`/ui/tools/${tool.tool_id}`}
          />
        ))}
      </div>
    </div>
  );
}

function ToolCard({ tool, href }) {
  return (
    <Link href={href}>
      <Card>
        <CardHeader>
          <CardTitle>{tool.name}</CardTitle>
          <CardDescription>{tool.description}</CardDescription>
        </CardHeader>
        <CardFooter>
          <Badge variant={tool.enabled ? 'success' : 'secondary'}>
            {tool.enabled ? 'Enabled' : 'Not Configured'}
          </Badge>
        </CardFooter>
      </Card>
    </Link>
  );
}
```

### /ui/tools/[tool_id] - Tool Configuration

Configuration page for individual tool.

```tsx
// crates/bodhi/src/app/ui/tools/[toolId]/page.tsx

export default function ToolConfigPage({ params }: { params: { toolId: string } }) {
  const { toolId } = params;

  const { data: config } = useQuery({
    queryKey: ['tool-config', toolId],
    queryFn: () => fetch(`/bodhi/v1/tools/${toolId}`).then(r => r.json()),
  });

  const updateMutation = useMutation({
    mutationFn: (data: UpdateToolConfigRequest) =>
      fetch(`/bodhi/v1/tools/${toolId}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries(['tool-config', toolId]);
      toast.success('Tool configuration saved');
    },
  });

  const [enabled, setEnabled] = useState(config?.enabled ?? false);
  const [apiKey, setApiKey] = useState('');

  const handleSave = () => {
    updateMutation.mutate({
      enabled,
      api_key: apiKey || undefined,
    });
  };

  return (
    <div>
      <h1>Exa Web Search</h1>
      <p>Configure Exa AI for web search capabilities.</p>

      <Card>
        <CardContent className="space-y-4">
          {/* API Key Input */}
          <div>
            <Label htmlFor="api-key">Exa API Key</Label>
            <Input
              id="api-key"
              type="password"
              placeholder={config?.has_api_key ? '••••••••' : 'Enter your Exa API key'}
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              data-testid="exa-api-key-input"
            />
            <p className="text-sm text-muted-foreground">
              Get your API key from{' '}
              <a href="https://exa.ai" target="_blank" rel="noopener noreferrer">
                exa.ai
              </a>
            </p>
          </div>

          {/* Enable Toggle */}
          <div className="flex items-center justify-between">
            <Label htmlFor="enabled">Enable Tool</Label>
            <Switch
              id="enabled"
              checked={enabled}
              onCheckedChange={setEnabled}
              disabled={!config?.has_api_key && !apiKey}
              data-testid="exa-enabled-toggle"
            />
          </div>
        </CardContent>

        <CardFooter>
          <Button
            onClick={handleSave}
            disabled={updateMutation.isPending}
            data-testid="save-tool-config"
          >
            {updateMutation.isPending ? 'Saving...' : 'Save'}
          </Button>
        </CardFooter>
      </Card>
    </div>
  );
}
```

## Data Test IDs

| Element | data-testid |
|---------|-------------|
| API key input | `exa-api-key-input` |
| Enable toggle | `exa-enabled-toggle` |
| Save button | `save-tool-config` |
| Tool card | `tool-card-{tool_id}` |

## MSW Mocks (for tests)

```typescript
// crates/bodhi/src/mocks/handlers.ts

export const toolHandlers = [
  http.get('/bodhi/v1/tools/available', () => {
    return HttpResponse.json({
      tools: [
        {
          tool_id: 'builtin-exa-web-search',
          name: 'Exa Web Search',
          description: 'Search the web using Exa AI',
          configured: false,
          enabled: false,
          scope_required: 'scope_tools-builtin-exa-web-search',
        },
      ],
    });
  }),

  http.get('/bodhi/v1/tools/:toolId', ({ params }) => {
    return HttpResponse.json({
      tool_id: params.toolId,
      enabled: false,
      has_api_key: false,
      scope_required: 'scope_tools-builtin-exa-web-search',
    });
  }),

  http.put('/bodhi/v1/tools/:toolId', async ({ request }) => {
    const body = await request.json();
    return HttpResponse.json({
      tool_id: params.toolId,
      enabled: body.enabled,
      has_api_key: !!body.api_key,
      scope_required: 'scope_tools-builtin-exa-web-search',
    });
  }),

  // Mock Exa execution for chat flow tests
  http.post('/bodhi/v1/tools/:toolId/execute', async ({ request }) => {
    const body = await request.json();
    return HttpResponse.json({
      tool_call_id: body.tool_call_id,
      result: {
        results: [
          {
            title: 'Mock Search Result',
            url: 'https://example.com',
            snippet: 'Mock snippet for testing',
          },
        ],
      },
    });
  }),
];
```
