'use client';

import { createContext, useContext, useCallback, useEffect, useState } from 'react';

interface ChatSettings {
  model: string;
  temperature?: number;
  top_p?: number;
  n?: number;
  stream?: boolean;
  max_tokens?: number;
  presence_penalty?: number;
  frequency_penalty?: number;
  logit_bias?: Record<string, number>;
  stop?: string[] | string;
  seed?: number;
  systemPrompt?: string;
  response_format?: {
    type: 'text' | 'json_object';
    schema?: object;
  };
}

const defaultSettings: ChatSettings = {
  model: ''
};

interface ChatSettingsContextType extends ChatSettings {
  setModel: (model: string) => void;
  setTemperature: (temp: number | undefined) => void;
  setTopP: (topP: number | undefined) => void;
  setN: (n: number | undefined) => void;
  setStream: (stream: boolean | undefined) => void;
  setMaxTokens: (tokens: number | undefined) => void;
  setPresencePenalty: (penalty: number | undefined) => void;
  setFrequencyPenalty: (penalty: number | undefined) => void;
  setLogitBias: (bias: Record<string, number> | undefined) => void;
  setStop: (stop: string[] | string | undefined) => void;
  setSeed: (seed: number | undefined) => void;
  setSystemPrompt: (prompt: string | undefined) => void;
  setResponseFormat: (format: ChatSettings['response_format'] | undefined) => void;
  getRequestSettings: () => Omit<ChatSettings, 'systemPrompt'>;
  reset: () => void;
}

const ChatSettingsContext = createContext<ChatSettingsContextType | undefined>(undefined);

export function ChatSettingsProvider({ children }: { children: React.ReactNode }) {
  const [settings, setSettings] = useState<ChatSettings>(() => {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem('chat-settings');
      return saved ? JSON.parse(saved) : defaultSettings;
    }
    return defaultSettings;
  });

  useEffect(() => {
    localStorage.setItem('chat-settings', JSON.stringify(settings));
  }, [settings]);

  // Generic setter that removes undefined values
  const setSetting = useCallback(<K extends keyof ChatSettings>(
    key: K,
    value: ChatSettings[K] | undefined
  ) => {
    setSettings(prev => {
      const next = { ...prev };
      if (value === undefined) {
        delete next[key];
      } else {
        next[key] = value;
      }
      return next;
    });
  }, []);

  const setModel = useCallback((model: string) => {
    setSetting('model', model);
  }, [setSetting]);

  const setTemperature = useCallback((temperature: number | undefined) => {
    setSetting('temperature', temperature);
  }, [setSetting]);

  const setTopP = useCallback((top_p: number | undefined) => {
    setSetting('top_p', top_p);
  }, [setSetting]);

  const setN = useCallback((n: number | undefined) => {
    setSetting('n', n);
  }, [setSetting]);

  const setStream = useCallback((stream: boolean | undefined) => {
    setSetting('stream', stream);
  }, [setSetting]);

  const setMaxTokens = useCallback((max_tokens: number | undefined) => {
    setSetting('max_tokens', max_tokens);
  }, [setSetting]);

  const setPresencePenalty = useCallback((presence_penalty: number | undefined) => {
    setSetting('presence_penalty', presence_penalty);
  }, [setSetting]);

  const setFrequencyPenalty = useCallback((frequency_penalty: number | undefined) => {
    setSetting('frequency_penalty', frequency_penalty);
  }, [setSetting]);

  const setLogitBias = useCallback((logit_bias: Record<string, number> | undefined) => {
    setSetting('logit_bias', logit_bias);
  }, [setSetting]);

  const setStop = useCallback((stop: string[] | string | undefined) => {
    setSetting('stop', stop);
  }, [setSetting]);

  const setSeed = useCallback((seed: number | undefined) => {
    setSetting('seed', seed);
  }, [setSetting]);

  const setSystemPrompt = useCallback((systemPrompt: string | undefined) => {
    setSetting('systemPrompt', systemPrompt);
  }, [setSetting]);

  const setResponseFormat = useCallback((response_format: ChatSettings['response_format'] | undefined) => {
    setSetting('response_format', response_format);
  }, [setSetting]);

  const getRequestSettings = useCallback(() => {
    const { systemPrompt, ...rest } = settings;
    // Remove undefined values from request settings
    return Object.fromEntries(
      Object.entries(rest).filter(([_, value]) => value !== undefined)
    ) as Omit<ChatSettings, 'systemPrompt'>;
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
        setTopP,
        setN,
        setStream,
        setMaxTokens,
        setPresencePenalty,
        setFrequencyPenalty,
        setLogitBias,
        setStop,
        setSeed,
        setSystemPrompt,
        setResponseFormat,
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