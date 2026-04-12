import { create } from 'zustand';
import type { ApiFormat } from '@bodhiapp/ts-client';

import { useChatStore } from './chatStore';
import type { PersistedChatSettings } from '@/lib/chatDb';

export type ChatSettings = PersistedChatSettings;

// Session-only fields (persisted to sessionStorage, not IndexedDB)
interface SessionOnlySettings {
  api_token?: string;
  api_token_enabled: boolean;
}

const SESSION_TOKEN_KEY = 'bodhi:api_token';
const SESSION_TOKEN_ENABLED_KEY = 'bodhi:api_token_enabled';

function loadSessionToken(): Partial<SessionOnlySettings> {
  if (typeof window === 'undefined') return {};
  try {
    const token = sessionStorage.getItem(SESSION_TOKEN_KEY) ?? undefined;
    const enabled = sessionStorage.getItem(SESSION_TOKEN_ENABLED_KEY) === 'true';
    return { api_token: token, api_token_enabled: enabled };
  } catch {
    return {};
  }
}

function persistSessionToken(token: string | undefined, enabled: boolean): void {
  if (typeof window === 'undefined') return;
  try {
    if (token !== undefined) {
      sessionStorage.setItem(SESSION_TOKEN_KEY, token);
    } else {
      sessionStorage.removeItem(SESSION_TOKEN_KEY);
    }
    sessionStorage.setItem(SESSION_TOKEN_ENABLED_KEY, String(enabled));
  } catch {
    // sessionStorage unavailable
  }
}

function clearSessionToken(): void {
  if (typeof window === 'undefined') return;
  try {
    sessionStorage.removeItem(SESSION_TOKEN_KEY);
    sessionStorage.removeItem(SESSION_TOKEN_ENABLED_KEY);
  } catch {
    // sessionStorage unavailable
  }
}

const defaultSessionSettings: SessionOnlySettings = {
  api_token_enabled: false,
};

export const defaultSettings: ChatSettings & SessionOnlySettings = {
  model: '',
  apiFormat: 'openai',
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
  response_format_enabled: false,
  maxToolIterations: 5,
  maxToolIterations_enabled: true,
  ...defaultSessionSettings,
};

type AllSettings = ChatSettings & SessionOnlySettings;
type SettingKey = keyof Omit<AllSettings, `${string}_enabled`>;
type EnabledKey = `${string}_enabled` & keyof AllSettings;

export interface ChatSettingsStoreState extends ChatSettings, SessionOnlySettings {
  setModel: (model: string) => void;
  setApiFormat: (format: ApiFormat) => void;
  setSetting: <K extends SettingKey>(key: K, value: AllSettings[K] | undefined) => void;
  setEnabled: (key: EnabledKey, enabled: boolean) => void;

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
  setApiToken: (token: string | undefined) => void;
  setApiTokenEnabled: (enabled: boolean) => void;
  setMaxToolIterations: (iterations: number | undefined) => void;
  setMaxToolIterationsEnabled: (enabled: boolean) => void;

  getRequestSettings: () => Record<string, unknown>;
  reset: (preserveModel?: boolean) => void;
  loadForChat: (chatId: string | null) => Promise<void>;
  saveForChat: (chatId: string) => Promise<void>;
}

function createSettingSetter(key: SettingKey) {
  return (value: unknown) => {
    useChatSettingsStore.getState().setSetting(key, value as never);
  };
}

function createEnabledSetter(key: EnabledKey) {
  return (enabled: boolean) => {
    useChatSettingsStore.getState().setEnabled(key, enabled);
  };
}

export const useChatSettingsStore = create<ChatSettingsStoreState>((set, get) => ({
  ...defaultSettings,
  ...loadSessionToken(),

  setModel: (model) => set({ model }),

  setApiFormat: (apiFormat) => set({ apiFormat }),

  setSetting: (key, value) => {
    if (value === undefined) {
      const update: Partial<ChatSettings> = { [key]: undefined };
      const enabledKey = `${key}_enabled` as EnabledKey;
      if (enabledKey in defaultSettings) {
        (update as Record<string, unknown>)[enabledKey] = false;
      }
      set(update as Partial<ChatSettingsStoreState>);
    } else {
      const update: Partial<ChatSettings> = { [key]: value };
      const enabledKey = `${key}_enabled` as EnabledKey;
      if (enabledKey in defaultSettings) {
        (update as Record<string, unknown>)[enabledKey] = true;
      }
      set(update as Partial<ChatSettingsStoreState>);
    }
  },

  setEnabled: (key, enabled) => {
    set({ [key]: enabled } as Partial<ChatSettingsStoreState>);
  },

  setTemperature: createSettingSetter('temperature') as (v: number | undefined) => void,
  setTemperatureEnabled: createEnabledSetter('temperature_enabled'),
  setTopP: createSettingSetter('top_p') as (v: number | undefined) => void,
  setTopPEnabled: createEnabledSetter('top_p_enabled'),
  setN: createSettingSetter('n') as (v: number | undefined) => void,
  setNEnabled: createEnabledSetter('n_enabled'),
  setStream: createSettingSetter('stream') as (v: boolean | undefined) => void,
  setStreamEnabled: createEnabledSetter('stream_enabled'),
  setMaxTokens: createSettingSetter('max_tokens') as (v: number | undefined) => void,
  setMaxTokensEnabled: createEnabledSetter('max_tokens_enabled'),
  setPresencePenalty: createSettingSetter('presence_penalty') as (v: number | undefined) => void,
  setPresencePenaltyEnabled: createEnabledSetter('presence_penalty_enabled'),
  setFrequencyPenalty: createSettingSetter('frequency_penalty') as (v: number | undefined) => void,
  setFrequencyPenaltyEnabled: createEnabledSetter('frequency_penalty_enabled'),
  setLogitBias: createSettingSetter('logit_bias') as (v: Record<string, number> | undefined) => void,
  setLogitBiasEnabled: createEnabledSetter('logit_bias_enabled'),
  setStop: (stop) => {
    if (stop === undefined) {
      get().setSetting('stop', undefined);
    } else {
      get().setSetting('stop', Array.isArray(stop) ? stop : [stop]);
    }
  },
  setStopEnabled: createEnabledSetter('stop_enabled'),
  setSeed: createSettingSetter('seed') as (v: number | undefined) => void,
  setSeedEnabled: createEnabledSetter('seed_enabled'),
  setSystemPrompt: createSettingSetter('systemPrompt') as (v: string | undefined) => void,
  setSystemPromptEnabled: createEnabledSetter('systemPrompt_enabled'),
  setResponseFormat: createSettingSetter('response_format') as (v: ChatSettings['response_format'] | undefined) => void,
  setResponseFormatEnabled: createEnabledSetter('response_format_enabled'),
  setApiToken: (token: string | undefined) => {
    set({ api_token: token, api_token_enabled: token !== undefined });
    persistSessionToken(token, token !== undefined);
  },
  setApiTokenEnabled: (enabled: boolean) => {
    set({ api_token_enabled: enabled });
    const token = get().api_token;
    persistSessionToken(token, enabled);
  },
  setMaxToolIterations: createSettingSetter('maxToolIterations') as (v: number | undefined) => void,
  setMaxToolIterationsEnabled: createEnabledSetter('maxToolIterations_enabled'),

  getRequestSettings: () => {
    const state = get();
    const requestSettings: Record<string, unknown> = {};

    if (state.model) {
      requestSettings.model = state.model;
    }

    const settingsToCheck: SettingKey[] = [
      'temperature',
      'top_p',
      'n',
      'stream',
      'max_tokens',
      'presence_penalty',
      'frequency_penalty',
      'logit_bias',
      'stop',
      'seed',
      'response_format',
    ];

    for (const key of settingsToCheck) {
      const value = state[key];
      const enabledKey = `${key}_enabled` as EnabledKey;
      const isEnabled = state[enabledKey];
      if (value !== undefined && isEnabled) {
        requestSettings[key] = value;
      }
    }

    return requestSettings;
  },

  reset: (preserveModel) => {
    const currentModel = get().model;
    clearSessionToken();
    set({
      ...defaultSettings,
      ...(preserveModel ? { model: currentModel } : {}),
    });
  },

  loadForChat: async (chatId) => {
    const sessionToken = loadSessionToken();
    if (!chatId) {
      // New chat: preserve the last active model/apiFormat so the user
      // continues with the same alias after deleting a chat or starting fresh.
      const currentModel = get().model;
      const currentApiFormat = get().apiFormat;
      set({
        ...defaultSettings,
        model: currentModel,
        apiFormat: currentApiFormat,
        ...sessionToken,
      });
      return;
    }
    const settings = await useChatStore.getState().getChatSettings(chatId);
    if (settings) {
      set({ ...defaultSettings, ...settings, ...sessionToken });
    } else {
      const currentModel = get().model;
      const currentApiFormat = get().apiFormat;
      set({
        ...defaultSettings,
        model: currentModel,
        apiFormat: currentApiFormat,
        ...sessionToken,
      });
    }
  },

  saveForChat: async (chatId) => {
    const state = get();
    const settings: PersistedChatSettings = {
      model: state.model,
      apiFormat: state.apiFormat,
      stream: state.stream,
      stream_enabled: state.stream_enabled,
      seed: state.seed,
      seed_enabled: state.seed_enabled,
      systemPrompt: state.systemPrompt,
      systemPrompt_enabled: state.systemPrompt_enabled,
      stop: state.stop,
      stop_enabled: state.stop_enabled,
      max_tokens: state.max_tokens,
      max_tokens_enabled: state.max_tokens_enabled,
      n: state.n,
      n_enabled: state.n_enabled,
      temperature: state.temperature,
      temperature_enabled: state.temperature_enabled,
      top_p: state.top_p,
      top_p_enabled: state.top_p_enabled,
      presence_penalty: state.presence_penalty,
      presence_penalty_enabled: state.presence_penalty_enabled,
      frequency_penalty: state.frequency_penalty,
      frequency_penalty_enabled: state.frequency_penalty_enabled,
      logit_bias: state.logit_bias,
      logit_bias_enabled: state.logit_bias_enabled,
      response_format: state.response_format,
      response_format_enabled: state.response_format_enabled,
      maxToolIterations: state.maxToolIterations,
      maxToolIterations_enabled: state.maxToolIterations_enabled,
    };
    await useChatStore.getState().saveChatSettings(chatId, settings);
  },
}));

// Cross-store subscription: when currentChatId changes, load settings for new chat
let _settingsUnsubscribe: (() => void) | null = null;
export function initChatSettingsSubscription() {
  _settingsUnsubscribe?.();
  _settingsUnsubscribe = useChatStore.subscribe((state, prevState) => {
    if (state.currentChatId !== prevState.currentChatId) {
      useChatSettingsStore.getState().loadForChat(state.currentChatId);
    }
  });
}
