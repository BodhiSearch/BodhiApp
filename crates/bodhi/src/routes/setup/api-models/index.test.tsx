import ApiModelsSetupPage from '@/routes/setup/api-models/index';

import { createWrapper } from '@/tests/wrapper';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterAll, afterEach, beforeAll, describe, expect, it, vi } from 'vitest';

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

import {
  fillApiKey,
  fillName,
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

import { SetupProvider } from '@/routes/setup/-components';

const navigateMock = vi.fn();
vi.mock('@tanstack/react-router', async () => {
  const actual = await vi.importActual('@tanstack/react-router');
  return {
    ...actual,
    Link: ({ to, children, ...rest }: any) => (
      <a href={to} {...rest}>
        {children}
      </a>
    ),
    useNavigate: () => navigateMock,
    useSearch: () => ({}),
    useLocation: () => ({ pathname: '/setup/api-models' }),
  };
});

const mockToast = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast, dismiss: () => {} }),
}));

vi.mock('@/routes/setup/-components/SetupProgress', () => ({
  SetupProgress: ({ currentStep, totalSteps, stepLabels }: any) => (
    <div data-testid="setup-progress">
      Step {currentStep} of {totalSteps} - {stepLabels?.[currentStep - 1]}
    </div>
  ),
}));

vi.mock('@/routes/setup/-components/BodhiLogo', () => ({
  BodhiLogo: () => <div data-testid="bodhi-logo">Bodhi Logo</div>,
}));

setupMswV2();

const renderWithSetupProvider = (component: React.ReactElement) => {
  return render(<SetupProvider>{component}</SetupProvider>, { wrapper: createWrapper() });
};

afterEach(() => {
  vi.clearAllMocks();
});

describe('Setup API Models Page - Page-Level Integration Tests', () => {
  describe('Page Structure and Initial Render', () => {
    it('renders page with correct authentication and app status requirements', async () => {
      server.use(
        ...mockAppInfoReady(),
        ...mockUserLoggedIn({ role: 'resource_user' }),
        ...mockApiFormatsDefault(),
        ...mockTestApiModelSuccess(),
        ...mockFetchApiModelsSuccess(),
        ...mockCreateApiModelSuccess()
      );

      renderWithSetupProvider(<ApiModelsSetupPage />);

      await waitFor(() => {
        expect(screen.getByTestId('api-models-setup-page')).toBeInTheDocument();
      });

      expect(screen.getByTestId('setup-progress')).toBeInTheDocument();
      expect(screen.getByTestId('setup-progress')).toHaveTextContent('Step 4 of 6 - API Models');

      expect(screen.getByTestId('bodhi-logo')).toBeInTheDocument();
    });

    it('displays complete setup page structure with form in setup mode', async () => {
      server.use(
        ...mockAppInfoReady(),
        ...mockUserLoggedIn({ role: 'resource_user' }),
        ...mockApiFormatsDefault(),
        ...mockTestApiModelSuccess(),
        ...mockFetchApiModelsSuccess(),
        ...mockCreateApiModelSuccess()
      );

      renderWithSetupProvider(<ApiModelsSetupPage />);

      await waitFor(() => {
        expect(screen.getByTestId('api-models-setup-page')).toBeInTheDocument();
      });

      expect(screen.getByTestId('setup-progress')).toBeInTheDocument();
      expect(screen.getByTestId('bodhi-logo')).toBeInTheDocument();

      const form = screen.getByTestId('setup-api-model-form');
      expect(form).toBeInTheDocument();

      const submitButton = screen.getByTestId('create-api-model-button');
      expect(submitButton).toBeInTheDocument();
      expect(submitButton).toHaveTextContent('Create API Model');

      const skipButton = screen.getByTestId('skip-api-setup');
      expect(skipButton).toBeInTheDocument();
      expect(skipButton).toHaveTextContent('Continue');

      // No cancel button in setup mode
      expect(screen.queryByTestId('cancel-button')).not.toBeInTheDocument();

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
      server.use(
        ...mockAppInfoReady(),
        ...mockUserLoggedIn({ role: 'resource_user' }),
        ...mockApiFormatsDefault(),
        ...mockTestApiModelSuccess(),
        ...mockFetchApiModelsSuccess(),
        ...mockCreateApiModelSuccess()
      );

      const user = userEvent.setup();

      renderWithSetupProvider(<ApiModelsSetupPage />);

      await waitFor(() => {
        expect(screen.getByTestId('api-models-setup-page')).toBeInTheDocument();
      });

      const skipButton = screen.getByTestId('skip-api-setup');
      await user.click(skipButton);

      expect(navigateMock).toHaveBeenCalledWith({ to: '/setup/browser-extension/' });
      expect(navigateMock).toHaveBeenCalledTimes(1);

      // No form submission means no toast.
      expect(mockToast).not.toHaveBeenCalled();
    });
  });

  describe('Form Initial State Validation', () => {
    it('form shows correct initial field values and button states for setup mode', async () => {
      server.use(
        ...mockAppInfoReady(),
        ...mockUserLoggedIn({ role: 'resource_user' }),
        ...mockApiFormatsDefault(),
        ...mockTestApiModelSuccess(),
        ...mockFetchApiModelsSuccess(),
        ...mockCreateApiModelSuccess()
      );

      renderWithSetupProvider(<ApiModelsSetupPage />);

      await waitFor(() => {
        expect(screen.getByTestId('api-models-setup-page')).toBeInTheDocument();
      });

      const apiFormatSelector = screen.getByTestId('api-format-selector');
      const baseUrlInput = screen.getByTestId('base-url-input') as HTMLInputElement;
      const apiKeyInput = screen.getByTestId('api-key-input') as HTMLInputElement;

      // Setup mode starts empty, with no API format pre-selected.
      expect(apiFormatSelector).toHaveTextContent('Select an API format');
      expect(baseUrlInput.value).toBe('');
      expect(apiKeyInput.value).toBe('');

      const testConnectionButton = screen.getByTestId('test-connection-button');
      const fetchModelsButton = screen.getByTestId('fetch-models-button');
      const submitButton = screen.getByTestId('create-api-model-button');

      expect(testConnectionButton).toBeDisabled();
      expect(fetchModelsButton).toBeDisabled();
      expect(submitButton).toHaveTextContent('Create API Model');
      expect(submitButton).not.toBeDisabled(); // Form allows submission (validation on submit)

      // API key field must be password type so the secret is masked.
      expect(apiKeyInput).toHaveAttribute('type', 'password');
    });
  });

  describe('Form Submission Success', () => {
    it('successfully creates API model and redirects to setup complete', async () => {
      const user = userEvent.setup();
      server.use(
        ...mockAppInfoReady(),
        ...mockUserLoggedIn({ role: 'resource_user' }),
        ...mockApiFormatsDefault(),
        ...mockTestApiModelSuccess(),
        ...mockFetchApiModelsSuccess(),
        ...mockCreateApiModelSuccess()
      );

      renderWithSetupProvider(<ApiModelsSetupPage />);

      await waitFor(() => {
        expect(screen.getByTestId('api-models-setup-page')).toBeInTheDocument();
      });

      // Selecting OpenAI auto-populates the base URL.
      await selectApiFormat(user, 'openai');

      await fillName(user, 'Test API Model');
      await fillApiKey(user, 'sk-test-key-123');

      await testConnection(user);
      await waitFor(() => expectConnectionSuccess());

      await fetchModels(user);
      await waitFor(() => {
        expectModelsLoaded(['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo-preview']);
      });

      await selectModels(user, ['gpt-4']);

      await submitForm(user);

      await waitFor(() => {
        expectSuccessToast(mockToast, 'API Model Created');
      });

      // Redirect goes to browser-extension, NOT to complete or models page.
      expect(navigateMock).toHaveBeenCalledWith({ to: '/setup/browser-extension/' });
      expect(navigateMock).not.toHaveBeenCalledWith({ to: '/setup/complete/' });
      expect(navigateMock).not.toHaveBeenCalledWith({ to: '/models/' });
    });
  });

  describe('Error Handling', () => {
    it('handles server error during API model creation and stays on setup page', async () => {
      const user = userEvent.setup();
      server.use(
        ...mockAppInfoReady(),
        ...mockUserLoggedIn({ role: 'resource_user' }),
        ...mockApiFormatsDefault(),
        ...mockTestApiModelSuccess(),
        ...mockFetchApiModelsSuccess(),
        ...mockCreateApiModelSuccess()
      );

      renderWithSetupProvider(<ApiModelsSetupPage />);

      await waitFor(() => {
        expect(screen.getByTestId('api-models-setup-page')).toBeInTheDocument();
      });

      await selectApiFormat(user, 'openai');
      await fillName(user, 'Test API Model');
      await fillApiKey(user, 'sk-test-key-123');

      await testConnection(user);
      await waitFor(() => expectConnectionSuccess());

      await fetchModels(user);
      await waitFor(() => {
        expectModelsLoaded(['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo-preview']);
      });

      await selectModels(user, ['gpt-4']);

      // Swap in a 500 only for the create call, after the happy-path setup above.
      server.use(
        ...mockCreateApiModelError({
          code: 'internal_server_error',
          message: 'Internal server error',
          type: 'internal_server_error',
        })
      );

      await submitForm(user);

      await waitFor(() => {
        expectErrorToast(mockToast, 'Failed to Create API Model');
      });

      // No navigation on failure: the user stays on the setup page.
      expect(navigateMock).not.toHaveBeenCalled();

      expect(screen.getByTestId('setup-api-model-form')).toBeInTheDocument();
      expect(screen.getByTestId('skip-api-setup')).toBeInTheDocument();
    });
  });
});
