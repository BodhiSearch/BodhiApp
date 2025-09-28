import { PullForm } from '@/app/ui/pull/PullForm';
import { ENDPOINT_MODEL_FILES, ENDPOINT_MODEL_FILES_PULL } from '@/hooks/useQuery';
import { showErrorParams, showSuccessParams } from '@/lib/utils.test';
import { createWrapper } from '@/tests/wrapper';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { setupMswV2, server } from '@/test-utils/msw-v2/setup';
import { mockModelFiles, mockModelPull, mockModelPullFileExistsError } from '@/test-utils/msw-v2/handlers/modelfiles';
import { afterEach, describe, expect, it, vi } from 'vitest';

const mockToast = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast }),
}));

setupMswV2();

afterEach(() => {
  mockToast.mockClear();
});

describe('PullForm', () => {
  beforeEach(() => {
    server.use(
      ...mockModelFiles({
        data: [
          { repo: 'test/repo1', filename: 'model1.gguf', size: 1073741824, snapshot: 'abc123', model_params: {} },
          { repo: 'test/repo1', filename: 'model2.gguf', size: 1073741824, snapshot: 'abc123', model_params: {} },
          { repo: 'test/repo2', filename: 'model3.gguf', size: 1073741824, snapshot: 'abc123', model_params: {} },
        ],
        total: 3,
        page: 1,
        page_size: 100,
      }),
      ...mockModelPull()
    );
  });

  it('renders form fields', () => {
    render(<PullForm />, { wrapper: createWrapper() });

    expect(screen.getByLabelText(/repository/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/filename/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /pull model/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /reset/i })).toBeInTheDocument();
  });

  it('shows validation errors for empty fields', async () => {
    render(<PullForm />, { wrapper: createWrapper() });

    const submitButton = screen.getByRole('button', { name: /pull model/i });
    await userEvent.click(submitButton);

    expect(await screen.findByText('Repository is required')).toBeInTheDocument();
    expect(screen.getByText('Filename is required')).toBeInTheDocument();
  });

  it('successfully submits form with valid data', async () => {
    render(<PullForm />, { wrapper: createWrapper() });

    await userEvent.type(screen.getByLabelText(/repository/i), 'test/repo1');
    await userEvent.type(screen.getByLabelText(/filename/i), 'model1.gguf');

    const submitButton = screen.getByRole('button', { name: /pull model/i });
    await userEvent.click(submitButton);

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'Model pull request submitted successfully'));
    });

    // Form should be reset after successful submission
    expect(screen.getByLabelText(/repository/i)).toHaveValue('');
    expect(screen.getByLabelText(/filename/i)).toHaveValue('');
  });

  it('handles API error and shows error message', async () => {
    server.use(...mockModelPullFileExistsError({ repo: 'test/repo', filename: 'model.gguf' }));

    render(<PullForm />, { wrapper: createWrapper() });

    await userEvent.type(screen.getByLabelText(/repository/i), 'test/repo');
    await userEvent.type(screen.getByLabelText(/filename/i), 'model.gguf');

    const submitButton = screen.getByRole('button', { name: /pull model/i });
    await userEvent.click(submitButton);

    // Check for toast error message
    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith(
        showErrorParams('Error', 'file "model.gguf" already exists in repo "test/repo" with snapshot "main"')
      );
    });

    // Check that form fields are in error state
    const formMessages = screen.getAllByRole('alert');
    expect(formMessages).toHaveLength(1); // One for each field
    formMessages.forEach((message) => {
      expect(message).toHaveTextContent('file "model.gguf" already exists in repo "test/repo" with snapshot "main"');
    });
  });

  it('resets form on reset button click', async () => {
    render(<PullForm />, { wrapper: createWrapper() });

    await userEvent.type(screen.getByLabelText(/repository/i), 'test/repo1');
    await userEvent.type(screen.getByLabelText(/filename/i), 'model1.gguf');

    // Submit with errors to show error state
    server.use(...mockModelPullFileExistsError({ repo: 'test/repo1', filename: 'model1.gguf' }));
    await userEvent.click(screen.getByRole('button', { name: /pull model/i }));

    // Wait for error message in toast
    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith(
        showErrorParams('Error', 'file "model1.gguf" already exists in repo "test/repo1" with snapshot "main"')
      );
    });

    // Verify error state before reset
    const errorMessages = screen.getAllByRole('alert');
    expect(errorMessages).toHaveLength(1);

    // Click reset
    const resetButton = screen.getByRole('button', { name: /reset/i });
    await userEvent.click(resetButton);

    // Form should be cleared
    expect(screen.getByLabelText(/repository/i)).toHaveValue('');
    expect(screen.getByLabelText(/filename/i)).toHaveValue('');

    // Error messages should be removed
    expect(screen.queryAllByRole('alert')).toHaveLength(0);
  });

  it('shows autocomplete suggestions', async () => {
    render(<PullForm />, { wrapper: createWrapper() });

    const repoInput = screen.getByLabelText(/repository/i);
    await userEvent.type(repoInput, 'test');

    // Should show repo suggestions
    await waitFor(() => {
      expect(screen.getByText('test/repo1')).toBeInTheDocument();
      expect(screen.getByText('test/repo2')).toBeInTheDocument();
    });

    // Select a repo and check filename suggestions
    await userEvent.click(screen.getByText('test/repo1'));
    const filenameInput = screen.getByLabelText(/filename/i);
    await userEvent.type(filenameInput, 'model');

    await waitFor(() => {
      expect(screen.getByText('model1.gguf')).toBeInTheDocument();
      expect(screen.getByText('model2.gguf')).toBeInTheDocument();
    });
  });
});
