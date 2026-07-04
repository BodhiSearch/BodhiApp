import EditApiModel from '@/routes/models/api/edit/index';
import { ShellHarness } from '@/test-utils/shell-harness';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import {
  expectApiFormatSelected,
  expectErrorToast,
  expectModelsLoaded,
  expectSuccessToast,
  fetchModels,
  removeSelectedModel,
  selectModels,
  submitForm,
} from '@/test-utils/api-model-test-utils';
import {
  mockApiFormatsDefault,
  mockFetchApiModelsSuccess,
  mockGetApiModel,
  mockTestApiModelSuccess,
  mockUpdateApiModel,
  mockUpdateApiModelError,
} from '@/test-utils/msw-v2/handlers/api-models';
import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';

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
    useSearch: () => ({ id: 'test-model' }),
  };
});

const mockToast = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast, dismiss: () => {} }),
}));

setupMswV2();

afterEach(() => {
  server.resetHandlers();
  vi.clearAllMocks();
});

describe('Edit API Model Page - Page-Level Integration Tests', () => {
  describe('Page Structure and Initial Render', () => {
    it('renders page successfully with form elements prefilled from API data', async () => {
      server.use(
        ...mockAppInfoReady(),
        ...mockUserLoggedIn({ role: 'resource_user' }),
        ...mockGetApiModel('test-model', {
          id: 'test-model',
          api_format: 'openai',
          base_url: 'https://api.openai.com/v1',
          has_api_key: true,
          models: [
            { id: 'gpt-3.5-turbo', object: 'model', created: 0, owned_by: 'openai', provider: 'openai', access: true },
          ],
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
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

      const apiFormatSelector = screen.getByTestId('api-format-selector');
      const baseUrlInput = screen.getByTestId('base-url-input');
      const apiKeyInput = screen.getByTestId('api-key-input');

      expect(apiFormatSelector).toBeInTheDocument();
      expectApiFormatSelected('openai');

      expect(baseUrlInput).toHaveValue('https://api.openai.com/v1');

      expect(apiKeyInput).toHaveValue(''); // empty for security in edit mode

      const submitButton = screen.getByTestId('update-api-model-button');
      expect(submitButton).toHaveTextContent(/update/i);

      const selectedModelBadge = screen.getByTestId('selected-model-gpt-3.5-turbo');
      expect(selectedModelBadge).toBeInTheDocument();

      expect(screen.getByText('Edit API Model')).toBeInTheDocument();
    });
  });

  describe('Form Update Flow - Success Cases', () => {
    beforeEach(() => {
      server.use(
        ...mockAppInfoReady(),
        ...mockUserLoggedIn({ role: 'resource_user' }),
        ...mockGetApiModel(
          'test-model',
          {
            id: 'test-model',
            api_format: 'openai',
            base_url: 'https://api.openai.com/v1',
            has_api_key: true,
            models: [
              {
                id: 'gpt-3.5-turbo',
                object: 'model',
                created: 0,
                owned_by: 'openai',
                provider: 'openai',
                access: true,
              },
            ],
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          },
          { stub: true }
        ),
        ...mockUpdateApiModel(
          'test-model',
          {
            id: 'test-model',
            api_format: 'openai',
            base_url: 'https://api.openai.com/v1',
            has_api_key: true,
            models: [
              { id: 'gpt-4', object: 'model', created: 0, owned_by: 'openai', provider: 'openai', access: true },
            ],
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          },
          { stub: true }
        ),
        ...mockApiFormatsDefault(),
        ...mockTestApiModelSuccess(),
        ...mockFetchApiModelsSuccess()
      );
    });

    it('successfully updates API model with different model selection', async () => {
      const user = userEvent.setup();

      render(<EditApiModel />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByTestId('edit-api-model-form')).toBeInTheDocument();
      });

      expect(screen.getByTestId('api-format-selector')).toBeInTheDocument();
      expectApiFormatSelected('openai');

      const initialSelectedModel = screen.getByTestId('selected-model-gpt-3.5-turbo');
      expect(initialSelectedModel).toBeInTheDocument();

      await fetchModels(user);

      // only unselected models show as available
      await waitFor(() => {
        expectModelsLoaded(['gpt-4', 'gpt-4-turbo-preview']);
      });

      await removeSelectedModel(user, 'gpt-3.5-turbo');

      await waitFor(() => {
        expect(screen.queryByTestId('selected-model-gpt-3.5-turbo')).not.toBeInTheDocument();
      });

      await selectModels(user, ['gpt-4']);

      await waitFor(() => {
        expect(screen.getByTestId('selected-model-gpt-4')).toBeInTheDocument();
      });

      await submitForm(user, 'update-api-model-button');

      await waitFor(() => {
        expect(mockToast).toHaveBeenLastCalledWith(
          expect.objectContaining({
            title: 'API Model Updated',
          })
        );
      });

      expect(navigateMock).toHaveBeenCalledWith({ to: '/models/' });
    });
  });

  describe('Form Update Flow - Error Cases', () => {
    beforeEach(() => {
      server.use(
        ...mockAppInfoReady(),
        ...mockUserLoggedIn({ role: 'resource_user' }),
        ...mockGetApiModel(
          'test-model',
          {
            id: 'test-model',
            api_format: 'openai',
            base_url: 'https://api.openai.com/v1',
            has_api_key: true,
            models: [
              {
                id: 'gpt-3.5-turbo',
                object: 'model',
                created: 0,
                owned_by: 'openai',
                provider: 'openai',
                access: true,
              },
            ],
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          },
          { stub: true }
        ),
        ...mockUpdateApiModelError('test-model', {}, { stub: true }),
        ...mockApiFormatsDefault(),
        ...mockTestApiModelSuccess(),
        ...mockFetchApiModelsSuccess()
      );
    });

    it('handles server error during API model update', async () => {
      const user = userEvent.setup();

      render(<EditApiModel />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByTestId('edit-api-model-form')).toBeInTheDocument();
      });

      const initialSelectedModel = screen.getByTestId('selected-model-gpt-3.5-turbo');
      expect(initialSelectedModel).toBeInTheDocument();

      await fetchModels(user);

      await waitFor(() => {
        expectModelsLoaded(['gpt-4', 'gpt-4-turbo-preview']);
      });

      await removeSelectedModel(user, 'gpt-3.5-turbo');
      await selectModels(user, ['gpt-4']);

      await waitFor(() => {
        expect(screen.getByTestId('selected-model-gpt-4')).toBeInTheDocument();
      });

      await submitForm(user, 'update-api-model-button');

      await waitFor(() => {
        expect(mockToast).toHaveBeenLastCalledWith(
          expect.objectContaining({
            title: 'Failed to Update API Model',
            variant: 'destructive',
          })
        );
      });

      expect(navigateMock).not.toHaveBeenCalled();

      expect(screen.getByTestId('edit-api-model-form')).toBeInTheDocument();
    });
  });
});

describe('Edit API Model Page - V2 shell chrome', () => {
  it('publishes the Edit breadcrumb, renders the container, and locks api_format', async () => {
    server.use(
      ...mockAppInfoReady(),
      ...mockUserLoggedIn({ role: 'resource_user' }),
      ...mockGetApiModel('test-model', {
        id: 'test-model',
        api_format: 'openai',
        base_url: 'https://api.openai.com/v1',
        has_api_key: true,
        models: [
          { id: 'gpt-3.5-turbo', object: 'model', created: 0, owned_by: 'openai', provider: 'openai', access: true },
        ],
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      }),
      ...mockApiFormatsDefault(),
      ...mockTestApiModelSuccess(),
      ...mockFetchApiModelsSuccess()
    );

    render(
      <ShellHarness>
        <EditApiModel />
      </ShellHarness>,
      { wrapper: createWrapper() }
    );

    await waitFor(() => {
      expect(screen.getByTestId('edit-api-model-form')).toBeInTheDocument();
    });
    expect(screen.getByTestId('harness-breadcrumb')).toHaveTextContent('Bodhi / Models / Edit API Model');
    expect(screen.getByTestId('edit-api-model-page')).toBeInTheDocument();
    // api_format is read-only on edit (server-enforced too); the selector is disabled + shows the hint.
    expect(screen.getByTestId('api-format-selector')).toBeDisabled();
    expect(screen.getByTestId('api-format-selector-locked-hint')).toBeInTheDocument();
  });
});
