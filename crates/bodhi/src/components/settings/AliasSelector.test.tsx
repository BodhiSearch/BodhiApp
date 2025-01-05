import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { AliasSelector } from '@/components/settings/AliasSelector';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { createWrapper } from '@/tests/wrapper';

const server = setupServer();

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

const mockModels = {
  data: [
    {
      alias: 'gpt-4',
      model_file: 'gpt-4.gguf',
      parameters: {}
    },
    {
      alias: 'tinyllama-chat',
      model_file: 'chat-v1.0.Q4_K_M.gguf',
      parameters: {}
    }
  ],
  total: 2,
  page: 1,
  page_size: 100
};

describe('AliasSelector', () => {
  it('calls isLoadingCallback with true when loading starts', () => {
    const mockLoadingCallback = vi.fn();
    server.use(
      rest.get('*/api/ui/models', (_, res, ctx) => {
        return res(ctx.delay(100), ctx.json(mockModels));
      })
    );

    render(<AliasSelector isLoadingCallback={mockLoadingCallback} />, {
      wrapper: createWrapper()
    });

    expect(mockLoadingCallback).toHaveBeenCalledWith(true);
  });

  it('calls isLoadingCallback with false when loading completes', async () => {
    const mockLoadingCallback = vi.fn();
    server.use(
      rest.get('*/api/ui/models', (_, res, ctx) => {
        return res(ctx.json(mockModels));
      })
    );

    render(<AliasSelector isLoadingCallback={mockLoadingCallback} />, {
      wrapper: createWrapper()
    });

    await waitFor(() => {
      expect(mockLoadingCallback).toHaveBeenLastCalledWith(false);
    });
  });

  it('handles missing isLoadingCallback gracefully', () => {
    render(<AliasSelector />, { wrapper: createWrapper() });
    // Should not throw any errors
  });
});

describe('AliasSelector loaded', () => {
  beforeEach(() => {
    server.use(
      rest.get('*/api/ui/models', (_, res, ctx) => {
        return res(ctx.json(mockModels));
      })
    );
  });

  it('becomes enabled when data is loaded', async () => {
    render(<AliasSelector />, { wrapper: createWrapper() });

    await waitFor(() => {
      const select = screen.getByRole('combobox');
      expect(select).not.toBeDisabled();
    });
  });

  it('shows placeholder text in select when no initial value', async () => {
    render(<AliasSelector />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByTestId('model-selector-loaded')).toBeInTheDocument();
    });

    expect(screen.getByText('Select alias')).toBeInTheDocument();
  });

  it('initializes with provided initial alias', async () => {
    render(<AliasSelector initialAlias="gpt-4" />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByTestId('model-selector-loaded')).toBeInTheDocument();
    });

    expect(screen.getByText('gpt-4')).toBeInTheDocument();
  });

  it('calls onAliasChange when selection changes', async () => {
    const handleChange = vi.fn();
    render(
      <AliasSelector initialAlias="gpt-4" onAliasChange={handleChange} />,
      { wrapper: createWrapper() }
    );

    await waitFor(() => {
      expect(screen.getByTestId('model-selector-loaded')).toBeInTheDocument();
    });

    const select = screen.getByRole('combobox');
    fireEvent.click(select);

    const option = screen.getByText('tinyllama-chat');
    fireEvent.click(option);

    expect(handleChange).toHaveBeenCalledWith('tinyllama-chat');
  });

  it('handles missing onAliasChange prop gracefully', async () => {
    render(<AliasSelector initialAlias="gpt-4" />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByTestId('model-selector-loaded')).toBeInTheDocument();
    });

    const select = screen.getByRole('combobox');
    fireEvent.click(select);

    const option = screen.getByText('tinyllama-chat');
    expect(() => fireEvent.click(option)).not.toThrow();
  });
});