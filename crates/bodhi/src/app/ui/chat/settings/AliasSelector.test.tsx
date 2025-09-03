import { render, screen, fireEvent } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { AliasSelector } from '@/app/ui/chat/settings/AliasSelector';
import { createWrapper } from '@/tests/wrapper';
import * as chatSettings from '@/hooks/use-chat-settings';

// Mock useMediaQuery hook
vi.mock('@/hooks/use-media-query', () => ({
  useMediaQuery: (query: string) => {
    return true;
  },
}));
vi.mock('@/components/CopyButton', () => ({
  CopyButton: () => <div>Copy Button</div>,
}));

// Mock required HTMLElement methods and styles for Radix UI and Vaul components
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

// Mock useChatSettings
vi.mock('@/hooks/use-chat-settings', () => ({
  useChatSettings: vi.fn(),
}));

const mockModels = [
  {
    alias: 'gpt-4',
  },
  {
    alias: 'tinyllama-chat',
  },
];

const mockUnifiedModels = [
  {
    model_type: 'local',
    alias: 'local-model-1',
    repo: 'user/repo1',
    filename: 'model1.gguf',
  },
  {
    model_type: 'local',
    alias: 'local-model-2',
    repo: 'user/repo2',
    filename: 'model2.gguf',
  },
  {
    model_type: 'api',
    id: 'openai-api',
    provider: 'OpenAI',
    models: ['gpt-4', 'gpt-3.5-turbo'],
  },
  {
    model_type: 'api',
    id: 'anthropic-api',
    provider: 'Anthropic',
    models: ['claude-3-opus', 'claude-3-sonnet'],
  },
];

describe('AliasSelector', () => {
  beforeEach(() => {
    // Reset mock before each test
    vi.mocked(chatSettings.useChatSettings).mockReturnValue({
      model: '',
      setModel: vi.fn(),
    } as any);
  });

  it('renders in disabled state when loading', () => {
    render(<AliasSelector models={mockModels} isLoading={true} />, {
      wrapper: createWrapper(),
    });

    const select = screen.getByRole('combobox');
    expect(select).toBeDisabled();
  });

  it('renders in enabled state when not loading', () => {
    render(<AliasSelector models={mockModels} isLoading={false} />, {
      wrapper: createWrapper(),
    });

    const select = screen.getByRole('combobox');
    expect(select).not.toBeDisabled();
  });

  it('shows placeholder text when no model is selected', () => {
    render(<AliasSelector models={mockModels} />, {
      wrapper: createWrapper(),
    });

    expect(screen.getByText('Select alias')).toBeInTheDocument();
  });

  it('displays the current model from chat settings', () => {
    vi.mocked(chatSettings.useChatSettings).mockReturnValue({
      model: 'gpt-4',
      setModel: vi.fn(),
    } as any);

    render(<AliasSelector models={mockModels} />, {
      wrapper: createWrapper(),
    });

    expect(screen.getByText('gpt-4')).toBeInTheDocument();
  });

  it('calls setModel when selection changes', () => {
    const mockSetModel = vi.fn();
    vi.mocked(chatSettings.useChatSettings).mockReturnValue({
      model: '',
      setModel: mockSetModel,
    } as any);

    render(<AliasSelector models={mockModels} />, {
      wrapper: createWrapper(),
    });

    const select = screen.getByRole('combobox');
    fireEvent.click(select);

    const option = screen.getByText('tinyllama-chat');
    fireEvent.click(option);

    expect(mockSetModel).toHaveBeenCalledWith('tinyllama-chat');
  });

  it('renders all provided model options', () => {
    render(<AliasSelector models={mockModels} />, {
      wrapper: createWrapper(),
    });

    const select = screen.getByRole('combobox');
    fireEvent.click(select);

    mockModels.forEach((model) => {
      expect(screen.getByText(model.alias)).toBeInTheDocument();
    });
  });

  // New tests for unified model support (local + API models)
  describe('Unified Model Support', () => {
    it('expands API models to show individual model names with provider labels', () => {
      const apiOnlyModels = mockUnifiedModels.filter((m) => m.model_type === 'api');

      render(<AliasSelector models={apiOnlyModels} />, {
        wrapper: createWrapper(),
      });

      const select = screen.getByRole('combobox');
      fireEvent.click(select);

      // Should show individual API models with provider labels
      expect(screen.getByText('gpt-4 (OpenAI)')).toBeInTheDocument();
      expect(screen.getByText('gpt-3.5-turbo (OpenAI)')).toBeInTheDocument();
      expect(screen.getByText('claude-3-opus (Anthropic)')).toBeInTheDocument();
      expect(screen.getByText('claude-3-sonnet (Anthropic)')).toBeInTheDocument();
    });

    it('shows local models as individual entries with their alias names', () => {
      const localOnlyModels = mockUnifiedModels.filter((m) => m.model_type === 'local');

      render(<AliasSelector models={localOnlyModels} />, {
        wrapper: createWrapper(),
      });

      const select = screen.getByRole('combobox');
      fireEvent.click(select);

      // Should show local models with their alias names
      expect(screen.getByText('local-model-1')).toBeInTheDocument();
      expect(screen.getByText('local-model-2')).toBeInTheDocument();
    });

    it('handles mixed local and API models correctly', () => {
      render(<AliasSelector models={mockUnifiedModels} />, {
        wrapper: createWrapper(),
      });

      const select = screen.getByRole('combobox');
      fireEvent.click(select);

      // Should show local models with their alias names
      expect(screen.getByText('local-model-1')).toBeInTheDocument();
      expect(screen.getByText('local-model-2')).toBeInTheDocument();

      // Should show expanded API models with provider labels
      expect(screen.getByText('gpt-4 (OpenAI)')).toBeInTheDocument();
      expect(screen.getByText('gpt-3.5-turbo (OpenAI)')).toBeInTheDocument();
      expect(screen.getByText('claude-3-opus (Anthropic)')).toBeInTheDocument();
      expect(screen.getByText('claude-3-sonnet (Anthropic)')).toBeInTheDocument();
    });

    it('calls setModel with correct value when local model is selected', () => {
      const mockSetModel = vi.fn();
      vi.mocked(chatSettings.useChatSettings).mockReturnValue({
        model: '',
        setModel: mockSetModel,
      } as any);

      const localOnlyModels = mockUnifiedModels.filter((m) => m.model_type === 'local');

      render(<AliasSelector models={localOnlyModels} />, {
        wrapper: createWrapper(),
      });

      const select = screen.getByRole('combobox');
      fireEvent.click(select);

      const localModelOption = screen.getByText('local-model-1');
      fireEvent.click(localModelOption);

      // Should call setModel with the alias value
      expect(mockSetModel).toHaveBeenCalledWith('local-model-1');
    });

    it('calls setModel with correct value when API model is selected', () => {
      const mockSetModel = vi.fn();
      vi.mocked(chatSettings.useChatSettings).mockReturnValue({
        model: '',
        setModel: mockSetModel,
      } as any);

      const apiOnlyModels = mockUnifiedModels.filter((m) => m.model_type === 'api');

      render(<AliasSelector models={apiOnlyModels} />, {
        wrapper: createWrapper(),
      });

      const select = screen.getByRole('combobox');
      fireEvent.click(select);

      const apiModelOption = screen.getByText('gpt-4 (OpenAI)');
      fireEvent.click(apiModelOption);

      // Should call setModel with the actual model name (not the API id)
      expect(mockSetModel).toHaveBeenCalledWith('gpt-4');
    });

    it('correctly identifies and displays currently selected API model', () => {
      vi.mocked(chatSettings.useChatSettings).mockReturnValue({
        model: 'claude-3-opus',
        setModel: vi.fn(),
      } as any);

      const apiOnlyModels = mockUnifiedModels.filter((m) => m.model_type === 'api');

      render(<AliasSelector models={apiOnlyModels} />, {
        wrapper: createWrapper(),
      });

      // Should show the selected API model with provider label
      expect(screen.getByText('claude-3-opus (Anthropic)')).toBeInTheDocument();
    });

    it('correctly identifies and displays currently selected local model', () => {
      vi.mocked(chatSettings.useChatSettings).mockReturnValue({
        model: 'local-model-2',
        setModel: vi.fn(),
      } as any);

      const localOnlyModels = mockUnifiedModels.filter((m) => m.model_type === 'local');

      render(<AliasSelector models={localOnlyModels} />, {
        wrapper: createWrapper(),
      });

      // Should show the selected local model
      expect(screen.getByText('local-model-2')).toBeInTheDocument();
    });

    it('handles API models with empty models array', () => {
      const apiModelWithoutModels = [
        {
          model_type: 'api',
          id: 'empty-api',
          provider: 'Empty Provider',
          models: [],
        },
      ];

      render(<AliasSelector models={apiModelWithoutModels} />, {
        wrapper: createWrapper(),
      });

      const select = screen.getByRole('combobox');
      fireEvent.click(select);

      // Should not show any options for API models with no models
      expect(screen.queryByText('Empty Provider')).not.toBeInTheDocument();
    });

    it('falls back to displaying unknown selected model', () => {
      vi.mocked(chatSettings.useChatSettings).mockReturnValue({
        model: 'unknown-model-not-in-list',
        setModel: vi.fn(),
      } as any);

      render(<AliasSelector models={mockUnifiedModels} />, {
        wrapper: createWrapper(),
      });

      // Should show the unknown model name as fallback
      expect(screen.getByText('unknown-model-not-in-list')).toBeInTheDocument();
    });
  });
});
