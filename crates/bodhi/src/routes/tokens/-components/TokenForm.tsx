import { useMemo } from 'react';

import { CreateTokenRequest, TokenCreated } from '@bodhiapp/ts-client';
import { zodResolver } from '@hookform/resolvers/zod';
import { Loader2, ShieldPlus } from 'lucide-react';
import { useForm } from 'react-hook-form';
import { z } from 'zod';

import { AccessMode, GrantBlock } from '@/components/access-picker';
import { Button } from '@/components/ui/button';
import { Form, FormControl, FormField, FormItem, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { useListMcps } from '@/hooks/mcps';
import { useListModels } from '@/hooks/models';
import { useCreateToken } from '@/hooks/tokens';
import { useGetUser } from '@/hooks/users';
import { useToastMessages } from '@/hooks/useToastMessages';
import { grantableMcpItems, grantableModelItems } from '@/lib/grantItems';
import { cn } from '@/lib/utils';

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
  modelMode: 'specific',
  models: [],
  listMcps: false,
  mcpMode: 'specific',
  mcps: [],
};

export function toCreateTokenRequest(data: TokenFormData): CreateTokenRequest {
  const models = data.modelMode === 'all' ? { type: 'all' as const } : { type: 'specific' as const, ids: data.models };
  const mcps = data.mcpMode === 'all' ? { type: 'all' as const } : { type: 'specific' as const, ids: data.mcps };
  return {
    name: data.name || undefined,
    scope: data.scope,
    grants: { version: '1', models_list: data.listModels, models, mcps_list: data.listMcps, mcps },
  };
}

const ROLE_CARDS = [
  {
    scope: 'scope_token_user' as const,
    name: 'User',
    desc: 'Standard access. Can make inference requests and list the models and MCPs permitted by this token.',
    badge: 'scope_token_user',
    badgeClass: 'user',
  },
  {
    scope: 'scope_token_power_user' as const,
    name: 'Power User',
    desc: 'Elevated access. Can manage models, configure MCP servers, and perform admin-level API operations.',
    badge: 'scope_token_power_user',
    badgeClass: 'power',
  },
];

interface TokenFormProps {
  onTokenCreated: (token: TokenCreated) => void;
  onCancel: () => void;
}

export function TokenForm({ onTokenCreated, onCancel }: TokenFormProps) {
  const { showSuccess, showError } = useToastMessages();
  const { data: userInfo } = useGetUser();
  const { data: modelsData, isLoading: modelsLoading } = useListModels(1, 100, 'alias', 'asc');
  const { data: mcpsData, isLoading: mcpsLoading } = useListMcps();

  // The model/MCP access pickers re-render when these queries settle; expose a
  // ready marker so tests interact only after the grantable lists have loaded
  // (clicking a picker mid-load drops the event).
  const grantsState = modelsLoading || mcpsLoading ? 'loading' : 'ready';

  const canPowerUser = userInfo?.auth_status === 'logged_in' && userInfo.role !== 'resource_user';

  const modelItems = useMemo(() => grantableModelItems(modelsData?.data ?? []), [modelsData]);
  const mcpItems = useMemo(() => grantableMcpItems(mcpsData?.mcps ?? []), [mcpsData]);

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

  const listModels = form.watch('listModels');
  const modelMode = form.watch('modelMode');
  const selectedModels = form.watch('models');
  const listMcps = form.watch('listMcps');
  const mcpMode = form.watch('mcpMode');
  const selectedMcps = form.watch('mcps');
  const scope = form.watch('scope');

  const toggle = (field: 'models' | 'mcps', id: string) => {
    const current = form.getValues(field);
    form.setValue(field, current.includes(id) ? current.filter((x) => x !== id) : [...current, id]);
  };

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} data-testid="token-form" data-test-state={grantsState}>
        {/* §1 Token Identity */}
        <div className="nt-section">
          <div className="nt-section-title">Token Identity</div>
          <FormField
            control={form.control}
            name="name"
            render={({ field }) => (
              <FormItem>
                <FormControl>
                  <Input
                    placeholder="e.g. my-app-token"
                    disabled={isLoading}
                    data-testid="token-name-input"
                    {...field}
                  />
                </FormControl>
                <div className="nt-hint">
                  Token Name<span className="nt-optional">Optional</span> — a human-readable label to identify this
                  token in the token list.
                </div>
                <FormMessage />
              </FormItem>
            )}
          />
        </div>

        {/* §2 Model Access */}
        <div className="nt-section">
          <div className="nt-section-title">Model Access</div>
          <GrantBlock
            noun="model"
            listChecked={listModels}
            onListToggle={() => form.setValue('listModels', !listModels)}
            listLabel="List all models"
            listCode="/v1/models"
            listDescription="Let the token enumerate every model via the catalog. Off → it only sees the models you grant for inference below. (Listing is separate from running inference.)"
            listTestId="list-models-switch"
            mode={modelMode}
            onModeChange={(m: AccessMode) => form.setValue('modelMode', m)}
            items={modelItems}
            selectedIds={selectedModels}
            onToggle={(id) => toggle('models', id)}
            panelTitle="Select Models"
            panelSubtitle="Choose which models this token can access"
            testIdPrefix="model-access"
            disabled={isLoading}
          />
        </div>

        {/* §3 MCP Access */}
        <div className="nt-section">
          <div className="nt-section-title">MCP Access</div>
          <GrantBlock
            noun="MCP"
            listChecked={listMcps}
            onListToggle={() => form.setValue('listMcps', !listMcps)}
            listLabel="List all MCPs"
            listCode="/v1/mcps"
            listDescription="Let the token discover every MCP server. Off → it only sees the servers you grant a connection to below. (Listing is separate from connecting.)"
            listTestId="list-mcps-switch"
            mode={mcpMode}
            onModeChange={(m: AccessMode) => form.setValue('mcpMode', m)}
            items={mcpItems}
            selectedIds={selectedMcps}
            onToggle={(id) => toggle('mcps', id)}
            panelTitle="Select MCPs"
            panelSubtitle="Choose which MCP servers this token can invoke"
            allLabel="All MCPs"
            allDesc="Access all currently registered MCP servers and any added in the future."
            specificLabel="Specific MCPs"
            specificDesc="Choose exactly which MCP servers this token can invoke."
            testIdPrefix="mcp-access"
            disabled={isLoading}
          />
        </div>

        {/* §4 Token Scope */}
        <div className="nt-section">
          <div className="nt-section-title">Token Scope</div>
          <div className="nt-role-grid">
            {ROLE_CARDS.map((card) => {
              const disabledCard = card.scope === 'scope_token_power_user' && !canPowerUser;
              const selected = scope === card.scope;
              return (
                <button
                  type="button"
                  key={card.scope}
                  className={cn('nt-role-card', selected && 'selected', disabledCard && 'is-disabled')}
                  onClick={() => !disabledCard && !isLoading && form.setValue('scope', card.scope)}
                  aria-pressed={selected}
                  disabled={disabledCard}
                  data-testid={`scope-card-${card.scope}`}
                >
                  <div className="nt-role-head">
                    <span className="nt-role-name">{card.name}</span>
                    <span className="nt-role-dot">
                      <span className="nt-role-dot-inner" />
                    </span>
                  </div>
                  <div className="nt-role-desc">{card.desc}</div>
                  <span className={cn('nt-role-badge', card.badgeClass)}>{card.badge}</span>
                </button>
              );
            })}
          </div>
        </div>

        {/* Footer */}
        <div className="nt-footer">
          <Button
            type="button"
            variant="ghost"
            onClick={onCancel}
            disabled={isLoading}
            data-testid="cancel-token-button"
          >
            Cancel
          </Button>
          <Button type="submit" disabled={isLoading} data-testid="generate-token-button">
            {isLoading ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Generating...
              </>
            ) : (
              <>
                <ShieldPlus className="mr-2 h-4 w-4" />
                Generate Token
              </>
            )}
          </Button>
        </div>
      </form>
    </Form>
  );
}
