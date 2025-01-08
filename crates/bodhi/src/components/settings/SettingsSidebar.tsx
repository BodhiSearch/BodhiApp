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

export function SettingsSidebar() {
  const { data: modelsResponse, isLoading } = useModels(1, 100, 'alias', 'asc');
  const settings = useChatSettings();
  const models = modelsResponse?.data || [];

  return (
    <Sidebar inner={true} side="right" variant="floating" data-testid="settings-sidebar">
      <SidebarHeader>
        <h2>Settings</h2>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <div className="space-y-4">
            <AliasSelector
              models={models}
              isLoading={isLoading}
            />

            <div className="flex items-center justify-between">
              <Label htmlFor="stream-mode">Stream Response</Label>
              <Switch
                id="stream-mode"
                checked={settings.stream}
                onCheckedChange={settings.setStream}
                disabled={isLoading}
              />
            </div>
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label htmlFor="seed-input">Seed</Label>
                <div className="flex items-center gap-4">
                  <Switch
                    id="seed-enabled"
                    checked={settings.seed_enabled}
                    onCheckedChange={settings.setSeedEnabled}
                    disabled={isLoading}
                  />
                </div>
              </div>
              <input
                type="number"
                id="seed-input"
                className="w-full h-9 rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm transition-colors"
                value={settings.seed}
                onChange={(e) => settings.setSeed(parseInt(e.target.value) || 0)}
                min={0}
                max={999999}
                disabled={isLoading || !settings.seed_enabled}
              />
            </div>            <SystemPrompt isLoading={isLoading} />
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

            <Separator />

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

            {/* Penalties */}
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
