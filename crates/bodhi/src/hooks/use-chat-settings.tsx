'use client';

import { createContext, useContext, useCallback, useEffect, useState } from 'react';

interface ChatSettings {
  model: string;
  stream?: boolean;
  stream_enabled: boolean;
  seed?: number;
  seed_enabled: boolean;
  systemPrompt?: string;
  systemPrompt_enabled: boolean;
  stop?: string[];
  stop_enabled: boolean;
  max_tokens?: number;
  max_tokens_enabled: boolean;
  n?: number;
  n_enabled: boolean;
  temperature?: number;
  temperature_enabled: boolean;
  top_p?: number;
  top_p_enabled: boolean;
  presence_penalty?: number;
  presence_penalty_enabled: boolean;
  frequency_penalty?: number;
  frequency_penalty_enabled: boolean;
  logit_bias?: Record<string, number>;
  logit_bias_enabled: boolean;
  response_format?: {
    type: 'text' | 'json_object';
    schema?: object;
  };
  response_format_enabled: boolean;
}

const defaultSettings: ChatSettings = {
  model: '',
  stream: true,
  temperature_enabled: false,
  top_p_enabled: false,
  n_enabled: false,
  stream_enabled: true,
  max_tokens_enabled: false,
  presence_penalty_enabled: false,
  frequency_penalty_enabled: false,
  logit_bias_enabled: false,
  stop_enabled: false,
  seed_enabled: false,
  systemPrompt_enabled: false,
  response_format_enabled: false
};

interface ChatSettingsContextType extends ChatSettings {
  setModel: (model: string) => void;
  setTemperature: (temp: number | undefined) => void;
  setTemperatureEnabled: (enabled: boolean) => void;
  setTopP: (topP: number | undefined) => void;
  setTopPEnabled: (enabled: boolean) => void;
  setN: (n: number | undefined) => void;
  setNEnabled: (enabled: boolean) => void;
  setStream: (stream: boolean | undefined) => void;
  setStreamEnabled: (enabled: boolean) => void;
  setMaxTokens: (tokens: number | undefined) => void;
  setMaxTokensEnabled: (enabled: boolean) => void;
  setPresencePenalty: (penalty: number | undefined) => void;
  setPresencePenaltyEnabled: (enabled: boolean) => void;
  setFrequencyPenalty: (penalty: number | undefined) => void;
  setFrequencyPenaltyEnabled: (enabled: boolean) => void;
  setLogitBias: (bias: Record<string, number> | undefined) => void;
  setLogitBiasEnabled: (enabled: boolean) => void;
  setStop: (stop: string[] | string | undefined) => void;
  setStopEnabled: (enabled: boolean) => void;
  setSeed: (seed: number | undefined) => void;
  setSeedEnabled: (enabled: boolean) => void;
  setSystemPrompt: (prompt: string | undefined) => void;
  setSystemPromptEnabled: (enabled: boolean) => void;
  setResponseFormat: (format: ChatSettings['response_format'] | undefined) => void;
  setResponseFormatEnabled: (enabled: boolean) => void;
  getRequestSettings: () => Omit<ChatSettings, 'systemPrompt' | keyof { [K in keyof ChatSettings as K extends `${string}_enabled` ? K : never]: never }>;
  reset: () => void;
}

const ChatSettingsContext = createContext<ChatSettingsContextType | undefined>(undefined);

export function ChatSettingsProvider({ children }: { children: React.ReactNode }) {
  const [settings, setSettings] = useState<ChatSettings>(() => {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem('chat-settings');
      if (saved) {
        try {
          return { ...defaultSettings, ...JSON.parse(saved) };
        } catch (e) {
          // If JSON parsing fails, return default settings
          console.warn('Failed to parse chat settings from localStorage:', e);
          return defaultSettings;
        }
      }
      return defaultSettings;
    }
    return defaultSettings;
  });

  useEffect(() => {
    localStorage.setItem('chat-settings', JSON.stringify(settings));
  }, [settings]);

  // Generic setter that handles both value and enabled state
  const setSetting = useCallback(<K extends keyof Omit<ChatSettings, `${string}_enabled`>>(
    key: K,
    value: ChatSettings[K] | undefined
  ) => {
    setSettings(prev => {
      const next = { ...prev };
      if (value === undefined) {
        delete next[key];
        // @ts-ignore - enabled key is valid but TypeScript can't infer it
        next[`${key}_enabled`] = false;
      } else {
        next[key] = value;
        // @ts-ignore - enabled key is valid but TypeScript can't infer it
        next[`${key}_enabled`] = true;
      }
      return next;
    });
  }, []);

  // Create setters for both value and enabled state
  const createSetters = useCallback(<K extends keyof Omit<ChatSettings, 'model' | `${string}_enabled`>>(
    key: K
  ) => {
    return {
      setValue: (value: ChatSettings[K] | undefined) => setSetting(key, value),
      setEnabled: (enabled: boolean) => setSettings(prev => ({
        ...prev,
        [`${key}_enabled`]: enabled
      }))
    };
  }, [setSetting]);

  const setModel = useCallback((model: string) => {
    setSetting('model', model);
  }, [setSetting]);

  const { setValue: setTemperature, setEnabled: setTemperatureEnabled } = createSetters('temperature');
  const { setValue: setTopP, setEnabled: setTopPEnabled } = createSetters('top_p');
  const { setValue: setN, setEnabled: setNEnabled } = createSetters('n');
  const { setValue: setStream, setEnabled: setStreamEnabled } = createSetters('stream');
  const { setValue: setMaxTokens, setEnabled: setMaxTokensEnabled } = createSetters('max_tokens');
  const { setValue: setPresencePenalty, setEnabled: setPresencePenaltyEnabled } = createSetters('presence_penalty');
  const { setValue: setFrequencyPenalty, setEnabled: setFrequencyPenaltyEnabled } = createSetters('frequency_penalty');
  const { setValue: setLogitBias, setEnabled: setLogitBiasEnabled } = createSetters('logit_bias');
  const { setValue: setStopRaw, setEnabled: setStopEnabled } = createSetters('stop');
  const setStop = useCallback((stop: string[] | string | undefined) => {
    if (stop === undefined) {
      setStopRaw(undefined);
    } else {
      // Convert to array if string
      setStopRaw(Array.isArray(stop) ? stop : [stop]);
    }
  }, [setStopRaw]);
  const { setValue: setSeed, setEnabled: setSeedEnabled } = createSetters('seed');
  const { setValue: setSystemPrompt, setEnabled: setSystemPromptEnabled } = createSetters('systemPrompt');
  const { setValue: setResponseFormat, setEnabled: setResponseFormatEnabled } = createSetters('response_format');

  const getRequestSettings = useCallback(() => {
    const requestSettings: any = {};

    // Helper to check if a setting should be included
    const shouldInclude = (key: keyof ChatSettings) => {
      const value = settings[key];
      const isEnabled = settings[`${key}_enabled` as keyof ChatSettings];
      return value !== undefined && isEnabled;
    };

    // Always include model
    if (settings.model) {
      requestSettings.model = settings.model;
    }

    // Include other settings only if they're enabled and defined
    const settingsToCheck: (keyof Omit<ChatSettings, 'model' | `${string}_enabled`>)[] = [
      'temperature', 'top_p', 'n', 'stream', 'max_tokens',
      'presence_penalty', 'frequency_penalty', 'logit_bias',
      'stop', 'seed', 'response_format'
    ];

    settingsToCheck.forEach(key => {
      if (shouldInclude(key)) {
        requestSettings[key] = settings[key];
      }
    });

    return requestSettings;
  }, [settings]);

  const reset = useCallback(() => {
    setSettings(defaultSettings);
  }, []);

  return (
    <ChatSettingsContext.Provider
      value={{
        ...settings,
        setModel,
        setTemperature,
        setTemperatureEnabled,
        setTopP,
        setTopPEnabled,
        setN,
        setNEnabled,
        setStream,
        setStreamEnabled,
        setMaxTokens,
        setMaxTokensEnabled,
        setPresencePenalty,
        setPresencePenaltyEnabled,
        setFrequencyPenalty,
        setFrequencyPenaltyEnabled,
        setLogitBias,
        setLogitBiasEnabled,
        setStop,
        setStopEnabled,
        setSeed,
        setSeedEnabled,
        setSystemPrompt,
        setSystemPromptEnabled,
        setResponseFormat,
        setResponseFormatEnabled,
        getRequestSettings,
        reset
      }}
    >
      {children}
    </ChatSettingsContext.Provider>
  );
}

export function useChatSettings() {
  const context = useContext(ChatSettingsContext);
  if (context === undefined) {
    throw new Error('useChatSettings must be used within a ChatSettingsProvider');
  }
  return context;
} 