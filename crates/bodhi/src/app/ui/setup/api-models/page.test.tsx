// Import the page component
import ApiModelsSetupPage from '@/app/ui/setup/api-models/page';

// Import testing utilities
import { createWrapper } from '@/tests/wrapper';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterAll, afterEach, beforeAll, describe, expect, it, vi } from 'vitest';

// Import MSW v2 setup and API model handlers
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import {
  mockApiFormatsDefault,
  mockTestApiModelSuccess,
  mockFetchApiModelsSuccess,
  mockCreateApiModelSuccess,
  mockCreateApiModel,
  mockCreateApiModelError,
} from '@/test-utils/msw-v2/handlers/api-models';
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';

// Import API model test utilities
import {
  fillApiKey,
  testConnection,
  fetchModels,
  selectModels,
  submitForm,
  expectConnectionSuccess,
  expectModelsLoaded,
  expectSuccessToast,
  expectErrorToast,
  selectApiFormat,
} from '@/test-utils/api-model-test-utils';

// Import SetupProvider for context
import { SetupProvider } from '@/app/ui/setup/components';

// Mock next/navigation router
const mockPush = vi.fn();
const mockReplace = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: mockPush,
    replace: mockReplace,
  }),
  useSearchParams: vi.fn(),
  usePathname: () => '/ui/setup/api-models',
}));

// Mock toast notifications
const mockToast = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast, dismiss: () => {} }),
}));

// Mock setup-specific components to keep tests focused
vi.mock('@/app/ui/setup/SetupProgress', () => ({
  SetupProgress: ({ currentStep, totalSteps, stepLabels }: any) => (
    <div data-testid="setup-progress">
      Step {currentStep} of {totalSteps} - {stepLabels?.[currentStep - 1]}
    </div>
  ),
}));

vi.mock('@/app/ui/setup/BodhiLogo', () => ({
  BodhiLogo: () => <div data-testid="bodhi-logo">Bodhi Logo</div>,
}));

// Setup MSW v2 server
setupMswV2();

// Helper to render with SetupProvider
const renderWithSetupProvider = (component: React.ReactElement) => {
  return render(<SetupProvider>{component}</SetupProvider>, { wrapper: createWrapper() });
};

afterEach(() => {
  vi.clearAllMocks();
});

describe('Setup API Models Page - Page-Level Integration Tests', () => {
  describe('Page Structure and Initial Render', () => {
    it('renders page with correct authentication and app status requirements', async () => {
      // Setup API handlers
      server.use(
        ...mockAppInfoReady(),
        ...mockUserLoggedIn({ role: 'resource_user' }),
        ...mockApiFormatsDefault(),
        ...mockTestApiModelSuccess(),
        ...mockFetchApiModelsSuccess(),
        ...mockCreateApiModelSuccess()
      );

      // Render the setup page
      renderWithSetupProvider(<ApiModelsSetupPage />);

      // Wait for and verify the page container is rendered
      await waitFor(() => {
        expect(screen.getByTestId('api-models-setup-page')).toBeInTheDocument();
      });

      // Verify it's wrapped in AppInitializer (by checking page rendered successfully)
      // The createWrapper() provides the necessary context for AppInitializer

      // Verify setup progress is rendered (mocked component)
      expect(screen.getByTestId('setup-progress')).toBeInTheDocument();
      expect(screen.getByTestId('setup-progress')).toHaveTextContent('Step 4 of 7 - API Models');

      // Verify logo is rendered (mocked component)
      expect(screen.getByTestId('bodhi-logo')).toBeInTheDocument();
    });

    it('displays complete setup page structure with form in setup mode', async () => {
      // Setup API handlers
      server.use(
        ...mockAppInfoReady(),
        ...mockUserLoggedIn({ role: 'resource_user' }),
        ...mockApiFormatsDefault(),
        ...mockTestApiModelSuccess(),
        ...mockFetchApiModelsSuccess(),
        ...mockCreateApiModelSuccess()
      );

      // Render the setup page
      renderWithSetupProvider(<ApiModelsSetupPage />);

      // Wait for the page to fully load
      await waitFor(() => {
        expect(screen.getByTestId('api-models-setup-page')).toBeInTheDocument();
      });

      // Verify core components are present
      expect(screen.getByTestId('setup-progress')).toBeInTheDocument();
      expect(screen.getByTestId('bodhi-logo')).toBeInTheDocument();

      // Verify form structure
      const form = screen.getByTestId('setup-api-model-form');
      expect(form).toBeInTheDocument();

      // Verify important buttons and their labels
      const submitButton = screen.getByTestId('create-api-model-button');
      expect(submitButton).toBeInTheDocument();
      expect(submitButton).toHaveTextContent('Create API Model');

      const skipButton = screen.getByTestId('skip-api-setup');
      expect(skipButton).toBeInTheDocument();
      expect(skipButton).toHaveTextContent('Continue');

      // No cancel button in setup mode
      expect(screen.queryByTestId('cancel-button')).not.toBeInTheDocument();

      // Verify initial field states
      const apiFormatSelect = screen.getByTestId('api-format-selector');
      expect(apiFormatSelect).toBeInTheDocument();
      expect(apiFormatSelect).toHaveTextContent('Select an API format'); // Empty in setup mode

      const baseUrlInput = screen.getByTestId('base-url-input') as HTMLInputElement;
      expect(baseUrlInput.value).toBe('');

      const apiKeyInput = screen.getByTestId('api-key-input') as HTMLInputElement;
      expect(apiKeyInput.value).toBe('');

      // Action buttons should be disabled initially
      expect(screen.getByTestId('test-connection-button')).toBeDisabled();
      expect(screen.getByTestId('fetch-models-button')).toBeDisabled();
    });
  });

  describe('Skip Functionality', () => {
    it('navigates to setup complete when skip button is clicked', async () => {
      // Setup API handlers
      server.use(
        ...mockAppInfoReady(),
        ...mockUserLoggedIn({ role: 'resource_user' }),
        ...mockApiFormatsDefault(),
        ...mockTestApiModelSuccess(),
        ...mockFetchApiModelsSuccess(),
        ...mockCreateApiModelSuccess()
      );

      // Create user event instance
      const user = userEvent.setup();

      // Render the setup page
      renderWithSetupProvider(<ApiModelsSetupPage />);

      // Wait for the page to fully load
      await waitFor(() => {
        expect(screen.getByTestId('api-models-setup-page')).toBeInTheDocument();
      });

      // Find and click skip button
      const skipButton = screen.getByTestId('skip-api-setup');
      await user.click(skipButton);

      // Verify navigation to setup browser extension
      expect(mockPush).toHaveBeenCalledWith('/ui/setup/toolsets');
      expect(mockPush).toHaveBeenCalledTimes(1);

      // Verify no form submission occurred (no toast notifications)
      expect(mockToast).not.toHaveBeenCalled();
    });
  });

  describe('Form Initial State Validation', () => {
    it('form shows correct initial field values and button states for setup mode', async () => {
      // Setup API handlers
      server.use(
        ...mockAppInfoReady(),
        ...mockUserLoggedIn({ role: 'resource_user' }),
        ...mockApiFormatsDefault(),
        ...mockTestApiModelSuccess(),
        ...mockFetchApiModelsSuccess(),
        ...mockCreateApiModelSuccess()
      );

      // Render the setup page
      renderWithSetupProvider(<ApiModelsSetupPage />);

      // Wait for the page to fully load
      await waitFor(() => {
        expect(screen.getByTestId('api-models-setup-page')).toBeInTheDocument();
      });

      // Get form elements
      const apiFormatSelector = screen.getByTestId('api-format-selector');
      const baseUrlInput = screen.getByTestId('base-url-input') as HTMLInputElement;
      const apiKeyInput = screen.getByTestId('api-key-input') as HTMLInputElement;

      // Verify setup mode specific initial values (empty, no pre-selection)
      expect(apiFormatSelector).toHaveTextContent('Select an API format'); // Empty selector
      expect(baseUrlInput.value).toBe(''); // Empty, not OpenAI URL
      expect(apiKeyInput.value).toBe(''); // Empty

      // Verify button states
      const testConnectionButton = screen.getByTestId('test-connection-button');
      const fetchModelsButton = screen.getByTestId('fetch-models-button');
      const submitButton = screen.getByTestId('create-api-model-button');

      expect(testConnectionButton).toBeDisabled(); // Disabled without valid data
      expect(fetchModelsButton).toBeDisabled(); // Disabled without valid data
      expect(submitButton).toHaveTextContent('Create API Model'); // Correct button text
      expect(submitButton).not.toBeDisabled(); // Form allows submission (validation on submit)

      // Verify API key field is password type (security)
      expect(apiKeyInput).toHaveAttribute('type', 'password');
    });
  });

  describe('Form Submission Success', () => {
    it('successfully creates API model and redirects to setup complete', async () => {
      // Setup with happy path handlers
      const user = userEvent.setup();
      server.use(
        ...mockAppInfoReady(),
        ...mockUserLoggedIn({ role: 'resource_user' }),
        ...mockApiFormatsDefault(),
        ...mockTestApiModelSuccess(),
        ...mockFetchApiModelsSuccess(),
        ...mockCreateApiModelSuccess()
      );

      // Render the setup page
      renderWithSetupProvider(<ApiModelsSetupPage />);

      // Wait for the page to fully load
      await waitFor(() => {
        expect(screen.getByTestId('api-models-setup-page')).toBeInTheDocument();
      });

      // Select OpenAI API format (this will auto-populate base URL)
      await selectApiFormat(user, 'openai');

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

      // Verify redirect to setup browser extension (NOT to complete or models page)
      expect(mockPush).toHaveBeenCalledWith('/ui/setup/toolsets');
      expect(mockPush).not.toHaveBeenCalledWith('/ui/setup/complete');
      expect(mockPush).not.toHaveBeenCalledWith('/ui/models');
    });
  });

  describe('Error Handling', () => {
    it('handles server error during API model creation and stays on setup page', async () => {
      // Setup with happy path for initial operations
      const user = userEvent.setup();
      server.use(
        ...mockAppInfoReady(),
        ...mockUserLoggedIn({ role: 'resource_user' }),
        ...mockApiFormatsDefault(),
        ...mockTestApiModelSuccess(),
        ...mockFetchApiModelsSuccess(),
        ...mockCreateApiModelSuccess()
      );

      // Render the setup page
      renderWithSetupProvider(<ApiModelsSetupPage />);

      // Wait for the page to fully load
      await waitFor(() => {
        expect(screen.getByTestId('api-models-setup-page')).toBeInTheDocument();
      });

      // Fill valid form data
      await selectApiFormat(user, 'openai');
      await fillApiKey(user, 'sk-test-key-123');

      // Test connection successfully
      await testConnection(user);
      await waitFor(() => expectConnectionSuccess());

      // Fetch models successfully
      await fetchModels(user);
      await waitFor(() => {
        expectModelsLoaded(['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo-preview']);
      });

      // Select a model (form is now valid)
      await selectModels(user, ['gpt-4']);

      // Override create handler to return 500 error
      server.use(
        ...mockCreateApiModelError({
          code: 'internal_server_error',
          message: 'Internal server error',
          type: 'internal_server_error',
        })
      );

      // Submit form (will fail)
      await submitForm(user);

      // Verify error toast
      await waitFor(() => {
        expectErrorToast(mockToast, 'Failed to Create API Model');
      });

      // Verify NO navigation occurred (stays on setup page)
      expect(mockPush).not.toHaveBeenCalled();

      // Verify form is still visible and functional
      expect(screen.getByTestId('setup-api-model-form')).toBeInTheDocument();
      expect(screen.getByTestId('skip-api-setup')).toBeInTheDocument(); // Skip still available
    });
  });
});
