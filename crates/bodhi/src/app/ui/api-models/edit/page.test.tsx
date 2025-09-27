import EditApiModel from '@/app/ui/api-models/edit/page';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

// Test utilities and data
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import {
  mockGetApiModel,
  mockUpdateApiModel,
  mockApiFormatsDefault,
  mockTestApiModelSuccess,
  mockFetchApiModelsSuccess,
} from '@/test-utils/msw-v2/handlers/api-models';
import {
  selectProvider,
  selectApiFormat,
  fillApiKey,
  fillBaseUrl,
  testConnection,
  fetchModels,
  selectModels,
  removeSelectedModel,
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

// Mock router
const mockPush = vi.fn();
const mockReplace = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: mockPush,
    replace: mockReplace,
  }),
  useSearchParams: () => new URLSearchParams('id=test-model'),
}));

// Mock toast
const mockToast = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast, dismiss: () => {} }),
}));

// Use MSW v2 setup
setupMswV2();

afterEach(() => {
  server.resetHandlers();
  vi.clearAllMocks();
});

describe('Edit API Model Page - Page-Level Integration Tests', () => {
  describe('Page Structure and Initial Render', () => {
    it('renders page with correct authentication and app status requirements', async () => {
      server.use(
        ...mockAppInfoReady(),
        ...mockUserLoggedIn({ role: 'resource_user' }),
        ...mockGetApiModel({
          id: 'test-model',
          response: {
            id: 'test-model',
            api_format: 'openai',
            base_url: 'https://api.openai.com/v1',
            api_key_masked: '****123',
            models: ['gpt-3.5-turbo'],
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          },
        }),
        ...mockApiFormatsDefault(),
        ...mockTestApiModelSuccess(),
        ...mockFetchApiModelsSuccess()
      );

      render(<EditApiModel />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByTestId('edit-api-model-form')).toBeInTheDocument();
        expect(screen.getByText('Edit API Model')).toBeInTheDocument();
      });
    });

    it('loads successfully with form elements prefilled with data retrieved from API', async () => {
      server.use(
        ...mockAppInfoReady(),
        ...mockUserLoggedIn({ role: 'resource_user' }),
        ...mockGetApiModel({
          id: 'test-model',
          response: {
            id: 'test-model',
            api_format: 'openai',
            base_url: 'https://api.openai.com/v1',
            api_key_masked: '****123',
            models: ['gpt-3.5-turbo'],
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          },
        }),
        ...mockApiFormatsDefault(),
        ...mockTestApiModelSuccess(),
        ...mockFetchApiModelsSuccess()
      );

      render(<EditApiModel />, { wrapper: createWrapper() });

      // Wait for form to load and verify basic structure
      await waitFor(() => {
        expect(screen.getByTestId('edit-api-model-form')).toBeInTheDocument();
        expect(screen.getByText('Edit API Model')).toBeInTheDocument();
      });

      // Verify form fields are prefilled with existing data
      const apiFormatSelector = screen.getByTestId('api-format-selector');
      const baseUrlInput = screen.getByTestId('base-url-input');
      const apiKeyInput = screen.getByTestId('api-key-input');

      expect(apiFormatSelector).toBeInTheDocument();
      expectApiFormatSelected('openai');

      // Verify base URL is prefilled with existing value
      expect(baseUrlInput).toHaveValue('https://api.openai.com/v1');

      // Verify API key shows masked value (for security)
      expect(apiKeyInput).toHaveValue(''); // Should be empty for security in edit mode

      // Verify button shows update mode text
      const submitButton = screen.getByTestId('update-api-model-button');
      expect(submitButton).toHaveTextContent(/update/i);

      // Verify previously selected models are shown as selected
      const selectedModelBadge = screen.getByTestId('selected-model-gpt-3.5-turbo');
      expect(selectedModelBadge).toBeInTheDocument();

      // Verify form is in edit mode
      expect(screen.getByText('Edit API Model')).toBeInTheDocument();
    });
  });

  describe('Form Update Flow - Success Cases', () => {
    beforeEach(() => {
      // Set up success handlers for all tests in this block
      server.use(
        ...mockAppInfoReady(),
        ...mockUserLoggedIn({ role: 'resource_user' }),
        ...mockGetApiModel({
          id: 'test-model',
          response: {
            id: 'test-model',
            api_format: 'openai',
            base_url: 'https://api.openai.com/v1',
            api_key_masked: '****123',
            models: ['gpt-3.5-turbo'],
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          },
        }),
        ...mockUpdateApiModel({
          id: 'test-model',
          response: {
            id: 'test-model',
            api_format: 'openai',
            base_url: 'https://api.openai.com/v1',
            api_key_masked: '****456',
            models: ['gpt-4'],
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          },
        }),
        ...mockApiFormatsDefault(),
        ...mockTestApiModelSuccess(),
        ...mockFetchApiModelsSuccess()
      );
    });

    it('successfully updates API model with different model selection', async () => {
      const user = userEvent.setup();

      render(<EditApiModel />, { wrapper: createWrapper() });

      // Wait for form to load with existing data
      await waitFor(() => {
        expect(screen.getByTestId('edit-api-model-form')).toBeInTheDocument();
      });

      // Verify initial state - form should be prefilled
      expect(screen.getByTestId('api-format-selector')).toBeInTheDocument();
      expectApiFormatSelected('openai');

      // Verify existing model is selected
      const initialSelectedModel = screen.getByTestId('selected-model-gpt-3.5-turbo');
      expect(initialSelectedModel).toBeInTheDocument();

      // Fetch available models (using stored credentials, no API key needed)
      await fetchModels(user);

      // Wait for models to be loaded - only unselected models show as available
      await waitFor(() => {
        expectModelsLoaded(['gpt-4', 'gpt-4-turbo-preview']);
      });

      // Remove the currently selected model
      await removeSelectedModel(user, 'gpt-3.5-turbo');

      // Verify the model was removed from selection
      await waitFor(() => {
        expect(screen.queryByTestId('selected-model-gpt-3.5-turbo')).not.toBeInTheDocument();
      });

      // Select a new model
      await selectModels(user, ['gpt-4']);

      // Verify new model is selected
      await waitFor(() => {
        expect(screen.getByTestId('selected-model-gpt-4')).toBeInTheDocument();
      });

      // Submit the form to update
      await submitForm(user, 'update-api-model-button');

      // Verify success toast
      await waitFor(() => {
        expectSuccessToast(mockToast, 'API Model Updated');
      });

      // Verify redirect to models page
      expect(mockPush).toHaveBeenCalledWith('/ui/models');
    });
  });

  describe('Form Update Flow - Error Cases', () => {
    beforeEach(() => {
      // Set up handlers with error response for PUT
      server.use(
        ...mockAppInfoReady(),
        ...mockUserLoggedIn({ role: 'resource_user' }),
        ...mockGetApiModel({
          id: 'test-model',
          response: {
            id: 'test-model',
            api_format: 'openai',
            base_url: 'https://api.openai.com/v1',
            api_key_masked: '****123',
            models: ['gpt-3.5-turbo'],
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          },
        }),
        ...mockUpdateApiModel({
          id: 'test-model',
          error: {
            status: 500,
            code: 'internal_server_error',
            message: 'Internal server error',
          },
        }),
        ...mockApiFormatsDefault(),
        ...mockTestApiModelSuccess(),
        ...mockFetchApiModelsSuccess()
      );
    });

    it('handles server error during API model update', async () => {
      const user = userEvent.setup();

      render(<EditApiModel />, { wrapper: createWrapper() });

      // Wait for form to load with existing data
      await waitFor(() => {
        expect(screen.getByTestId('edit-api-model-form')).toBeInTheDocument();
      });

      // Verify existing model is selected
      const initialSelectedModel = screen.getByTestId('selected-model-gpt-3.5-turbo');
      expect(initialSelectedModel).toBeInTheDocument();

      // Fetch available models (using stored credentials)
      await fetchModels(user);

      // Wait for models to be loaded - only unselected models show as available
      await waitFor(() => {
        expectModelsLoaded(['gpt-4', 'gpt-4-turbo-preview']);
      });

      // Remove the currently selected model and select a new one
      await removeSelectedModel(user, 'gpt-3.5-turbo');
      await selectModels(user, ['gpt-4']);

      // Verify form is ready for submission
      await waitFor(() => {
        expect(screen.getByTestId('selected-model-gpt-4')).toBeInTheDocument();
      });

      // Submit the form (should fail with 500)
      await submitForm(user, 'update-api-model-button');

      // Verify error toast is shown
      await waitFor(() => {
        expectErrorToast(mockToast, 'Failed to Update API Model');
      });

      // Verify NO navigation occurred (stays on same page)
      expect(mockPush).not.toHaveBeenCalled();

      // Form should still be visible after error
      expect(screen.getByTestId('edit-api-model-form')).toBeInTheDocument();
    });
  });
});
