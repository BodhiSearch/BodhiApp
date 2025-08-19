import Setup from '@/app/ui/setup/page';
import { ENDPOINT_APP_INFO, ENDPOINT_APP_SETUP, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
import { showErrorParams } from '@/lib/utils.test';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

const mockToast = vi.fn();
const pushMock = vi.fn();

vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast }),
}));

const server = setupServer();

beforeAll(() => server.listen());
afterEach(() => {
  server.resetHandlers();
  vi.clearAllMocks();
  pushMock.mockClear();
  mockToast.mockClear();
});
afterAll(() => server.close());

describe('Setup Page', () => {
  beforeEach(() => {
    Object.defineProperty(window, 'localStorage', {
      value: {
        getItem: vi.fn(() => null),
        setItem: vi.fn(),
        removeItem: vi.fn(),
        clear: vi.fn(),
      },
      writable: true,
    });

    // Setup default handlers
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'setup' }));
      }),
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ logged_in: false }));
      }),
      rest.post(`*${ENDPOINT_APP_SETUP}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      })
    );
  });

  it('should render setup form and handle successful submission with redirect to download models', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<Setup />, { wrapper: createWrapper() });
    });

    // Verify form is rendered
    expect(screen.getByTestId('setup-form')).toBeInTheDocument();
    expect(screen.getByLabelText(/server name/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/description/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /setup bodhi server/i })).toBeInTheDocument();

    // Fill out the form with valid data (minimum 10 characters for name)
    await user.type(screen.getByLabelText(/server name/i), 'My Test Server Instance');
    await user.type(screen.getByLabelText(/description/i), 'Test description for my server');

    // Submit the form
    await user.click(screen.getByRole('button', { name: /setup bodhi server/i }));

    // Wait for the API call and redirect
    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup/download-models');
    });
  });

  it('should redirect to resource admin when setup returns resource-admin status', async () => {
    server.use(
      rest.post(`*${ENDPOINT_APP_SETUP}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'resource-admin' }));
      })
    );

    const user = userEvent.setup();

    await act(async () => {
      render(<Setup />, { wrapper: createWrapper() });
    });

    // Fill out the form with valid data (minimum 10 characters for name)
    await user.type(screen.getByLabelText(/server name/i), 'My Test Server Instance');

    // Submit the form
    await user.click(screen.getByRole('button', { name: /setup bodhi server/i }));

    // Wait for the API call and redirect
    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup/resource-admin');
    });
  });

  it('should show error toast when setup fails', async () => {
    server.use(
      rest.post(`*${ENDPOINT_APP_SETUP}`, (_, res, ctx) => {
        return res(ctx.status(400), ctx.json({ error: { message: 'Setup failed' } }));
      })
    );

    const user = userEvent.setup();

    await act(async () => {
      render(<Setup />, { wrapper: createWrapper() });
    });

    // Fill out the form with valid data (minimum 10 characters for name)
    await user.type(screen.getByLabelText(/server name/i), 'My Test Server Instance');

    // Submit the form
    await user.click(screen.getByRole('button', { name: /setup bodhi server/i }));

    // Wait for the error toast
    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith(showErrorParams('Error', 'Setup failed'));
    });
  });

  it('should show validation error for server name shorter than 10 characters', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<Setup />, { wrapper: createWrapper() });
    });

    // Fill out the form with invalid data (less than 10 characters)
    await user.type(screen.getByLabelText(/server name/i), 'Short');

    // Submit the form
    await user.click(screen.getByRole('button', { name: /setup bodhi server/i }));

    // Check for validation error
    await waitFor(() => {
      expect(screen.getByText('Server name must be at least 10 characters long')).toBeInTheDocument();
    });

    // Ensure no API call was made
    expect(pushMock).not.toHaveBeenCalled();
  });

  it('should show validation error for server name longer than 100 characters', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<Setup />, { wrapper: createWrapper() });
    });

    // Fill out the form with invalid data (more than 100 characters)
    const longName = 'a'.repeat(101);
    await user.type(screen.getByLabelText(/server name/i), longName);

    // Submit the form
    await user.click(screen.getByRole('button', { name: /setup bodhi server/i }));

    // Check for validation error
    await waitFor(() => {
      expect(screen.getByText('Server name must be less than 100 characters')).toBeInTheDocument();
    });

    // Ensure no API call was made
    expect(pushMock).not.toHaveBeenCalled();
  });

  it('should show validation error for description longer than 500 characters', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<Setup />, { wrapper: createWrapper() });
    });

    // Fill out the form with valid name but invalid description
    await user.type(screen.getByLabelText(/server name/i), 'My Test Server Instance');

    // Use paste instead of typing for performance
    const longDescription = 'a'.repeat(501);
    const descriptionField = screen.getByLabelText(/description/i);
    await user.click(descriptionField);
    await user.keyboard(`{Control>}a{/Control}`); // Select all
    await user.paste(longDescription); // Paste the long text

    // Submit the form
    await user.click(screen.getByRole('button', { name: /setup bodhi server/i }));

    // Check for validation error
    await waitFor(() => {
      expect(screen.getByText('Description must be less than 500 characters')).toBeInTheDocument();
    });

    // Ensure no API call was made
    expect(pushMock).not.toHaveBeenCalled();
  });

  it('should render page content', async () => {
    await act(async () => {
      render(<Setup />, { wrapper: createWrapper() });
    });

    expect(screen.getByText('Complete Privacy')).toBeInTheDocument();
    expect(screen.getByText('Always Free')).toBeInTheDocument();
    expect(screen.getByText('Full Control')).toBeInTheDocument();
    expect(screen.getByText('Local Performance')).toBeInTheDocument();
    expect(screen.getByText('AI for Everyone')).toBeInTheDocument();
    expect(screen.getByText('Solid Foundation')).toBeInTheDocument();
    expect(screen.getByText('Welcome to Bodhi App')).toBeInTheDocument();
    expect(screen.getByText('Run AI Models Locally, Privately, and Completely Free')).toBeInTheDocument();

    // Check setup progress
    expect(screen.getByText('Step 1 of 4')).toBeInTheDocument();
  });

  it('should disable form fields and button when loading', async () => {
    server.use(
      rest.post(`*${ENDPOINT_APP_SETUP}`, (_, res, ctx) => {
        return res(ctx.delay(1000), ctx.json({ status: 'ready' }));
      })
    );

    const user = userEvent.setup();
    await act(async () => {
      render(<Setup />, { wrapper: createWrapper() });
    });
    await user.type(screen.getByLabelText(/server name/i), 'My Test Server Instance');
    await user.click(screen.getByRole('button', { name: /setup bodhi server/i }));
    await waitFor(() => {
      expect(screen.getByLabelText(/server name/i)).toBeDisabled();
      expect(screen.getByLabelText(/description/i)).toBeDisabled();
      expect(screen.getByRole('button', { name: /setting up/i })).toBeDisabled();
    });
  });

  it('should submit form with only required fields', async () => {
    const user = userEvent.setup();
    await act(async () => {
      render(<Setup />, { wrapper: createWrapper() });
    });
    await user.type(screen.getByLabelText(/server name/i), 'My Test Server Instance');
    await user.click(screen.getByRole('button', { name: /setup bodhi server/i }));
    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup/download-models');
    });
  });
});
