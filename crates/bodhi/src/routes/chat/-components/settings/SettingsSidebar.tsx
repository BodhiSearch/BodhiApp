import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Separator } from '@/components/ui/separator';
import { Switch } from '@/components/ui/switch';
import { useListModels } from '@/hooks/models';
import { API_FORMAT_PRESETS } from '@/schemas/apiModel';
import { useChatSettingsStore } from '@/stores/chatSettingsStore';

import { AliasSelector } from './AliasSelector';
import { HelpTooltip } from './HelpTooltip';
import { SettingSlider } from './SettingSlider';
import { StopWords } from './StopWords';
import { SystemPrompt } from './SystemPrompt';
import { SETTINGS_TOOLTIPS } from './tooltips';

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
        <HelpTooltip text={tooltip} />
      </div>
      {children}
    </div>
  );
}

export function SettingsSidebar() {
  const { data: modelsResponse, isLoading } = useListModels(1, 100, 'alias', 'asc');
  const settings = useChatSettingsStore();
  const models = modelsResponse?.data || [];

  return (
    <div className="flex h-full min-h-0 flex-col" data-testid="settings-sidebar">
      <div className="flex-1 min-h-0 overflow-y-auto px-4 py-4">
        <div className="space-y-6">
          <div className="space-y-2">
            <AliasSelector models={models} isLoading={isLoading} tooltip={SETTINGS_TOOLTIPS.alias} />
            {settings.model && (
              <div className="text-xs text-muted-foreground px-1" data-testid="api-format-label">
                API Format:{' '}
                {API_FORMAT_PRESETS[settings.apiFormat as keyof typeof API_FORMAT_PRESETS]?.name ?? settings.apiFormat}
              </div>
            )}
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
              data-testid="api-token-input"
              data-enabled={String(!isLoading && settings.api_token_enabled)}
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

          <div className="space-y-2">
            <SettingRow
              label="Max Tool Iterations"
              tooltip={SETTINGS_TOOLTIPS.maxToolIterations}
              htmlFor="max-tool-iterations-enabled"
            >
              <Switch
                id="max-tool-iterations-enabled"
                checked={settings.maxToolIterations_enabled}
                onCheckedChange={settings.setMaxToolIterationsEnabled}
                disabled={isLoading}
                size="sm"
              />
            </SettingRow>
            <Input
              type="number"
              id="max-tool-iterations"
              data-testid="max-tool-iterations-input"
              value={settings.maxToolIterations ?? 5}
              onChange={(e) => settings.setMaxToolIterations(parseInt(e.target.value) || 5)}
              min={1}
              max={20}
              disabled={isLoading || !settings.maxToolIterations_enabled}
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
      </div>
    </div>
  );
}
