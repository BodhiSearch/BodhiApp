'use client';

import { AliasSelector } from '@/components/settings/AliasSelector';
import { SystemPrompt } from '@/components/settings/SystemPrompt';
import { StopWords } from '@/components/settings/StopWords';
import { useModels } from '@/hooks/useQuery';
import { useChatSettings } from '@/hooks/use-chat-settings';
import {
  SidebarContent,
  SidebarHeader,
  Sidebar,
  SidebarGroup,
} from '@/components/ui/sidebar';
import { Switch } from '@/components/ui/switch';
import { Label } from '@/components/ui/label';
import { Separator } from '@/components/ui/separator';
import { SettingSlider } from '@/components/settings/SettingSlider';
import { Input } from '@/components/ui/input';

interface SettingRowProps {
  label: string;
  htmlFor: string;
  children: React.ReactNode;
}

function SettingRow({ label, htmlFor, children }: SettingRowProps) {
  return (
    <div className="flex items-center justify-between gap-4">
      <Label htmlFor={htmlFor}>{label}</Label>
      {children}
    </div>
  );
}

export function SettingsSidebar() {
  const { data: modelsResponse, isLoading } = useModels(1, 100, 'alias', 'asc');
  const settings = useChatSettings();
  const models = modelsResponse?.data || [];

  return (
    <Sidebar
      inner={true}
      side="right"
      variant="floating"
      data-testid="settings-sidebar"
    >
      <SidebarHeader className="px-4 py-2 bg-muted">
        <h2 className="text-lg font-semibold">Settings</h2>
      </SidebarHeader>
      <SidebarContent className="h-[calc(100vh-4rem)]">
        <SidebarGroup className="pb-20">
          <div className="space-y-6">
            <AliasSelector models={models} isLoading={isLoading} />

            <SettingRow label="Stream Response" htmlFor="stream-mode">
              <Switch
                id="stream-mode"
                checked={settings.stream}
                onCheckedChange={settings.setStream}
                disabled={isLoading}
                size="sm"
              />
            </SettingRow>

            <div className="space-y-2">
              <SettingRow label="API Token" htmlFor="api-token-enabled">
                <Switch
                  id="api-token-enabled"
                  checked={settings.api_token_enabled}
                  onCheckedChange={settings.setApiTokenEnabled}
                  disabled={isLoading}
                  size="sm"
                />
              </SettingRow>
              <Input
                type="password"
                id="api-token"
                value={settings.api_token || ''}
                onChange={(e) =>
                  settings.setApiToken(e.target.value || undefined)
                }
                disabled={isLoading || !settings.api_token_enabled}
                placeholder="Enter your API token"
              />
            </div>

            <div className="space-y-2">
              <SettingRow label="Seed" htmlFor="seed-enabled">
                <Switch
                  id="seed-enabled"
                  checked={settings.seed_enabled}
                  onCheckedChange={settings.setSeedEnabled}
                  disabled={isLoading}
                  size="sm"
                />
              </SettingRow>
              <Input
                type="number"
                id="seed-input"
                value={settings.seed}
                onChange={(e) =>
                  settings.setSeed(parseInt(e.target.value) || 0)
                }
                min={0}
                max={999999}
                disabled={isLoading || !settings.seed_enabled}
              />
            </div>

            <SystemPrompt isLoading={isLoading} />
            <StopWords isLoading={isLoading} />

            <SettingSlider
              label="Temperature"
              value={settings.temperature}
              enabled={settings.temperature_enabled}
              onValueChange={settings.setTemperature}
              onEnabledChange={settings.setTemperatureEnabled}
              min={0}
              max={2}
              step={0.1}
              defaultValue={1}
              isLoading={isLoading}
            />

            <SettingSlider
              label="Top P"
              value={settings.top_p}
              enabled={settings.top_p_enabled}
              onValueChange={settings.setTopP}
              onEnabledChange={settings.setTopPEnabled}
              min={0}
              max={1}
              step={0.1}
              defaultValue={1}
              isLoading={isLoading}
            />

            <Separator className="my-4" />

            <SettingSlider
              label="Max Tokens"
              value={settings.max_tokens}
              enabled={settings.max_tokens_enabled}
              onValueChange={settings.setMaxTokens}
              onEnabledChange={settings.setMaxTokensEnabled}
              min={0}
              max={2048}
              step={1}
              defaultValue={2048}
              isLoading={isLoading}
            />

            <SettingSlider
              label="Presence Penalty"
              value={settings.presence_penalty}
              enabled={settings.presence_penalty_enabled}
              onValueChange={settings.setPresencePenalty}
              onEnabledChange={settings.setPresencePenaltyEnabled}
              min={-2}
              max={2}
              step={0.1}
              defaultValue={0}
              isLoading={isLoading}
            />

            <SettingSlider
              label="Frequency Penalty"
              value={settings.frequency_penalty}
              enabled={settings.frequency_penalty_enabled}
              onValueChange={settings.setFrequencyPenalty}
              onEnabledChange={settings.setFrequencyPenaltyEnabled}
              min={-2}
              max={2}
              step={0.1}
              defaultValue={0}
              isLoading={isLoading}
            />
          </div>
        </SidebarGroup>
      </SidebarContent>
    </Sidebar>
  );
}
