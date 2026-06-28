import { render, screen, fireEvent } from '@testing-library/react';
import { describe, expect, it, vi, beforeEach } from 'vitest';
import type { AliasResponse } from '@bodhiapp/ts-client';
import { AliasSelector } from '@/routes/chat/-components/settings/AliasSelector';
import { createWrapper } from '@/tests/wrapper';
import { useChatSettingsStore } from '@/stores/chatSettingsStore';

vi.mock('@/components/CopyButton', () => ({
  CopyButton: () => <div>Copy Button</div>,
}));

const setModel = vi.fn();
const setApiFormat = vi.fn();
const setLlmLibertyProvider = vi.fn();

vi.mock('@/stores/chatStore', () => {
  const { create } = require('zustand');
  return { useChatStore: create(() => ({ getChatSettings: vi.fn() })) };
});

vi.mock('@/stores/chatSettingsStore', () => {
  const { create } = require('zustand');
  const store = create(() => ({
    model: '',
    apiFormat: 'openai',
    llmLibertyProvider: null,
    setModel: (...a: unknown[]) => setModel(...a),
    setApiFormat: (...a: unknown[]) => setApiFormat(...a),
    setLlmLibertyProvider: (...a: unknown[]) => setLlmLibertyProvider(...a),
  }));
  return { useChatSettingsStore: store };
});

const localModels: AliasResponse[] = [
  {
    source: 'user',
    alias: 'gpt-4',
    repo: 'test/repo',
    filename: 'model.gguf',
    snapshot: 'abc123',
    request_params: {},
    context_params: [],
    model_params: {},
  } as AliasResponse,
  {
    source: 'user',
    alias: 'tinyllama-chat',
    repo: 'test/repo',
    filename: 'model.gguf',
    snapshot: 'def456',
    request_params: {},
    context_params: [],
    model_params: {},
  } as AliasResponse,
];

const apiModels: AliasResponse[] = [
  {
    source: 'model',
    alias: 'local-model-2',
    repo: 'user/repo2',
    filename: 'model2.gguf',
    snapshot: 'def456',
  } as AliasResponse,
  {
    source: 'api',
    id: 'openai-api',
    name: 'OpenAI API',
    api_format: 'openai' as const,
    base_url: 'https://api.openai.com/v1',
    has_api_key: true,
    models: [{ id: 'gpt-4', object: 'model', created: 0, owned_by: 'openai', provider: 'openai' }],
    forward_all_with_prefix: false,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  } as AliasResponse,
  {
    source: 'api',
    id: 'anthropic-api',
    name: 'Anthropic API',
    api_format: 'anthropic' as const,
    base_url: 'https://api.anthropic.com/v1',
    has_api_key: true,
    models: [
      {
        id: 'claude-3-opus',
        display_name: 'Claude 3 Opus',
        created_at: '2024-01-01T00:00:00Z',
        type: 'model',
        provider: 'anthropic' as const,
      },
    ],
    forward_all_with_prefix: false,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  } as AliasResponse,
];

const renderSelector = (props: Partial<Parameters<typeof AliasSelector>[0]> = {}) =>
  render(<AliasSelector models={localModels} tooltip="Pick a model" {...props} />, { wrapper: createWrapper() });

const input = () => screen.getByTestId('model-selector-trigger') as HTMLInputElement;

describe('AliasSelector (free-text autocomplete)', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useChatSettingsStore.setState({ model: '', apiFormat: 'openai', llmLibertyProvider: null });
  });

  it('disables the input while loading', () => {
    renderSelector({ isLoading: true });
    expect(input()).toBeDisabled();
  });

  it('shows the placeholder when no model is selected', () => {
    renderSelector();
    expect(input()).toHaveValue('');
    expect(input()).toHaveAttribute('placeholder', expect.stringContaining('model name'));
  });

  it('reflects the current model from chat settings', () => {
    useChatSettingsStore.setState({ model: 'tinyllama-chat' });
    renderSelector();
    expect(input()).toHaveValue('tinyllama-chat');
  });

  it('opens the suggestion list on focus and lists all models', () => {
    renderSelector();
    fireEvent.focus(input());
    expect(screen.getByTestId('combobox-option-gpt-4')).toBeInTheDocument();
    expect(screen.getByTestId('combobox-option-tinyllama-chat')).toBeInTheDocument();
  });

  it('commits a selected suggestion via setModel', () => {
    renderSelector();
    fireEvent.focus(input());
    fireEvent.click(screen.getByTestId('combobox-option-tinyllama-chat'));
    expect(setModel).toHaveBeenCalledWith('tinyllama-chat');
  });

  it('accepts free-typed text that is not in the list (any value allowed)', () => {
    renderSelector();
    fireEvent.change(input(), { target: { value: 'some/custom-model:latest' } });
    expect(setModel).toHaveBeenCalledWith('some/custom-model:latest');
  });

  describe('unified API + local models', () => {
    it('expands API models into per-model entries alongside local ones', () => {
      renderSelector({ models: apiModels });
      fireEvent.focus(input());
      expect(screen.getByTestId('combobox-option-local-model-2')).toBeInTheDocument();
      // No alias prefix on these fixtures → bare model ids.
      expect(screen.getByTestId('combobox-option-gpt-4')).toBeInTheDocument();
      expect(screen.getByTestId('combobox-option-claude-3-opus')).toBeInTheDocument();
    });

    it('sets apiFormat=openai and clears the liberty provider when a local model is picked', () => {
      renderSelector({ models: apiModels });
      fireEvent.focus(input());
      fireEvent.click(screen.getByTestId('combobox-option-local-model-2'));
      expect(setModel).toHaveBeenCalledWith('local-model-2');
      expect(setApiFormat).toHaveBeenCalledWith('openai');
      expect(setLlmLibertyProvider).toHaveBeenCalledWith(null);
    });

    it('sets the API model api_format when an API model is picked', () => {
      renderSelector({ models: apiModels });
      fireEvent.focus(input());
      fireEvent.click(screen.getByTestId('combobox-option-claude-3-opus'));
      expect(setModel).toHaveBeenCalledWith('claude-3-opus');
      expect(setApiFormat).toHaveBeenCalledWith('anthropic');
    });
  });

  it('falls back to openai format for a free-typed unknown model', () => {
    renderSelector({ models: apiModels });
    fireEvent.change(input(), { target: { value: 'unknown-model-not-in-list' } });
    expect(setModel).toHaveBeenCalledWith('unknown-model-not-in-list');
    expect(setApiFormat).toHaveBeenCalledWith('openai');
    expect(setLlmLibertyProvider).toHaveBeenCalledWith(null);
  });

  describe('keyboard navigation', () => {
    it('highlights the first option on ArrowDown and selects it on Enter', () => {
      // models sort A→Z: gpt-4, tinyllama-chat
      renderSelector();
      fireEvent.focus(input());
      fireEvent.keyDown(input(), { key: 'ArrowDown' });
      expect(screen.getByTestId('combobox-option-gpt-4').className).toContain('active');

      fireEvent.keyDown(input(), { key: 'Enter' });
      expect(setModel).toHaveBeenCalledWith('gpt-4');
    });

    it('moves the highlight down through the list', () => {
      renderSelector();
      fireEvent.focus(input());
      fireEvent.keyDown(input(), { key: 'ArrowDown' });
      fireEvent.keyDown(input(), { key: 'ArrowDown' });
      expect(screen.getByTestId('combobox-option-tinyllama-chat').className).toContain('active');
    });

    it('wraps to the last option on ArrowUp from the top', () => {
      renderSelector();
      fireEvent.focus(input());
      fireEvent.keyDown(input(), { key: 'ArrowUp' });
      expect(screen.getByTestId('combobox-option-tinyllama-chat').className).toContain('active');
    });

    it('opens the list on ArrowDown when closed', () => {
      renderSelector();
      // Not opened yet — options absent.
      expect(screen.queryByTestId('combobox-option-gpt-4')).not.toBeInTheDocument();
      fireEvent.keyDown(input(), { key: 'ArrowDown' });
      expect(screen.getByTestId('combobox-option-gpt-4')).toBeInTheDocument();
    });

    it('Enter with no highlight keeps the typed text (commits via onChange, not selection)', () => {
      renderSelector();
      fireEvent.change(input(), { target: { value: 'free-typed' } });
      setModel.mockClear();
      fireEvent.keyDown(input(), { key: 'Enter' });
      // No option was highlighted, so Enter does not re-commit a list value.
      expect(setModel).not.toHaveBeenCalled();
    });
  });
});
