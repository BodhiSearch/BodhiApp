import { ModelSelector } from '@/components/ModelSelector';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

describe('ModelSelector', () => {
  const defaultProps = {
    selectedModels: [],
    availableModels: [],
    onModelSelect: vi.fn(),
    onModelRemove: vi.fn(),
    onModelsSelectAll: vi.fn(),
    onFetchModels: vi.fn(),
    isFetchingModels: false,
    canFetch: true,
  };

  describe('Model selection', () => {
    it('handles individual model selection', async () => {
      const user = userEvent.setup();
      const onModelSelect = vi.fn();

      render(
        <ModelSelector {...defaultProps} availableModels={['gpt-4', 'gpt-3.5-turbo']} onModelSelect={onModelSelect} />
      );

      const modelElement = screen.getByTestId('available-model-gpt-4');
      await user.click(modelElement);

      expect(onModelSelect).toHaveBeenCalledWith('gpt-4');
    });

    it('handles model removal', async () => {
      const user = userEvent.setup();
      const onModelRemove = vi.fn();

      render(<ModelSelector {...defaultProps} selectedModels={['gpt-4']} onModelRemove={onModelRemove} />);

      const removeButton = screen.getByTestId('remove-model-gpt-4');
      await user.click(removeButton);

      expect(onModelRemove).toHaveBeenCalledWith('gpt-4');
    });

    it('handles select all models', async () => {
      const user = userEvent.setup();
      const onModelsSelectAll = vi.fn();

      render(
        <ModelSelector
          {...defaultProps}
          availableModels={['gpt-4', 'gpt-3.5-turbo']}
          onModelsSelectAll={onModelsSelectAll}
        />
      );

      const selectAllButton = screen.getByTestId('select-all-models');
      await user.click(selectAllButton);

      expect(onModelsSelectAll).toHaveBeenCalledWith(['gpt-4', 'gpt-3.5-turbo']);
    });

    it('filters out already selected models from available list', () => {
      render(
        <ModelSelector
          {...defaultProps}
          selectedModels={['gpt-4']}
          availableModels={['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo']}
        />
      );

      expect(screen.queryByTestId('available-model-gpt-4')).not.toBeInTheDocument();
      expect(screen.getByTestId('available-model-gpt-3.5-turbo')).toBeInTheDocument();
      expect(screen.getByTestId('available-model-gpt-4-turbo')).toBeInTheDocument();
    });
  });

  describe('Model search', () => {
    it('filters models based on search term', async () => {
      const user = userEvent.setup();

      render(<ModelSelector {...defaultProps} availableModels={['gpt-4', 'gpt-3.5-turbo', 'claude-3-sonnet']} />);

      const searchInput = screen.getByTestId('model-search-input');
      await user.type(searchInput, 'gpt');

      expect(screen.getByTestId('available-model-gpt-4')).toBeInTheDocument();
      expect(screen.getByTestId('available-model-gpt-3.5-turbo')).toBeInTheDocument();
      expect(screen.queryByTestId('available-model-claude-3-sonnet')).not.toBeInTheDocument();
    });

    it('clears search when clear button is clicked', async () => {
      const user = userEvent.setup();

      render(<ModelSelector {...defaultProps} availableModels={['gpt-4', 'gpt-3.5-turbo', 'claude-3-sonnet']} />);

      const searchInput = screen.getByTestId('model-search-input');
      await user.type(searchInput, 'gpt');

      expect(screen.queryByTestId('available-model-claude-3-sonnet')).not.toBeInTheDocument();

      const clearButton = screen.getByTestId('clear-search-button');
      await user.click(clearButton);

      expect(searchInput).toHaveValue('');
      expect(screen.getByTestId('available-model-claude-3-sonnet')).toBeInTheDocument();
    });
  });

  describe('Fetch models', () => {
    it('calls onFetchModels when fetch button is clicked', async () => {
      const user = userEvent.setup();
      const onFetchModels = vi.fn();

      render(<ModelSelector {...defaultProps} onFetchModels={onFetchModels} />);

      const fetchButton = screen.getByTestId('fetch-models-button');
      await user.click(fetchButton);

      expect(onFetchModels).toHaveBeenCalled();
    });

    it('disables fetch button when canFetch is false', () => {
      render(<ModelSelector {...defaultProps} canFetch={false} />);

      const fetchButton = screen.getByTestId('fetch-models-button');
      expect(fetchButton).toBeDisabled();
    });

    it('shows loading state when fetching models', () => {
      render(<ModelSelector {...defaultProps} isFetchingModels={true} />);

      expect(screen.getByText('Fetching...')).toBeInTheDocument();
    });
  });

  describe('Clear all', () => {
    it('clears all selected models when clear all is clicked', async () => {
      const user = userEvent.setup();
      const onModelsSelectAll = vi.fn();

      render(
        <ModelSelector
          {...defaultProps}
          selectedModels={['gpt-4', 'gpt-3.5-turbo']}
          onModelsSelectAll={onModelsSelectAll}
        />
      );

      const clearAllButton = screen.getByTestId('clear-all-models');
      await user.click(clearAllButton);

      expect(onModelsSelectAll).toHaveBeenCalledWith([]);
    });

    it('does not show clear all button when no models selected', () => {
      render(<ModelSelector {...defaultProps} selectedModels={[]} />);

      expect(screen.queryByTestId('clear-all-models')).not.toBeInTheDocument();
    });
  });

  describe('Empty states', () => {
    it('shows "No models selected" when no models are selected', () => {
      render(<ModelSelector {...defaultProps} />);

      expect(screen.getByTestId('no-models-selected')).toBeInTheDocument();
      expect(screen.getByText('No models selected')).toBeInTheDocument();
    });

    it('shows "No models available" message when no models to fetch', () => {
      render(<ModelSelector {...defaultProps} availableModels={[]} />);

      expect(screen.getByTestId('empty-model-list')).toBeInTheDocument();
      expect(screen.getByText('No models available. Fetch models to see options.')).toBeInTheDocument();
    });

    it('shows "Loading models..." when fetching', () => {
      render(<ModelSelector {...defaultProps} isFetchingModels={true} availableModels={[]} />);

      expect(screen.getByText('Loading models...')).toBeInTheDocument();
    });
  });

  describe('Selected models display', () => {
    it('displays selected models count', () => {
      render(<ModelSelector {...defaultProps} selectedModels={['gpt-4', 'gpt-3.5-turbo', 'claude-3-sonnet']} />);

      expect(screen.getByText('Selected Models (3)')).toBeInTheDocument();
    });

    it('displays all selected models as badges', () => {
      render(<ModelSelector {...defaultProps} selectedModels={['gpt-4', 'gpt-3.5-turbo']} />);

      expect(screen.getByTestId('selected-model-gpt-4')).toBeInTheDocument();
      expect(screen.getByTestId('selected-model-gpt-3.5-turbo')).toBeInTheDocument();
    });
  });
});
