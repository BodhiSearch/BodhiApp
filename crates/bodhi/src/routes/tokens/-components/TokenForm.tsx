import { useMemo } from 'react';

import { AliasResponse, CreateTokenRequest, TokenCreated } from '@bodhiapp/ts-client';
import { zodResolver } from '@hookform/resolvers/zod';
import { Loader2 } from 'lucide-react';
import { useForm } from 'react-hook-form';
import { z } from 'zod';

import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { useListMcps } from '@/hooks/mcps';
import { useListModels } from '@/hooks/models';
import { useCreateToken } from '@/hooks/tokens';
import { useGetUser } from '@/hooks/users';
import { useToastMessages } from '@/hooks/useToastMessages';

export const createTokenSchema = z.object({
  name: z.string().optional(),
  scope: z.enum(['scope_token_user', 'scope_token_power_user']),
  listModels: z.boolean(),
  modelMode: z.enum(['all', 'specific']),
  models: z.array(z.string()),
  listMcps: z.boolean(),
  mcpMode: z.enum(['all', 'specific']),
  mcps: z.array(z.string()),
});

export type TokenFormData = z.infer<typeof createTokenSchema>;

const DEFAULTS: TokenFormData = {
  name: '',
  scope: 'scope_token_user',
  listModels: false,
  modelMode: 'all',
  models: [],
  listMcps: false,
  mcpMode: 'all',
  mcps: [],
};

/** Resolve each alias to the model ids usable in inference (the grant id space). */
function grantableModelIds(aliases: AliasResponse[]): string[] {
  const ids: string[] = [];
  for (const alias of aliases) {
    if (alias.source === 'api') {
      const prefix = alias.prefix ?? '';
      for (const model of alias.models) ids.push(`${prefix}${model.id}`);
    } else if ('alias' in alias) {
      ids.push(alias.alias);
    }
  }
  return Array.from(new Set(ids));
}

export function toCreateTokenRequest(data: TokenFormData): CreateTokenRequest {
  const models = data.modelMode === 'all' ? { type: 'all' as const } : { type: 'specific' as const, ids: data.models };
  const mcps = data.mcpMode === 'all' ? { type: 'all' as const } : { type: 'specific' as const, ids: data.mcps };
  return {
    name: data.name || undefined,
    scope: data.scope,
    grants: { version: '1', list_models: data.listModels, models, list_mcps: data.listMcps, mcps },
  };
}

interface TokenFormProps {
  onTokenCreated: (token: TokenCreated) => void;
}

function MultiSelect({
  items,
  selected,
  onToggle,
  testIdPrefix,
  emptyText,
}: {
  items: { id: string; label: string }[];
  selected: string[];
  onToggle: (id: string) => void;
  testIdPrefix: string;
  emptyText: string;
}) {
  if (items.length === 0) {
    return <p className="text-sm text-muted-foreground">{emptyText}</p>;
  }
  return (
    <div className="max-h-48 space-y-2 overflow-y-auto rounded-md border p-3" data-testid={`${testIdPrefix}-list`}>
      {items.map((item) => (
        <label key={item.id} className="flex items-center gap-2 text-sm" htmlFor={`${testIdPrefix}-${item.id}`}>
          <Checkbox
            id={`${testIdPrefix}-${item.id}`}
            checked={selected.includes(item.id)}
            onCheckedChange={() => onToggle(item.id)}
            data-testid={`${testIdPrefix}-${item.id}`}
          />
          <span>{item.label}</span>
        </label>
      ))}
    </div>
  );
}

export function TokenForm({ onTokenCreated }: TokenFormProps) {
  const { showSuccess, showError } = useToastMessages();
  const { data: userInfo } = useGetUser();
  const { data: modelsData } = useListModels(1, 100, 'alias', 'asc');
  const { data: mcpsData } = useListMcps();

  const scopeOptions = useMemo(() => {
    const userRole = userInfo?.auth_status === 'logged_in' ? userInfo.role : undefined;
    if (userRole === 'resource_user') {
      return [{ value: 'scope_token_user', label: 'User' }];
    }
    return [
      { value: 'scope_token_user', label: 'User' },
      { value: 'scope_token_power_user', label: 'PowerUser' },
    ];
  }, [userInfo]);

  const modelItems = useMemo(
    () => grantableModelIds(modelsData?.data ?? []).map((id) => ({ id, label: id })),
    [modelsData]
  );
  const mcpItems = useMemo(() => (mcpsData?.mcps ?? []).map((m) => ({ id: m.id, label: m.name })), [mcpsData]);

  const form = useForm<TokenFormData>({
    resolver: zodResolver(createTokenSchema),
    mode: 'onSubmit',
    defaultValues: DEFAULTS,
  });

  const { mutate: createToken, isPending: isLoading } = useCreateToken({
    onSuccess: (response) => {
      onTokenCreated(response);
      form.reset(DEFAULTS);
      showSuccess('Success', 'API token successfully generated');
    },
    onError: (message) => showError('Error', message),
  });

  const onSubmit = (data: TokenFormData) => createToken(toCreateTokenRequest(data));

  const modelMode = form.watch('modelMode');
  const mcpMode = form.watch('mcpMode');
  const selectedModels = form.watch('models');
  const selectedMcps = form.watch('mcps');

  const toggle = (field: 'models' | 'mcps', id: string) => {
    const current = form.getValues(field);
    form.setValue(field, current.includes(id) ? current.filter((x) => x !== id) : [...current, id]);
  };

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6" data-testid="token-form">
        <FormField
          control={form.control}
          name="name"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Token Name (Optional)</FormLabel>
              <FormControl>
                <Input
                  placeholder="Enter a name for your token"
                  disabled={isLoading}
                  data-testid="token-name-input"
                  {...field}
                />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        {/* Model Access */}
        <div className="space-y-3 rounded-md border p-4" data-testid="model-access-section">
          <div className="flex items-center justify-between">
            <Label htmlFor="list-models-switch">List all models (/v1/models)</Label>
            <FormField
              control={form.control}
              name="listModels"
              render={({ field }) => (
                <Switch
                  id="list-models-switch"
                  checked={field.value}
                  onCheckedChange={field.onChange}
                  disabled={isLoading}
                  data-testid="list-models-switch"
                />
              )}
            />
          </div>
          <FormField
            control={form.control}
            name="modelMode"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Model inference</FormLabel>
                <Select onValueChange={field.onChange} value={field.value} disabled={isLoading}>
                  <FormControl>
                    <SelectTrigger data-testid="model-mode-select">
                      <SelectValue />
                    </SelectTrigger>
                  </FormControl>
                  <SelectContent>
                    <SelectItem value="all" data-testid="model-mode-all">
                      All models
                    </SelectItem>
                    <SelectItem value="specific" data-testid="model-mode-specific">
                      Specific models
                    </SelectItem>
                  </SelectContent>
                </Select>
              </FormItem>
            )}
          />
          {modelMode === 'specific' && (
            <MultiSelect
              items={modelItems}
              selected={selectedModels}
              onToggle={(id) => toggle('models', id)}
              testIdPrefix="model-option"
              emptyText="No models available to grant."
            />
          )}
        </div>

        {/* MCP Access */}
        <div className="space-y-3 rounded-md border p-4" data-testid="mcp-access-section">
          <div className="flex items-center justify-between">
            <Label htmlFor="list-mcps-switch">List all MCPs (/v1/mcps)</Label>
            <FormField
              control={form.control}
              name="listMcps"
              render={({ field }) => (
                <Switch
                  id="list-mcps-switch"
                  checked={field.value}
                  onCheckedChange={field.onChange}
                  disabled={isLoading}
                  data-testid="list-mcps-switch"
                />
              )}
            />
          </div>
          <FormField
            control={form.control}
            name="mcpMode"
            render={({ field }) => (
              <FormItem>
                <FormLabel>MCP connect</FormLabel>
                <Select onValueChange={field.onChange} value={field.value} disabled={isLoading}>
                  <FormControl>
                    <SelectTrigger data-testid="mcp-mode-select">
                      <SelectValue />
                    </SelectTrigger>
                  </FormControl>
                  <SelectContent>
                    <SelectItem value="all" data-testid="mcp-mode-all">
                      All MCPs
                    </SelectItem>
                    <SelectItem value="specific" data-testid="mcp-mode-specific">
                      Specific MCPs
                    </SelectItem>
                  </SelectContent>
                </Select>
              </FormItem>
            )}
          />
          {mcpMode === 'specific' && (
            <MultiSelect
              items={mcpItems}
              selected={selectedMcps}
              onToggle={(id) => toggle('mcps', id)}
              testIdPrefix="mcp-option"
              emptyText="No MCP instances available to grant."
            />
          )}
        </div>

        <FormField
          control={form.control}
          name="scope"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Token Scope</FormLabel>
              <Select onValueChange={field.onChange} value={field.value} disabled={isLoading}>
                <FormControl>
                  <SelectTrigger data-testid="token-scope-select">
                    <SelectValue placeholder="Select scope" />
                  </SelectTrigger>
                </FormControl>
                <SelectContent>
                  {scopeOptions.map((option) => (
                    <SelectItem key={option.value} value={option.value} data-testid={`scope-option-${option.value}`}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <FormMessage />
            </FormItem>
          )}
        />

        <Button type="submit" disabled={isLoading} data-testid="generate-token-button">
          {isLoading ? (
            <>
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              Generating...
            </>
          ) : (
            'Generate Token'
          )}
        </Button>
      </form>
    </Form>
  );
}
