import NewApiModel from '@/app/ui/api-models/new/page';
import EditApiModel from '@/app/ui/api-models/edit/page';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

// Test utilities and data
import { createApiModelHandlers } from '@/test-utils/msw-handlers';
import {
  selectProvider,
  selectApiFormat,
  fillApiKey,
  fillBaseUrl,
  testConnection,
  fetchModels,
  selectModels,
  submitForm,
  expectProviderSelected,
  expectApiFormatSelected,
  expectConnectionSuccess,
  expectModelsLoaded,
  expectModelSelected,
  expectBaseUrlVisible,
  expectBaseUrlHidden,
  expectApiKeyHidden,
  expectApiKeyVisible,
  toggleApiKeyVisibility,
  expectLoadingState,
  waitForNoLoadingState,
  completeFullWorkflow,
  expectSuccessToast,
  expectErrorToast,
  expectNavigationCall,
} from '@/test-utils/api-model-test-utils';
import {
  TEST_SCENARIOS,
  createTestHandlers,
  navigationPaths,
  loadingStates,
  toastMessages,
  mockApiModelResponses,
} from '@/test-utils/api-model-test-data';

// Mock router
const mockPush = vi.fn();
const mockReplace = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: mockPush,
    replace: mockReplace,
  }),
  useSearchParams: vi.fn(),
}));

// Mock toast
const mockToast = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast, dismiss: () => {} }),
}));

const server = setupServer();

beforeAll(() => {
  server.listen({ onUnhandledRequest: 'error' });
});

afterAll(() => {
  server.close();
});

afterEach(() => {
  server.resetHandlers();
  vi.clearAllMocks();
});

describe('New API Model Page - Page-Level Integration Tests', () => {
  describe('Page Structure and Initial Render', () => {
    it('renders page with correct authentication and app status requirements', async () => {
      server.use(...createApiModelHandlers());

      render(<NewApiModel />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByTestId('create-api-model-form')).toBeInTheDocument();
        expect(screen.getByText('Create New API Model')).toBeInTheDocument();
      });
    });

    it('displays API format selector initially', async () => {
      server.use(...createApiModelHandlers());

      render(<NewApiModel />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByTestId('api-format-selector')).toBeInTheDocument();
        expect(screen.getByTestId('api-key-input')).toBeInTheDocument();
        expect(screen.getByTestId('base-url-input')).toBeInTheDocument();
      });
    });
  });

  describe('Page State Verification', () => {
    it('New API Model page loads with correct initial state', async () => {
      server.use(...createApiModelHandlers());

      render(<NewApiModel />, { wrapper: createWrapper() });

      // Wait for form to load and verify basic structure
      await waitFor(() => {
        expect(screen.getByTestId('create-api-model-form')).toBeInTheDocument();
        expect(screen.getByText('Create New API Model')).toBeInTheDocument();
      });

      // Verify initial field states
      const apiFormatSelector = screen.getByTestId('api-format-selector');
      const baseUrlInput = screen.getByTestId('base-url-input');
      const apiKeyInput = screen.getByTestId('api-key-input');

      expect(apiFormatSelector).toBeInTheDocument();
      expect(baseUrlInput).toHaveValue('https://api.openai.com/v1'); // Default OpenAI URL
      expect(apiKeyInput).toHaveValue(''); // Empty initially

      // Verify button states - should be disabled initially
      const testConnectionButton = screen.getByTestId('test-connection-button');
      const fetchModelsButton = screen.getByTestId('fetch-models-button');
      expect(testConnectionButton).toBeDisabled();
      expect(fetchModelsButton).toBeDisabled();

      // Verify submit button shows create mode text and initial state
      const submitButton = screen.getByTestId('create-api-model-button');
      expect(submitButton).toHaveTextContent(/create/i);
      expect(submitButton).not.toBeDisabled(); // Form allows submission (validation happens on submit)
    });

    it('Form shows correct initial field values and states', async () => {
      server.use(...createApiModelHandlers());

      render(<NewApiModel />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByTestId('create-api-model-form')).toBeInTheDocument();
      });

      // Verify API key field is password type initially (hidden)
      expectApiKeyHidden();
    });

    it('Form validation prevents submission with empty required fields', async () => {
      const user = userEvent.setup();
      server.use(...createApiModelHandlers());

      render(<NewApiModel />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByTestId('create-api-model-form')).toBeInTheDocument();
      });

      // Try to submit without filling required fields
      await submitForm(user);

      // Should show validation errors (form doesn't submit successfully)
      await waitFor(() => {
        expect(mockToast).not.toHaveBeenCalledWith(
          expect.objectContaining({
            title: 'API Model Created',
          })
        );
      });
    });
  });

  describe('Form Submission and Navigation', () => {
    it('successfully creates API model and redirects to models page', async () => {
      const user = userEvent.setup();

      server.use(...createApiModelHandlers(createTestHandlers.openaiHappyPath()));

      render(<NewApiModel />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByTestId('create-api-model-form')).toBeInTheDocument();
      });

      // Fill API key
      await fillApiKey(user, 'sk-test-key-123');

      // Test connection
      await testConnection(user);
      await waitFor(() => expectConnectionSuccess());

      // Fetch available models
      await fetchModels(user);
      await waitFor(() => {
        expectModelsLoaded(['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo-preview']);
      });

      // Select gpt-4 model
      await selectModels(user, ['gpt-4']);

      // Submit the form
      await submitForm(user);

      // Verify success toast
      await waitFor(() => {
        expectSuccessToast(mockToast, 'API Model Created');
      });

      // Verify redirect to models page
      expect(mockPush).toHaveBeenCalledWith('/ui/models');
    });

    it('handles server error during API model creation', async () => {
      const user = userEvent.setup();

      // Use normal handlers for initial operations
      server.use(...createApiModelHandlers(createTestHandlers.openaiHappyPath()));

      render(<NewApiModel />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByTestId('create-api-model-form')).toBeInTheDocument();
      });

      // Fill API key
      await fillApiKey(user, 'sk-test-key-123');

      // Test connection
      await testConnection(user);
      await waitFor(() => expectConnectionSuccess());

      // Fetch models
      await fetchModels(user);
      await waitFor(() => {
        expectModelsLoaded(['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo-preview']);
      });

      // Select a model (form is now valid)
      await selectModels(user, ['gpt-4']);

      // Override the create handler to return server error
      server.use(
        rest.post('*/bodhi/v1/api-models', (_, res, ctx) => {
          return res(
            ctx.status(500),
            ctx.json({
              error: {
                message: 'Internal server error',
                type: 'internal_server_error',
              },
            })
          );
        })
      );

      // Submit the form (should fail with 500)
      await submitForm(user);

      // Verify error toast is shown
      await waitFor(() => {
        expectErrorToast(mockToast, 'Failed to Create API Model');
      });

      // Verify NO navigation occurred (stays on same page)
      expect(mockPush).not.toHaveBeenCalled();

      // Form should still be visible
      expect(screen.getByTestId('create-api-model-form')).toBeInTheDocument();
    });
  });
});
