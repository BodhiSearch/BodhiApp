'use client';

import { createContext, useContext, useCallback, useEffect, useState } from 'react';

interface ChatSettings {
  model: string;
  temperature: number;
  top_p: number;
  n: number;
  stream: boolean;
  max_tokens: number;
  presence_penalty: number;
  frequency_penalty: number;
  logit_bias: Record<string, number>;
  stop: string[] | string;
  seed?: number;
  response_format?: {
    type: 'text' | 'json_object';
    schema?: object;
  };
}

const defaultSettings: ChatSettings = {
  model: '',
  temperature: 0.7,
  top_p: 1,
  n: 1,
  stream: true,
  max_tokens: 2048,
  presence_penalty: 0,
  frequency_penalty: 0,
  logit_bias: {},
  stop: [],
};

interface ChatSettingsContextType extends ChatSettings {
  setModel: (model: string) => void;
  setTemperature: (temp: number) => void;
  setTopP: (topP: number) => void;
  setN: (n: number) => void;
  setStream: (stream: boolean) => void;
  setMaxTokens: (tokens: number) => void;
  setPresencePenalty: (penalty: number) => void;
  setFrequencyPenalty: (penalty: number) => void;
  setLogitBias: (bias: Record<string, number>) => void;
  setStop: (stop: string[] | string) => void;
  setSeed: (seed?: number) => void;
  setResponseFormat: (format?: ChatSettings['response_format']) => void;
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

  const setModel = useCallback((model: string) => {
    setSettings(prev => ({ ...prev, model }));
  }, []);

  const setTemperature = useCallback((temperature: number) => {
    setSettings(prev => ({ ...prev, temperature }));
  }, []);

  const setTopP = useCallback((top_p: number) => {
    setSettings(prev => ({ ...prev, top_p }));
  }, []);

  const setN = useCallback((n: number) => {
    setSettings(prev => ({ ...prev, n }));
  }, []);

  const setStream = useCallback((stream: boolean) => {
    setSettings(prev => ({ ...prev, stream }));
  }, []);

  const setMaxTokens = useCallback((max_tokens: number) => {
    setSettings(prev => ({ ...prev, max_tokens }));
  }, []);

  const setPresencePenalty = useCallback((presence_penalty: number) => {
    setSettings(prev => ({ ...prev, presence_penalty }));
  }, []);

  const setFrequencyPenalty = useCallback((frequency_penalty: number) => {
    setSettings(prev => ({ ...prev, frequency_penalty }));
  }, []);

  const setLogitBias = useCallback((logit_bias: Record<string, number>) => {
    setSettings(prev => ({ ...prev, logit_bias }));
  }, []);

  const setStop = useCallback((stop: string[] | string) => {
    setSettings(prev => ({ ...prev, stop }));
  }, []);

  const setSeed = useCallback((seed?: number) => {
    setSettings(prev => ({ ...prev, seed }));
  }, []);

  const setResponseFormat = useCallback((response_format?: ChatSettings['response_format']) => {
    setSettings(prev => ({ ...prev, response_format }));
  }, []);

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
        setResponseFormat,
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