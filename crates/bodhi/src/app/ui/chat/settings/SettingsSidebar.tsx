import { AliasSelector } from '@/app/ui/chat/settings/AliasSelector';
import { SettingSlider } from '@/app/ui/chat/settings/SettingSlider';
import { StopWords } from '@/app/ui/chat/settings/StopWords';
import { SystemPrompt } from '@/app/ui/chat/settings/SystemPrompt';
import { SETTINGS_TOOLTIPS } from '@/app/ui/chat/settings/tooltips';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Separator } from '@/components/ui/separator';
import { Sidebar, SidebarContent, SidebarGroup, SidebarHeader } from '@/components/ui/sidebar';
import { Switch } from '@/components/ui/switch';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { useChatSettings } from '@/hooks/use-chat-settings';
import { useModels } from '@/hooks/useQuery';
import { HelpCircle } from 'lucide-react';

interface SettingRowProps {
  label: string;
  tooltip: string;
  htmlFor: string;
  children: React.ReactNode;
}

function SettingRow({ label, tooltip, htmlFor, children }: SettingRowProps) {
  return (
    <div className="flex items-center justify-between gap-4">
      <div className="flex items-center gap-2">
        <Label htmlFor={htmlFor}>{label}</Label>
        <TooltipProvider>
          <Tooltip delayDuration={300}>
            <TooltipTrigger asChild>
              <HelpCircle className="h-4 w-4 text-muted-foreground hover:text-foreground transition-colors cursor-help" />
            </TooltipTrigger>
            <TooltipContent>
              <p className="max-w-xs text-sm">{tooltip}</p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      </div>
      {children}
    </div>
  );
}

export function SettingsSidebar() {
  const { data: modelsResponse, isLoading } = useModels(1, 100, 'alias', 'asc');
  const settings = useChatSettings();
  const models = modelsResponse?.data || [];

  return (
    <Sidebar inner={true} side="right" variant="floating" data-testid="settings-sidebar">
      <SidebarHeader className="px-4 py-2 bg-muted">
        <h2 className="text-lg font-semibold">Settings</h2>
      </SidebarHeader>
      <SidebarContent className="h-[calc(100vh-4rem)]">
        <SidebarGroup className="pb-20">
          <div className="space-y-6">
            <div className="space-y-2">
              <AliasSelector models={models} isLoading={isLoading} tooltip={SETTINGS_TOOLTIPS.alias} />
            </div>

            <SettingRow label="Stream Response" tooltip={SETTINGS_TOOLTIPS.stream} htmlFor="stream-mode">
              <Switch
                id="stream-mode"
                checked={settings.stream}
                onCheckedChange={settings.setStream}
                disabled={isLoading}
                size="sm"
              />
            </SettingRow>

            <div className="space-y-2">
              <SettingRow label="API Token" tooltip={SETTINGS_TOOLTIPS.apiToken} htmlFor="api-token-enabled">
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
                onChange={(e) => settings.setApiToken(e.target.value || undefined)}
                disabled={isLoading || !settings.api_token_enabled}
                placeholder="Enter your API token"
              />
            </div>

            <div className="space-y-2">
              <SettingRow label="Seed" tooltip={SETTINGS_TOOLTIPS.seed} htmlFor="seed-enabled">
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
                onChange={(e) => settings.setSeed(parseInt(e.target.value) || 0)}
                min={0}
                max={999999}
                disabled={isLoading || !settings.seed_enabled}
              />
            </div>

            <SystemPrompt isLoading={isLoading} tooltip={SETTINGS_TOOLTIPS.systemPrompt} />
            <StopWords isLoading={isLoading} tooltip={SETTINGS_TOOLTIPS.stopWords} />

            <SettingSlider
              label="Temperature"
              tooltip={SETTINGS_TOOLTIPS.temperature}
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
              tooltip={SETTINGS_TOOLTIPS.topP}
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
              tooltip={SETTINGS_TOOLTIPS.maxTokens}
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
              tooltip={SETTINGS_TOOLTIPS.presencePenalty}
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
              tooltip={SETTINGS_TOOLTIPS.frequencyPenalty}
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
