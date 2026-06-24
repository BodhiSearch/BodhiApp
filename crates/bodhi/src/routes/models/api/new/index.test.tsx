import NewApiModel from '@/routes/models/api/new/index';
import EditApiModel from '@/routes/models/api/edit/index';
import { ShellSlotsProvider, useShellSlots } from '@/components/shell';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { server } from '@/test-utils/msw-v2/setup';
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

import {
  mockApiFormatsDefault,
  mockTestApiModelSuccess,
  mockFetchApiModelsSuccess,
  mockCreateApiModelSuccess,
  mockCreateApiModelError,
} from '@/test-utils/msw-v2/handlers/api-models';
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import {
  selectProvider,
  selectApiFormat,
  fillApiKey,
  fillName,
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
  };
});

const mockToast = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast, dismiss: () => {} }),
}));

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
    it('renders page with authentication and all form elements', async () => {
      server.use(
        ...mockAppInfoReady(),
        ...mockUserLoggedIn({ role: 'resource_user' }),
        ...mockApiFormatsDefault(),
        ...mockTestApiModelSuccess(),
        ...mockFetchApiModelsSuccess(),
        ...mockCreateApiModelSuccess()
      );

      render(<NewApiModel />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByTestId('create-api-model-form')).toBeInTheDocument();
        expect(screen.getByText('Create New API Model')).toBeInTheDocument();
        expect(screen.getByTestId('api-format-selector')).toBeInTheDocument();
        expect(screen.getByTestId('api-key-input')).toBeInTheDocument();
        expect(screen.getByTestId('base-url-input')).toBeInTheDocument();
      });
    });
  });

  describe('Page State Verification', () => {
    it('New API Model page loads with correct initial state', async () => {
      server.use(
        ...mockAppInfoReady(),
        ...mockUserLoggedIn({ role: 'resource_user' }),
        ...mockApiFormatsDefault(),
        ...mockTestApiModelSuccess(),
        ...mockFetchApiModelsSuccess(),
        ...mockCreateApiModelSuccess()
      );

      render(<NewApiModel />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByTestId('create-api-model-form')).toBeInTheDocument();
        expect(screen.getByText('Create New API Model')).toBeInTheDocument();
      });

      const apiFormatSelector = screen.getByTestId('api-format-selector');
      const baseUrlInput = screen.getByTestId('base-url-input');
      const apiKeyInput = screen.getByTestId('api-key-input');

      expect(apiFormatSelector).toBeInTheDocument();
      expect(baseUrlInput).toHaveValue('https://api.openai.com/v1');
      expect(apiKeyInput).toHaveValue('');

      expectApiKeyHidden();

      const testConnectionButton = screen.getByTestId('test-connection-button');
      const fetchModelsButton = screen.getByTestId('fetch-models-button');
      expect(testConnectionButton).not.toBeDisabled();
      expect(fetchModelsButton).not.toBeDisabled();

      const submitButton = screen.getByTestId('create-api-model-button');
      expect(submitButton).toHaveTextContent(/create/i);
      expect(submitButton).not.toBeDisabled(); // validation happens on submit, not field-level
    });

    it('can select openai_responses format and updates form correctly', async () => {
      const user = userEvent.setup();
      server.use(
        ...mockAppInfoReady(),
        ...mockUserLoggedIn({ role: 'resource_user' }),
        ...mockApiFormatsDefault(),
        ...mockTestApiModelSuccess(),
        ...mockFetchApiModelsSuccess(),
        ...mockCreateApiModelSuccess()
      );

      render(<NewApiModel />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByTestId('create-api-model-form')).toBeInTheDocument();
      });

      expectApiFormatSelected('openai');

      await selectApiFormat(user, 'openai_responses');
      expectApiFormatSelected('openai_responses');

      // both presets use the same base URL
      const baseUrlInput = screen.getByTestId('base-url-input');
      expect(baseUrlInput).toHaveValue('https://api.openai.com/v1');
    });

    it('Form validation prevents submission with empty required fields', async () => {
      const user = userEvent.setup();
      server.use(
        ...mockAppInfoReady(),
        ...mockUserLoggedIn({ role: 'resource_user' }),
        ...mockApiFormatsDefault(),
        ...mockTestApiModelSuccess(),
        ...mockFetchApiModelsSuccess(),
        ...mockCreateApiModelSuccess()
      );

      render(<NewApiModel />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByTestId('create-api-model-form')).toBeInTheDocument();
      });

      await submitForm(user);

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

      server.use(
        ...mockAppInfoReady(),
        ...mockUserLoggedIn({ role: 'resource_user' }),
        ...mockApiFormatsDefault(),
        ...mockTestApiModelSuccess(),
        ...mockFetchApiModelsSuccess(),
        ...mockCreateApiModelSuccess()
      );

      render(<NewApiModel />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByTestId('create-api-model-form')).toBeInTheDocument();
      });

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

      expect(navigateMock).toHaveBeenCalledWith({ to: '/models/' });
    });

    it('handles server error during API model creation', async () => {
      const user = userEvent.setup();

      server.use(
        ...mockAppInfoReady(),
        ...mockUserLoggedIn({ role: 'resource_user' }),
        ...mockApiFormatsDefault(),
        ...mockTestApiModelSuccess(),
        ...mockFetchApiModelsSuccess(),
        ...mockCreateApiModelSuccess()
      );

      render(<NewApiModel />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByTestId('create-api-model-form')).toBeInTheDocument();
      });

      await fillName(user, 'Test API Model');
      await fillApiKey(user, 'sk-test-key-123');

      await testConnection(user);
      await waitFor(() => expectConnectionSuccess());

      await fetchModels(user);
      await waitFor(() => {
        expectModelsLoaded(['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo-preview']);
      });

      await selectModels(user, ['gpt-4']);

      server.use(...mockCreateApiModelError());

      await submitForm(user);

      await waitFor(() => {
        expectErrorToast(mockToast, 'Failed to Create API Model');
      });

      expect(navigateMock).not.toHaveBeenCalled();

      expect(screen.getByTestId('create-api-model-form')).toBeInTheDocument();
    });
  });
});

// V2 shell chrome: publishes the Models breadcrumb + renders a centered container (always-on — the
// API-model form shipped V2-only, no flag). The form itself is unchanged (same testids); this covers
// the additive chrome via the canonical ShellSlotsProvider harness (mirrors routes/models/index.v2.test).
function BreadcrumbConsumer() {
  const { breadcrumb } = useShellSlots();
  const crumbs = Array.isArray(breadcrumb) ? breadcrumb.map((b) => b.label).join(' / ') : '';
  return <div data-testid="harness-breadcrumb">{crumbs}</div>;
}

describe('New API Model Page - V2 shell chrome', () => {
  it('publishes the Models breadcrumb and renders the centered container', async () => {
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_user' }),
      ...mockApiFormatsDefault(),
      ...mockTestApiModelSuccess(),
      ...mockFetchApiModelsSuccess(),
      ...mockCreateApiModelSuccess()
    );

    render(
      <ShellSlotsProvider>
        <BreadcrumbConsumer />
        <NewApiModel />
      </ShellSlotsProvider>,
      { wrapper: createWrapper() }
    );

    await waitFor(() => {
      expect(screen.getByTestId('create-api-model-form')).toBeInTheDocument();
    });
    expect(screen.getByTestId('harness-breadcrumb')).toHaveTextContent('Bodhi / Models / New API Model');
    expect(screen.getByTestId('new-api-model-page')).toBeInTheDocument();
  });
});
