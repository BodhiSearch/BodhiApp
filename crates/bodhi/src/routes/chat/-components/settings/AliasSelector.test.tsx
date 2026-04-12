import { render, screen, fireEvent } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { AliasResponse } from '@bodhiapp/ts-client';
import { AliasSelector } from '@/routes/chat/-components/settings/AliasSelector';
import { createWrapper } from '@/tests/wrapper';
import { useChatSettingsStore } from '@/stores/chatSettingsStore';

vi.mock('@/hooks/use-media-query', () => ({
  useMediaQuery: () => true,
}));
vi.mock('@/components/CopyButton', () => ({
  CopyButton: () => <div>Copy Button</div>,
}));

Object.assign(window.HTMLElement.prototype, {
  scrollIntoView: vi.fn(),
  releasePointerCapture: vi.fn(),
  hasPointerCapture: vi.fn(),
  setPointerCapture: vi.fn(),
  getBoundingClientRect: vi.fn().mockReturnValue({
    x: 0,
    y: 0,
    width: 0,
    height: 0,
    top: 0,
    right: 0,
    bottom: 0,
    left: 0,
  }),
});

vi.mock('@/stores/chatStore', () => {
  const { create } = require('zustand');
  return { useChatStore: create(() => ({ getChatSettings: vi.fn() })) };
});

vi.mock('@/stores/chatSettingsStore', () => {
  const { create } = require('zustand');
  const store = create(() => ({
    model: '',
    apiFormat: 'openai',
    setModel: vi.fn(),
    setApiFormat: vi.fn(),
  }));
  return { useChatSettingsStore: store };
});

const mockModels = [
  {
    source: 'user',
    alias: 'gpt-4',
    repo: 'test/repo',
    filename: 'model.gguf',
    snapshot: 'abc123',
    request_params: {},
    context_params: [],
    model_params: {},
  },
  {
    source: 'user',
    alias: 'tinyllama-chat',
    repo: 'test/repo',
    filename: 'model.gguf',
    snapshot: 'def456',
    request_params: {},
    context_params: [],
    model_params: {},
  },
];

const mockUnifiedModels: AliasResponse[] = [
  {
    source: 'user',
    id: 'local-1',
    alias: 'local-model-1',
    repo: 'user/repo1',
    filename: 'model1.gguf',
    snapshot: 'abc123',
    request_params: {},
    context_params: [],
    model_params: {},
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
  {
    source: 'model',
    alias: 'local-model-2',
    repo: 'user/repo2',
    filename: 'model2.gguf',
    snapshot: 'def456',
  },
  {
    source: 'api',
    id: 'openai-api',
    api_format: 'openai' as const,
    base_url: 'https://api.openai.com/v1',
    has_api_key: true,
    models: [
      { id: 'gpt-4', object: 'model', created: 0, owned_by: 'openai', provider: 'openai' },
      { id: 'gpt-3.5-turbo', object: 'model', created: 0, owned_by: 'openai', provider: 'openai' },
    ],
    forward_all_with_prefix: false,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
  {
    source: 'api',
    id: 'anthropic-api',
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
      {
        id: 'claude-3-sonnet',
        display_name: 'Claude 3 Sonnet',
        created_at: '2024-01-01T00:00:00Z',
        type: 'model',
        provider: 'anthropic' as const,
      },
    ],
    forward_all_with_prefix: false,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
];

describe('AliasSelector', () => {
  beforeEach(() => {
    useChatSettingsStore.setState({
      model: '',
      setModel: vi.fn(),
      setApiFormat: vi.fn(),
    });
  });

  it('renders in disabled state when loading', () => {
    render(<AliasSelector models={mockModels} isLoading={true} tooltip="Select a model" />, {
      wrapper: createWrapper(),
    });

    const select = screen.getByRole('combobox');
    expect(select).toBeDisabled();
  });

  it('renders in enabled state when not loading', () => {
    render(<AliasSelector models={mockModels} isLoading={false} tooltip="Select a model" />, {
      wrapper: createWrapper(),
    });

    const select = screen.getByRole('combobox');
    expect(select).not.toBeDisabled();
  });

  it('shows placeholder text when no model is selected', () => {
    render(<AliasSelector models={mockModels} tooltip="Select a model" />, {
      wrapper: createWrapper(),
    });

    expect(screen.getByText('Select alias')).toBeInTheDocument();
  });

  it('displays the current model from chat settings', () => {
    useChatSettingsStore.setState({ model: 'gpt-4' });

    render(<AliasSelector models={mockModels} tooltip="Select a model" />, {
      wrapper: createWrapper(),
    });

    expect(screen.getByText('gpt-4')).toBeInTheDocument();
  });

  it('calls setModel when selection changes', () => {
    const mockSetModel = vi.fn();
    useChatSettingsStore.setState({ model: '', setModel: mockSetModel });

    render(<AliasSelector models={mockModels} tooltip="Select a model" />, {
      wrapper: createWrapper(),
    });

    const select = screen.getByRole('combobox');
    fireEvent.click(select);

    const option = screen.getByText('tinyllama-chat');
    fireEvent.click(option);

    expect(mockSetModel).toHaveBeenCalledWith('tinyllama-chat');
  });

  it('renders all provided model options', () => {
    render(<AliasSelector models={mockModels} tooltip="Select a model" />, {
      wrapper: createWrapper(),
    });

    const select = screen.getByRole('combobox');
    fireEvent.click(select);

    mockModels.forEach((model) => {
      expect(screen.getByText(model.alias)).toBeInTheDocument();
    });
  });

  describe('Unified Model Support', () => {
    it('expands API models to show individual model names with api_format labels', () => {
      const apiOnlyModels = mockUnifiedModels.filter((m) => m.source === 'api');

      render(<AliasSelector models={apiOnlyModels} tooltip="Select a model" />, {
        wrapper: createWrapper(),
      });

      const select = screen.getByRole('combobox');
      fireEvent.click(select);

      expect(screen.getByText('gpt-4')).toBeInTheDocument();
      expect(screen.getByText('gpt-3.5-turbo')).toBeInTheDocument();
      expect(screen.getByText('claude-3-opus')).toBeInTheDocument();
      expect(screen.getByText('claude-3-sonnet')).toBeInTheDocument();
    });

    it('shows local models as individual entries with their alias names', () => {
      const localOnlyModels = mockUnifiedModels.filter((m) => m.source === 'user' || m.source === 'model');

      render(<AliasSelector models={localOnlyModels} tooltip="Select a model" />, {
        wrapper: createWrapper(),
      });

      const select = screen.getByRole('combobox');
      fireEvent.click(select);

      expect(screen.getByText('local-model-1')).toBeInTheDocument();
      expect(screen.getByText('local-model-2')).toBeInTheDocument();
    });

    it('handles mixed local and API models correctly', () => {
      render(<AliasSelector models={mockUnifiedModels} tooltip="Select a model" />, {
        wrapper: createWrapper(),
      });

      const select = screen.getByRole('combobox');
      fireEvent.click(select);

      expect(screen.getByText('local-model-1')).toBeInTheDocument();
      expect(screen.getByText('local-model-2')).toBeInTheDocument();

      expect(screen.getByText('gpt-4')).toBeInTheDocument();
      expect(screen.getByText('gpt-3.5-turbo')).toBeInTheDocument();
      expect(screen.getByText('claude-3-opus')).toBeInTheDocument();
      expect(screen.getByText('claude-3-sonnet')).toBeInTheDocument();
    });

    it('calls setModel with correct value when local model is selected', () => {
      const mockSetModel = vi.fn();
      useChatSettingsStore.setState({ model: '', setModel: mockSetModel });

      const localOnlyModels = mockUnifiedModels.filter((m) => m.source === 'user' || m.source === 'model');

      render(<AliasSelector models={localOnlyModels} tooltip="Select a model" />, {
        wrapper: createWrapper(),
      });

      const select = screen.getByRole('combobox');
      fireEvent.click(select);

      const localModelOption = screen.getByText('local-model-1');
      fireEvent.click(localModelOption);

      expect(mockSetModel).toHaveBeenCalledWith('local-model-1');
    });

    it('calls setModel with correct value when API model is selected', () => {
      const mockSetModel = vi.fn();
      useChatSettingsStore.setState({ model: '', setModel: mockSetModel });

      const apiOnlyModels = mockUnifiedModels.filter((m) => m.source === 'api');

      render(<AliasSelector models={apiOnlyModels} tooltip="Select a model" />, {
        wrapper: createWrapper(),
      });

      const select = screen.getByRole('combobox');
      fireEvent.click(select);

      const apiModelOption = screen.getByText('gpt-4');
      fireEvent.click(apiModelOption);

      expect(mockSetModel).toHaveBeenCalledWith('gpt-4');
    });

    it('correctly identifies and displays currently selected API model', () => {
      useChatSettingsStore.setState({ model: 'claude-3-opus' });

      const apiOnlyModels = mockUnifiedModels.filter((m) => m.source === 'api');

      render(<AliasSelector models={apiOnlyModels} tooltip="Select a model" />, {
        wrapper: createWrapper(),
      });

      expect(screen.getByText('claude-3-opus')).toBeInTheDocument();
    });

    it('correctly identifies and displays currently selected local model', () => {
      useChatSettingsStore.setState({ model: 'local-model-2' });

      const localOnlyModels = mockUnifiedModels.filter((m) => m.source === 'user' || m.source === 'model');

      render(<AliasSelector models={localOnlyModels} tooltip="Select a model" />, {
        wrapper: createWrapper(),
      });

      expect(screen.getByText('local-model-2')).toBeInTheDocument();
    });

    it('handles API models with empty models array', () => {
      const apiModelWithoutModels = [
        {
          source: 'api',
          id: 'empty-api',
          api_format: 'openai' as const,
          base_url: 'https://api.example.com/v1',
          has_api_key: false,
          models: [],
          forward_all_with_prefix: false,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      ];

      render(<AliasSelector models={apiModelWithoutModels} tooltip="Select a model" />, {
        wrapper: createWrapper(),
      });

      const select = screen.getByRole('combobox');
      fireEvent.click(select);

      expect(screen.queryByText('openai')).not.toBeInTheDocument();
    });

    it.each([
      {
        apiFormat: 'openai' as const,
        modelId: 'gpt-4',
        models: [{ id: 'gpt-4', object: 'model', created: 0, owned_by: 'openai', provider: 'openai' as const }],
      },
      {
        apiFormat: 'openai_responses' as const,
        modelId: 'gpt-4o',
        models: [{ id: 'gpt-4o', object: 'model', created: 0, owned_by: 'openai', provider: 'openai' as const }],
      },
      {
        apiFormat: 'anthropic' as const,
        modelId: 'claude-3-opus',
        models: [
          {
            id: 'claude-3-opus',
            display_name: 'Claude 3 Opus',
            created_at: '2024-01-01T00:00:00Z',
            type: 'model',
            provider: 'anthropic' as const,
          },
        ],
      },
      {
        apiFormat: 'anthropic_oauth' as const,
        modelId: 'claude-3-haiku-20240307',
        models: [
          {
            id: 'claude-3-haiku-20240307',
            display_name: 'Claude 3 Haiku',
            created_at: '2024-01-01T00:00:00Z',
            type: 'model',
            provider: 'anthropic' as const,
          },
        ],
      },
    ])('calls setApiFormat with $apiFormat when API model is selected', ({ apiFormat, modelId, models }) => {
      const mockSetApiFormat = vi.fn();
      useChatSettingsStore.setState({ model: '', setModel: vi.fn(), setApiFormat: mockSetApiFormat });

      const apiModels = [
        {
          source: 'api',
          id: 'test-api',
          api_format: apiFormat,
          base_url: 'https://api.example.com/v1',
          has_api_key: true,
          models,
          forward_all_with_prefix: false,
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      ];

      render(<AliasSelector models={apiModels} tooltip="Select a model" />, {
        wrapper: createWrapper(),
      });

      const select = screen.getByRole('combobox');
      fireEvent.click(select);
      fireEvent.click(screen.getByText(modelId));

      expect(mockSetApiFormat).toHaveBeenCalledWith(apiFormat);
    });

    it('calls setApiFormat with openai when local model is selected', () => {
      const mockSetApiFormat = vi.fn();
      useChatSettingsStore.setState({ model: '', setModel: vi.fn(), setApiFormat: mockSetApiFormat });

      const localModels = [
        {
          source: 'user',
          alias: 'my-local-model',
          repo: 'test/repo',
          filename: 'model.gguf',
          snapshot: 'abc123',
          request_params: {},
          context_params: [],
          model_params: {},
        },
      ];

      render(<AliasSelector models={localModels} tooltip="Select a model" />, {
        wrapper: createWrapper(),
      });

      const select = screen.getByRole('combobox');
      fireEvent.click(select);
      fireEvent.click(screen.getByText('my-local-model'));

      expect(mockSetApiFormat).toHaveBeenCalledWith('openai');
    });

    it('falls back to displaying unknown selected model', () => {
      useChatSettingsStore.setState({ model: 'unknown-model-not-in-list' });

      render(<AliasSelector models={mockUnifiedModels} tooltip="Select a model" />, {
        wrapper: createWrapper(),
      });

      expect(screen.getByText('unknown-model-not-in-list')).toBeInTheDocument();
    });
  });
});
