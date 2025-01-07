import { render, screen, fireEvent } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { AliasSelector } from '@/components/settings/AliasSelector';
import { createWrapper } from '@/tests/wrapper';
import * as chatSettings from '@/hooks/use-chat-settings';

// Mock useChatSettings
vi.mock('@/hooks/use-chat-settings', () => ({
  useChatSettings: vi.fn()
}));

const mockModels = [
  {
    alias: 'gpt-4',
  },
  {
    alias: 'tinyllama-chat',
  }
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
      wrapper: createWrapper()
    });

    const select = screen.getByRole('combobox');
    expect(select).toBeDisabled();
  });

  it('renders in enabled state when not loading', () => {
    render(<AliasSelector models={mockModels} isLoading={false} />, {
      wrapper: createWrapper()
    });

    const select = screen.getByRole('combobox');
    expect(select).not.toBeDisabled();
  });

  it('shows placeholder text when no model is selected', () => {
    render(<AliasSelector models={mockModels} />, {
      wrapper: createWrapper()
    });

    expect(screen.getByText('Select alias')).toBeInTheDocument();
  });

  it('displays the current model from chat settings', () => {
    vi.mocked(chatSettings.useChatSettings).mockReturnValue({
      model: 'gpt-4',
      setModel: vi.fn(),
    } as any);

    render(<AliasSelector models={mockModels} />, {
      wrapper: createWrapper()
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
      wrapper: createWrapper()
    });

    const select = screen.getByRole('combobox');
    fireEvent.click(select);

    const option = screen.getByText('tinyllama-chat');
    fireEvent.click(option);

    expect(mockSetModel).toHaveBeenCalledWith('tinyllama-chat');
  });

  it('renders all provided model options', () => {
    render(<AliasSelector models={mockModels} />, {
      wrapper: createWrapper()
    });

    const select = screen.getByRole('combobox');
    fireEvent.click(select);

    mockModels.forEach(model => {
      expect(screen.getByText(model.alias)).toBeInTheDocument();
    });
  });
});