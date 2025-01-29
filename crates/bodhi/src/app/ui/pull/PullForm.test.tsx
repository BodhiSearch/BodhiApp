import { PullForm } from '@/app/ui/pull/PullForm';
import { ENDPOINT_MODEL_FILES, ENDPOINT_MODEL_FILES_PULL } from '@/hooks/useQuery';
import { createWrapper } from '@/tests/wrapper';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { afterAll, afterEach, beforeAll, describe, expect, it, vi } from 'vitest';

const mockToast = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast }),
}));

const mockModelsResponse = {
  data: [
    { repo: 'test/repo1', filename: 'model1.gguf' },
    { repo: 'test/repo1', filename: 'model2.gguf' },
    { repo: 'test/repo2', filename: 'model3.gguf' },
  ],
  total: 3,
  page: 1,
  page_size: 100,
};

const server = setupServer(
  rest.get(`*${ENDPOINT_MODEL_FILES}`, (_, res, ctx) => {
    return res(ctx.json(mockModelsResponse));
  }),
  rest.post(`*${ENDPOINT_MODEL_FILES_PULL}`, (_, res, ctx) => {
    return res(
      ctx.status(201),
      ctx.json({
        id: '123',
        repo: 'test/repo1',
        filename: 'model1.gguf',
        status: 'pending',
        updated_at: new Date().toISOString(),
      })
    );
  })
);

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => {
  server.resetHandlers();
  mockToast.mockClear();
});

describe('PullForm', () => {
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
      expect(mockToast).toHaveBeenCalledWith({
        title: 'Success',
        description: 'Model pull request submitted successfully',
        duration: 5000,
      });
    });

    // Form should be reset after successful submission
    expect(screen.getByLabelText(/repository/i)).toHaveValue('');
    expect(screen.getByLabelText(/filename/i)).toHaveValue('');
  });

  it('handles API error and shows error message', async () => {
    server.use(
      rest.post(`*${ENDPOINT_MODEL_FILES_PULL}`, (_, res, ctx) => {
        return res(
          ctx.status(400),
          ctx.json({
            error: {
              message: 'file "model.gguf" already exists in repo "test/repo" with snapshot "main"',
              type: 'invalid_request_error',
              code: 'pull_error-file_already_exists'
            }
          })
        );
      })
    );

    render(<PullForm />, { wrapper: createWrapper() });

    await userEvent.type(screen.getByLabelText(/repository/i), 'test/repo');
    await userEvent.type(screen.getByLabelText(/filename/i), 'model.gguf');

    const submitButton = screen.getByRole('button', { name: /pull model/i });
    await userEvent.click(submitButton);

    // Check for toast error message
    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith({
        title: 'Error',
        description: 'file "model.gguf" already exists in repo "test/repo" with snapshot "main"',
        variant: 'destructive',
      });
    });

    // Check that form fields are in error state
    const formMessages = screen.getAllByRole('alert');
    expect(formMessages).toHaveLength(1); // One for each field
    formMessages.forEach(message => {
      expect(message).toHaveTextContent(
        'file "model.gguf" already exists in repo "test/repo" with snapshot "main"'
      );
    });
  });

  it('resets form on reset button click', async () => {
    render(<PullForm />, { wrapper: createWrapper() });

    await userEvent.type(screen.getByLabelText(/repository/i), 'test/repo1');
    await userEvent.type(screen.getByLabelText(/filename/i), 'model1.gguf');

    // Submit with errors to show error state
    server.use(
      rest.post(`*${ENDPOINT_MODEL_FILES_PULL}`, (_, res, ctx) => {
        return res(
          ctx.status(400),
          ctx.json({
            error: {
              message: 'file "model1.gguf" already exists in repo "test/repo1" with snapshot "main"',
              type: 'invalid_request_error',
              code: 'pull_error-file_already_exists'
            }
          })
        );
      })
    );
    await userEvent.click(screen.getByRole('button', { name: /pull model/i }));

    // Wait for error message in toast
    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith({
        title: 'Error',
        description: 'file "model1.gguf" already exists in repo "test/repo1" with snapshot "main"',
        variant: 'destructive',
      });
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