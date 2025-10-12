import ApiModelForm from '@/components/api-models/ApiModelForm';
import {
  mockApiFormats,
  mockCreateApiModel,
  mockCreateApiModelError,
  mockFetchApiModels,
  mockFetchApiModelsError,
  mockTestApiModel,
  mockTestApiModelError,
  mockUpdateApiModel,
} from '@/test-utils/msw-v2/handlers/api-models';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2, typedHttp } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { ApiModelResponse } from '@bodhiapp/ts-client';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

// API endpoint constants for MSW handlers
const ENDPOINT_API_MODEL_ID = '/bodhi/v1/api-models/{id}';

// Mock useRouter
const pushMock = vi.fn();
const replaceMock = vi.fn();
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
    replace: replaceMock,
  }),
}));

// Mock toast
const mockToast = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast, dismiss: () => {} }),
}));

// Mock component dependencies
vi.mock('@/components/ui/toaster', () => ({
  Toaster: () => null,
}));

// Mock required HTMLElement methods for Radix UI
Object.assign(window.HTMLElement.prototype, {
  scrollIntoView: vi.fn(),
  releasePointerCapture: vi.fn(),
  hasPointerCapture: vi.fn(),
  setPointerCapture: vi.fn(),
  getBoundingClientRect: vi.fn().mockReturnValue({
    x: 0,
    y: 0,
    width: 0,
    height: 0,
    top: 0,
    right: 0,
    bottom: 0,
    left: 0,
  }),
});

setupMswV2();

beforeAll(() => {
  Element.prototype.hasPointerCapture = vi.fn(() => false);
  Element.prototype.setPointerCapture = vi.fn();
  Element.prototype.releasePointerCapture = vi.fn();
});

afterEach(() => {
  vi.clearAllMocks();
});

beforeEach(() => {
  pushMock.mockClear();
  mockToast.mockClear();
});

const mockApiModelResponse: ApiModelResponse = {
  id: 'test-api-model',
  api_format: 'openai',
  base_url: 'https://api.openai.com/v1',
  api_key_masked: '***', // Has API key
  models: ['gpt-4', 'gpt-3.5-turbo'],
  prefix: null,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

// Helper functions for test setup
async function renderCreateFormWithApiKey(apiKey = 'sk-test-123') {
  const user = userEvent.setup();
  await act(async () => {
    render(<ApiModelForm mode="create" />, { wrapper: createWrapper() });
  });
  await user.click(screen.getByTestId('api-key-input-checkbox'));
  await user.type(screen.getByTestId('api-key-input'), apiKey);
  return user;
}

async function renderCreateFormWithoutApiKey() {
  const user = userEvent.setup();
  await act(async () => {
    render(<ApiModelForm mode="create" />, { wrapper: createWrapper() });
  });
  return user;
}

async function renderEditFormUsingStoredCreds() {
  const user = userEvent.setup();
  await act(async () => {
    render(<ApiModelForm mode="edit" initialData={mockApiModelResponse} />, {
      wrapper: createWrapper(),
    });
  });
  return user;
}

// Helper functions for model selection operations
async function fetchModelsAndWait(user: ReturnType<typeof userEvent.setup>) {
  await user.click(screen.getByTestId('fetch-models-button'));
  await waitFor(() => {
    expect(screen.queryByTestId('available-model-gpt-4')).toBeInTheDocument();
  });
}

async function selectModel(user: ReturnType<typeof userEvent.setup>, modelName: string) {
  const modelElement = screen.getByTestId(`available-model-${modelName}`);
  await user.click(modelElement);
}

describe('ApiModelForm', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }),
      ...mockUserLoggedIn(undefined),
      ...mockApiFormats({ data: ['openai'] }),
      ...mockCreateApiModel(mockApiModelResponse),
      ...mockUpdateApiModel('test-api-model', mockApiModelResponse),
      ...mockFetchApiModels({ models: ['gpt-4', 'gpt-3.5-turbo'] }),
      ...mockTestApiModel({ success: true, response: 'Test successful!' })
    );
  });

  describe('Create mode', () => {
    it('renders all form elements for creating new API model', async () => {
      await act(async () => {
        render(<ApiModelForm mode="create" />, { wrapper: createWrapper() });
      });

      expect(screen.getByText('Create New API Model')).toBeInTheDocument();

      expect(screen.getByTestId('api-format-selector')).toBeInTheDocument();
      expect(screen.getByTestId('base-url-input')).toBeInTheDocument();
      expect(screen.getByTestId('api-key-input')).toBeInTheDocument();
      expect(screen.getByTestId('api-key-input-checkbox')).toBeInTheDocument();

      expect(screen.getByTestId('create-api-model-button')).toBeInTheDocument();
      expect(screen.getByTestId('cancel-button')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /Fetch Models/i })).toBeInTheDocument();
      expect(screen.getByTestId('test-connection-button')).toBeInTheDocument();
    });

    it('shows validation errors for required fields', async () => {
      const user = userEvent.setup();

      await act(async () => {
        render(<ApiModelForm mode="create" />, { wrapper: createWrapper() });
      });

      await user.click(screen.getByTestId('create-api-model-button'));

      await waitFor(() => {
        const errorMessage = screen.queryByTestId('model-selection-section-error');
        expect(errorMessage).toBeInTheDocument();
        expect(errorMessage).toHaveTextContent(/at least one model must be selected/i);
      });
    });

    it('handles fetch models functionality', async () => {
      const user = userEvent.setup();
      server.use(
        ...mockFetchApiModels({
          models: ['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo'],
        })
      );

      await act(async () => {
        render(<ApiModelForm mode="create" />, { wrapper: createWrapper() });
      });

      await user.type(screen.getByTestId('base-url-input'), 'https://api.openai.com/v1');

      await user.click(screen.getByTestId('api-key-input-checkbox'));
      await user.type(screen.getByTestId('api-key-input'), 'sk-test123');

      await user.click(screen.getByTestId('fetch-models-button'));

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          title: 'Models Fetched Successfully',
          description: 'Found 3 available models',
        });
      });

      expect(screen.getByTestId('available-model-gpt-4')).toBeInTheDocument();
      expect(screen.getByTestId('available-model-gpt-3.5-turbo')).toBeInTheDocument();
      expect(screen.getByTestId('available-model-gpt-4-turbo')).toBeInTheDocument();
    });

    it('handles test connection functionality', async () => {
      const user = userEvent.setup();

      server.use(
        ...mockTestApiModel({
          success: true,
          response: 'Connection successful',
        })
      );

      await act(async () => {
        render(<ApiModelForm mode="create" />, { wrapper: createWrapper() });
      });

      await user.type(screen.getByTestId('base-url-input'), 'https://api.openai.com/v1');

      await user.click(screen.getByTestId('api-key-input-checkbox'));
      await user.type(screen.getByTestId('api-key-input'), 'sk-test123');

      await fetchModelsAndWait(user);
      await selectModel(user, 'gpt-4');

      const testButton = screen.getByTestId('test-connection-button');
      expect(testButton).not.toBeDisabled();

      await user.click(testButton);

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          title: 'Connection Test Successful',
          description: 'Connection successful',
        });
      });
    });

    it('creates API model successfully', async () => {
      const user = userEvent.setup();

      server.use(...mockCreateApiModel(mockApiModelResponse));

      await act(async () => {
        render(<ApiModelForm mode="create" />, { wrapper: createWrapper() });
      });

      await user.click(screen.getByTestId('api-key-input-checkbox'));
      await user.type(screen.getByTestId('api-key-input'), 'sk-test123');

      await fetchModelsAndWait(user);
      await selectModel(user, 'gpt-4');

      await user.click(screen.getByTestId('create-api-model-button'));

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith(
          expect.objectContaining({
            title: 'API Model Created',
          })
        );
      });

      expect(pushMock).toHaveBeenCalledWith('/ui/models');
    });

    it('enables fetch models button when base_url is present (regardless of API key)', async () => {
      await act(async () => {
        render(<ApiModelForm mode="create" />, { wrapper: createWrapper() });
      });

      const fetchButton = screen.getByTestId('fetch-models-button');
      // Fetch button should be enabled because base_url defaults to preset value
      expect(fetchButton).not.toBeDisabled();
    });

    it('validates empty API key when checkbox is checked and submitted', async () => {
      const user = userEvent.setup();

      await act(async () => {
        render(<ApiModelForm mode="create" />, { wrapper: createWrapper() });
      });

      await user.click(screen.getByTestId('api-key-input-checkbox'));
      await fetchModelsAndWait(user);
      await selectModel(user, 'gpt-4');
      await user.click(screen.getByTestId('create-api-model-button'));

      await waitFor(() => {
        const errorMessage = screen.queryByTestId('api-key-input-error');
        expect(errorMessage).toBeInTheDocument();
        expect(errorMessage).toHaveTextContent(/API key must not be empty/i);
      });
    });
  });

  describe('Edit mode', () => {
    describe('API key field initialization', () => {
      it.each([
        {
          description: 'when api_key_masked is "***" (has stored key)',
          apiKeyMasked: '***' as const,
          checkboxChecked: true,
          inputDisabled: false,
        },
        {
          description: 'when api_key_masked is null (no stored key)',
          apiKeyMasked: null,
          checkboxChecked: false,
          inputDisabled: true,
        },
        {
          description: 'when api_key_masked is undefined (no stored key)',
          apiKeyMasked: undefined,
          checkboxChecked: false,
          inputDisabled: true,
        },
      ])('renders API key field correctly $description', async ({ apiKeyMasked, checkboxChecked, inputDisabled }) => {
        const testData: ApiModelResponse = {
          ...mockApiModelResponse,
          api_key_masked: apiKeyMasked,
          prefix: null,
        };

        await act(async () => {
          render(<ApiModelForm mode="edit" initialData={testData} />, {
            wrapper: createWrapper(),
          });
        });

        expect(screen.getByText('Edit API Model')).toBeInTheDocument();
        expect(screen.getByTestId('base-url-input')).toHaveValue('https://api.openai.com/v1');
        expect(screen.getByTestId('update-api-model-button')).toBeInTheDocument();

        const apiKeyCheckbox = screen.getByTestId('api-key-input-checkbox');
        const apiKeyInput = screen.getByTestId('api-key-input');

        expect(apiKeyCheckbox).toHaveProperty('checked', checkboxChecked);
        expect(apiKeyInput).toHaveProperty('disabled', inputDisabled);
      });
    });

    describe('Prefix field initialization', () => {
      it.each([
        {
          description: 'when prefix is set to "azure/" (has prefix)',
          prefix: 'azure/',
          checkboxChecked: true,
          inputDisabled: false,
          inputValue: 'azure/',
        },
        {
          description: 'when prefix is null (no prefix)',
          prefix: null,
          checkboxChecked: false,
          inputDisabled: true,
          inputValue: '',
        },
        {
          description: 'when prefix is undefined (no prefix)',
          prefix: undefined,
          checkboxChecked: false,
          inputDisabled: true,
          inputValue: '',
        },
      ])(
        'renders prefix field correctly $description',
        async ({ prefix, checkboxChecked, inputDisabled, inputValue }) => {
          const testData: ApiModelResponse = {
            ...mockApiModelResponse,
            api_key_masked: '***',
            prefix,
          };

          await act(async () => {
            render(<ApiModelForm mode="edit" initialData={testData} />, {
              wrapper: createWrapper(),
            });
          });

          expect(screen.getByText('Edit API Model')).toBeInTheDocument();
          expect(screen.getByTestId('base-url-input')).toHaveValue('https://api.openai.com/v1');
          expect(screen.getByTestId('update-api-model-button')).toBeInTheDocument();

          const prefixCheckbox = screen.getByTestId('prefix-input-checkbox');
          const prefixInput = screen.getByTestId('prefix-input');

          expect(prefixCheckbox).toHaveProperty('checked', checkboxChecked);
          expect(prefixInput).toHaveProperty('disabled', inputDisabled);
          expect(prefixInput).toHaveValue(inputValue);
        }
      );
    });

    it('performs full workflow using stored credentials: fetch, select, test, and update', async () => {
      const user = userEvent.setup();

      server.use(
        ...mockFetchApiModels({
          models: ['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo', 'claude-3-sonnet'],
        }),
        ...mockTestApiModel({
          success: true,
          response: 'Test successful with stored credentials',
        }),
        ...mockUpdateApiModel('test-api-model', {
          ...mockApiModelResponse,
          models: ['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo'],
        })
      );

      await act(async () => {
        render(<ApiModelForm mode="edit" initialData={mockApiModelResponse} />, {
          wrapper: createWrapper(),
        });
      });

      // Verify API key checkbox is checked (has stored credentials)
      const apiKeyCheckbox = screen.getByTestId('api-key-input-checkbox');
      expect(apiKeyCheckbox).toBeChecked();

      // Verify fetch button is enabled with stored credentials
      const fetchButton = screen.getByTestId('fetch-models-button');
      expect(fetchButton).not.toBeDisabled();

      // Fetch models using stored credentials
      await user.click(fetchButton);

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          title: 'Models Fetched Successfully',
          description: 'Found 4 available models',
        });
      });

      // Verify available models (excluding already selected gpt-4 and gpt-3.5-turbo)
      expect(screen.getByTestId('available-model-gpt-4-turbo')).toBeInTheDocument();
      expect(screen.getByTestId('available-model-claude-3-sonnet')).toBeInTheDocument();

      // Select an additional model
      await selectModel(user, 'gpt-4-turbo');

      // Test connection using stored credentials
      const testButton = screen.getByTestId('test-connection-button');
      expect(testButton).not.toBeDisabled();

      await user.click(testButton);

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          title: 'Connection Test Successful',
          description: 'Test successful with stored credentials',
        });
      });

      // Update the API model
      await user.click(screen.getByTestId('update-api-model-button'));

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith(
          expect.objectContaining({
            title: 'API Model Updated',
          })
        );
      });

      expect(pushMock).toHaveBeenCalledWith('/ui/models');
    });
  });

  describe('Prefix functionality', () => {
    describe('Create Mode', () => {
      it('enables prefix checkbox, enters value, and shows prefixed model preview', async () => {
        const user = userEvent.setup();

        await act(async () => {
          render(<ApiModelForm mode="create" />, { wrapper: createWrapper() });
        });

        const prefixCheckbox = screen.getByTestId('prefix-input-checkbox');
        const prefixInput = screen.getByTestId('prefix-input');

        // Verify initial state
        expect(prefixCheckbox).not.toBeChecked();
        expect(prefixInput).toBeDisabled();

        // Enable prefix
        await user.click(prefixCheckbox);

        expect(prefixCheckbox).toBeChecked();
        expect(prefixInput).not.toBeDisabled();

        // Enter prefix value
        await user.type(prefixInput, 'azure/');
        expect(prefixInput).toHaveValue('azure/');

        // Select model and verify prefix preview
        await fetchModelsAndWait(user);
        await selectModel(user, 'gpt-4');

        expect(screen.getByText(/azure\/gpt-4/i)).toBeInTheDocument();
      });

      it('creates API model with prefix when checkbox is enabled', async () => {
        const user = userEvent.setup();

        server.use(
          ...mockCreateApiModel({
            ...mockApiModelResponse,
            prefix: 'azure/',
          })
        );

        await act(async () => {
          render(<ApiModelForm mode="create" />, { wrapper: createWrapper() });
        });

        await user.click(screen.getByTestId('api-key-input-checkbox'));
        await user.type(screen.getByTestId('api-key-input'), 'sk-test123');

        await user.click(screen.getByTestId('prefix-input-checkbox'));
        await user.type(screen.getByTestId('prefix-input'), 'azure/');

        await fetchModelsAndWait(user);
        await selectModel(user, 'gpt-4');

        await user.click(screen.getByTestId('create-api-model-button'));

        await waitFor(() => {
          expect(mockToast).toHaveBeenCalledWith(
            expect.objectContaining({
              title: 'API Model Created',
            })
          );
        });
      });

      it('sends null prefix when checkbox is unchecked', async () => {
        const user = userEvent.setup();

        server.use(
          ...mockCreateApiModel({
            ...mockApiModelResponse,
            prefix: null,
          })
        );

        await act(async () => {
          render(<ApiModelForm mode="create" />, { wrapper: createWrapper() });
        });

        await user.click(screen.getByTestId('api-key-input-checkbox'));
        await user.type(screen.getByTestId('api-key-input'), 'sk-test123');

        const prefixCheckbox = screen.getByTestId('prefix-input-checkbox');
        expect(prefixCheckbox).not.toBeChecked();

        await fetchModelsAndWait(user);
        await selectModel(user, 'gpt-4');

        await user.click(screen.getByTestId('create-api-model-button'));

        await waitFor(() => {
          expect(mockToast).toHaveBeenCalledWith(
            expect.objectContaining({
              title: 'API Model Created',
            })
          );
        });
      });
    });

    describe('Edit Mode', () => {
      it('updates prefix value', async () => {
        const user = userEvent.setup();
        const dataWithPrefix: ApiModelResponse = {
          ...mockApiModelResponse,
          prefix: 'azure/',
        };

        server.use(
          ...mockUpdateApiModel('test-api-model', {
            ...mockApiModelResponse,
            prefix: 'openai:',
          })
        );

        await act(async () => {
          render(<ApiModelForm mode="edit" initialData={dataWithPrefix} />, {
            wrapper: createWrapper(),
          });
        });

        const prefixInput = screen.getByTestId('prefix-input');
        expect(prefixInput).toHaveValue('azure/');

        await user.clear(prefixInput);
        await user.type(prefixInput, 'openai:');

        expect(prefixInput).toHaveValue('openai:');

        await user.click(screen.getByTestId('update-api-model-button'));

        await waitFor(() => {
          expect(mockToast).toHaveBeenCalledWith(
            expect.objectContaining({
              title: 'API Model Updated',
            })
          );
        });
      });

      it('removes prefix by unchecking checkbox', async () => {
        const user = userEvent.setup();
        const dataWithPrefix: ApiModelResponse = {
          ...mockApiModelResponse,
          prefix: 'azure/',
        };

        server.use(
          ...mockUpdateApiModel('test-api-model', {
            ...mockApiModelResponse,
            prefix: null,
          })
        );

        await act(async () => {
          render(<ApiModelForm mode="edit" initialData={dataWithPrefix} />, {
            wrapper: createWrapper(),
          });
        });

        const prefixCheckbox = screen.getByTestId('prefix-input-checkbox');
        expect(prefixCheckbox).toBeChecked();

        await user.click(prefixCheckbox);

        expect(prefixCheckbox).not.toBeChecked();
        expect(screen.getByTestId('prefix-input')).toBeDisabled();

        await user.click(screen.getByTestId('update-api-model-button'));

        await waitFor(() => {
          expect(mockToast).toHaveBeenCalledWith(
            expect.objectContaining({
              title: 'API Model Updated',
            })
          );
        });
      });

      it('keeps prefix unchanged when checkbox stays checked', async () => {
        const user = userEvent.setup();
        const dataWithPrefix: ApiModelResponse = {
          ...mockApiModelResponse,
          prefix: 'azure/',
        };

        server.use(
          ...mockUpdateApiModel('test-api-model', {
            ...mockApiModelResponse,
            prefix: 'azure/',
          })
        );

        await act(async () => {
          render(<ApiModelForm mode="edit" initialData={dataWithPrefix} />, {
            wrapper: createWrapper(),
          });
        });

        const prefixCheckbox = screen.getByTestId('prefix-input-checkbox');
        const prefixInput = screen.getByTestId('prefix-input');

        expect(prefixCheckbox).toBeChecked();
        expect(prefixInput).toHaveValue('azure/');

        await user.click(screen.getByTestId('update-api-model-button'));

        await waitFor(() => {
          expect(mockToast).toHaveBeenCalledWith(
            expect.objectContaining({
              title: 'API Model Updated',
            })
          );
        });
      });
    });
  });

  describe('API key optional scenarios', () => {
    describe('Create Mode - Public API', () => {
      it('performs full workflow without API key: fetch, select, test, validate, and create', async () => {
        const user = userEvent.setup();

        server.use(
          ...mockFetchApiModels({
            models: ['gpt-4', 'gpt-3.5-turbo'],
          }),
          ...mockTestApiModel({
            success: true,
            response: 'Test successful without API key',
          }),
          ...mockCreateApiModel({
            ...mockApiModelResponse,
            api_key_masked: null,
          })
        );

        await act(async () => {
          render(<ApiModelForm mode="create" />, { wrapper: createWrapper() });
        });

        const apiKeyCheckbox = screen.getByTestId('api-key-input-checkbox');
        expect(apiKeyCheckbox).not.toBeChecked();

        // Test form validation without API key - should not require API key
        await user.click(screen.getByTestId('create-api-model-button'));

        await waitFor(() => {
          const errorMessage = screen.queryByTestId('model-selection-section-error');
          expect(errorMessage).toBeInTheDocument();
        });

        const apiKeyError = screen.queryByTestId('api-key-input-error');
        expect(apiKeyError).not.toBeInTheDocument();

        // Fetch models without API key
        await fetchModelsAndWait(user);

        expect(screen.getByTestId('available-model-gpt-4')).toBeInTheDocument();
        expect(screen.getByTestId('available-model-gpt-3.5-turbo')).toBeInTheDocument();

        // Select a model
        await selectModel(user, 'gpt-4');

        // Test connection without API key
        const testButton = screen.getByTestId('test-connection-button');
        expect(testButton).not.toBeDisabled();

        await user.click(testButton);

        await waitFor(() => {
          expect(mockToast).toHaveBeenCalledWith({
            title: 'Connection Test Successful',
            description: 'Test successful without API key',
          });
        });

        // Create API model without API key
        await user.click(screen.getByTestId('create-api-model-button'));

        await waitFor(() => {
          expect(mockToast).toHaveBeenCalledWith(
            expect.objectContaining({
              title: 'API Model Created',
            })
          );
        });

        expect(pushMock).toHaveBeenCalledWith('/ui/models');
      });
    });

    describe('Edit Mode - API Key Update Scenarios (Request Body Verification)', () => {
      it.each([
        {
          description: 'sends {action: "set", value: "new-key"} when updating existing key with new value',
          initialApiKeyMasked: '***' as const,
          userActions: {
            toggleCheckbox: false, // Keep checkbox checked
            typeNewKey: 'sk-new-key-456',
          },
          expectedApiKeyRequest: { action: 'set', value: 'sk-new-key-456' },
        },
        {
          description: 'sends {action: "set", value: null} when clearing existing key by unchecking checkbox',
          initialApiKeyMasked: '***' as const,
          userActions: {
            toggleCheckbox: true, // Uncheck to remove key
            typeNewKey: undefined,
          },
          expectedApiKeyRequest: { action: 'set', value: null },
        },
        {
          description: 'sends {action: "keep"} when keeping existing key unchanged',
          initialApiKeyMasked: '***' as const,
          userActions: {
            toggleCheckbox: false, // Keep checkbox checked
            typeNewKey: undefined, // Don't type anything
          },
          expectedApiKeyRequest: { action: 'keep' },
        },
        {
          description: 'sends {action: "set", value: "new-key"} when adding key to model without existing key',
          initialApiKeyMasked: null,
          userActions: {
            toggleCheckbox: true, // Check to enable input
            typeNewKey: 'sk-new-key-789',
          },
          expectedApiKeyRequest: { action: 'set', value: 'sk-new-key-789' },
        },
        {
          description: 'sends {action: "set", value: null} when keeping model without key unchanged',
          initialApiKeyMasked: null,
          userActions: {
            toggleCheckbox: false, // Keep unchecked
            typeNewKey: undefined,
          },
          expectedApiKeyRequest: { action: 'set', value: null },
        },
      ])('$description', async ({ initialApiKeyMasked, userActions, expectedApiKeyRequest }) => {
        const user = userEvent.setup();
        const testData: ApiModelResponse = {
          ...mockApiModelResponse,
          api_key_masked: initialApiKeyMasked,
        };

        // Capture the request body using MSW handler
        let capturedRequestBody: any;

        server.use(
          typedHttp.put(ENDPOINT_API_MODEL_ID, async ({ request, params, response }) => {
            const { id } = params;
            if (id !== 'test-api-model') return;

            capturedRequestBody = await request.json();

            return response(200 as const).json({
              ...mockApiModelResponse,
              api_key_masked: initialApiKeyMasked,
            });
          })
        );

        await act(async () => {
          render(<ApiModelForm mode="edit" initialData={testData} />, {
            wrapper: createWrapper(),
          });
        });

        // Perform user actions based on parameters
        if (userActions.toggleCheckbox) {
          await user.click(screen.getByTestId('api-key-input-checkbox'));
        }

        if (userActions.typeNewKey) {
          await user.type(screen.getByTestId('api-key-input'), userActions.typeNewKey);
        }

        // Submit form
        await user.click(screen.getByTestId('update-api-model-button'));

        // Wait for success
        await waitFor(() => {
          expect(mockToast).toHaveBeenCalledWith(
            expect.objectContaining({
              title: 'API Model Updated',
            })
          );
        });

        // Verify request body contains correct api_key field
        expect(capturedRequestBody).toBeDefined();
        expect(capturedRequestBody.api_key).toEqual(expectedApiKeyRequest);

        expect(pushMock).toHaveBeenCalledWith('/ui/models');
      });
    });

    describe('Edit Mode - UI Behavior Tests', () => {
      it('toggles between keep existing key and provide new key modes', async () => {
        const user = userEvent.setup();

        await act(async () => {
          render(<ApiModelForm mode="edit" initialData={mockApiModelResponse} />, {
            wrapper: createWrapper(),
          });
        });

        const apiKeyCheckbox = screen.getByTestId('api-key-input-checkbox');
        const apiKeyInput = screen.getByTestId('api-key-input');

        expect(apiKeyCheckbox).toBeChecked();
        expect(apiKeyInput).not.toBeDisabled();

        await user.click(apiKeyCheckbox);

        expect(apiKeyCheckbox).not.toBeChecked();
        expect(apiKeyInput).toBeDisabled();

        await user.click(apiKeyCheckbox);

        expect(apiKeyCheckbox).toBeChecked();
        expect(apiKeyInput).not.toBeDisabled();
      });

      it('fetches models using stored credentials (type:"id")', async () => {
        const user = userEvent.setup();

        server.use(
          ...mockFetchApiModels({
            models: ['gpt-4', 'gpt-3.5-turbo', 'claude-3-sonnet'],
          })
        );

        await act(async () => {
          render(<ApiModelForm mode="edit" initialData={mockApiModelResponse} />, {
            wrapper: createWrapper(),
          });
        });

        const apiKeyCheckbox = screen.getByTestId('api-key-input-checkbox');
        expect(apiKeyCheckbox).toBeChecked();

        await user.click(screen.getByTestId('fetch-models-button'));

        await waitFor(() => {
          expect(mockToast).toHaveBeenCalledWith({
            title: 'Models Fetched Successfully',
            description: 'Found 3 available models',
          });
        });

        expect(screen.getByTestId('available-model-claude-3-sonnet')).toBeInTheDocument();
      });
    });
  });

  describe('Button states', () => {
    describe('Create Mode', () => {
      it('fetch button enabled with default base_url, test button disabled without models', async () => {
        await renderCreateFormWithoutApiKey();

        const fetchButton = screen.getByTestId('fetch-models-button');
        const testButton = screen.getByTestId('test-connection-button');

        expect(fetchButton).not.toBeDisabled();
        expect(testButton).toBeDisabled();
      });

      it('enables test button after selecting model', async () => {
        const user = await renderCreateFormWithoutApiKey();
        await fetchModelsAndWait(user);
        await selectModel(user, 'gpt-4');

        const testButton = screen.getByTestId('test-connection-button');
        expect(testButton).not.toBeDisabled();
      });
    });

    describe('Edit Mode', () => {
      it('enables both buttons with stored credentials and initial models', async () => {
        await renderEditFormUsingStoredCreds();

        const fetchButton = screen.getByTestId('fetch-models-button');
        const testButton = screen.getByTestId('test-connection-button');

        expect(fetchButton).not.toBeDisabled();
        expect(testButton).not.toBeDisabled();
      });
    });
  });

  describe('Error handling', () => {
    describe('Create Mode', () => {
      it('handles fetch models error', async () => {
        server.use(
          ...mockFetchApiModelsError({
            code: 'authentication_error',
            message: 'Invalid API key',
            type: 'invalid_request_error',
          })
        );

        const user = await renderCreateFormWithApiKey('invalid-key');
        await user.type(screen.getByTestId('base-url-input'), 'https://api.openai.com/v1');
        await user.click(screen.getByRole('button', { name: /Fetch Models/i }));

        await waitFor(() => {
          expect(mockToast).toHaveBeenCalledWith({
            title: 'Failed to Fetch Models',
            description: 'Invalid API key',
            variant: 'destructive',
          });
        });
      });

      it('handles test connection error', async () => {
        server.use(
          ...mockTestApiModelError({
            code: 'connection_error',
            message: 'Connection failed',
            type: 'internal_server_error',
          })
        );

        const user = await renderCreateFormWithApiKey('invalid-key');
        await user.type(screen.getByTestId('base-url-input'), 'https://api.openai.com/v1');
        await fetchModelsAndWait(user);
        await selectModel(user, 'gpt-4');
        await user.click(screen.getByTestId('test-connection-button'));

        await waitFor(() => {
          expect(mockToast).toHaveBeenCalledWith({
            title: 'Connection Test Failed',
            description: 'Connection failed',
            variant: 'destructive',
          });
        });
      });

      it('handles form submission error and stays on page', async () => {
        server.use(
          ...mockCreateApiModelError({
            code: 'conflict_error',
            message: 'API model with this ID already exists',
            type: 'invalid_request_error',
          }),
          ...mockCreateApiModelError({
            code: 'conflict_error',
            message: 'API model with this ID already exists',
            type: 'invalid_request_error',
          })
        );

        const user = await renderCreateFormWithApiKey('sk-test123');
        await fetchModelsAndWait(user);
        await selectModel(user, 'gpt-4');
        await user.click(screen.getByTestId('create-api-model-button'));

        await waitFor(() => {
          expect(mockToast).toHaveBeenCalledWith({
            title: 'Failed to Create API Model',
            description: 'API model with this ID already exists',
            variant: 'destructive',
          });
        });

        expect(pushMock).not.toHaveBeenCalled();
      });
    });
  });

  describe('Cancel button', () => {
    it('navigates back to models page when cancel is clicked', async () => {
      const user = userEvent.setup();

      await act(async () => {
        render(<ApiModelForm mode="create" />, { wrapper: createWrapper() });
      });

      await user.click(screen.getByTestId('cancel-button'));

      expect(pushMock).toHaveBeenCalledWith('/ui/models');
    });
  });

  describe('Fetch models network call and error handling', () => {
    it('allows fetch models without credentials (API may return 401 if required)', async () => {
      const user = userEvent.setup();

      server.use(
        ...mockFetchApiModels({
          models: ['gpt-4', 'gpt-3.5-turbo'],
        })
      );

      await act(async () => {
        render(<ApiModelForm mode="create" />, { wrapper: createWrapper() });
      });

      const fetchButton = screen.getByTestId('fetch-models-button');
      expect(fetchButton).not.toBeDisabled();

      await user.click(fetchButton);

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          title: 'Models Fetched Successfully',
          description: 'Found 2 available models',
        });
      });
    });

    it('triggers network call when fetch models clicked with credentials', async () => {
      const user = userEvent.setup();

      server.use(
        ...mockFetchApiModels({
          models: ['gpt-4', 'gpt-3.5-turbo'],
        })
      );

      await act(async () => {
        render(<ApiModelForm mode="create" />, { wrapper: createWrapper() });
      });

      await user.click(screen.getByTestId('api-key-input-checkbox'));
      await user.type(screen.getByTestId('api-key-input'), 'sk-test123');

      const fetchButton = screen.getByTestId('fetch-models-button');
      await user.click(fetchButton);

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          title: 'Models Fetched Successfully',
          description: 'Found 2 available models',
        });
      });

      expect(screen.getByTestId('available-model-gpt-4')).toBeInTheDocument();
      expect(screen.getByTestId('available-model-gpt-3.5-turbo')).toBeInTheDocument();
    });
  });

  describe('Full workflow: fetch → select → test connection', () => {
    describe('Create Mode with API Key', () => {
      it('completes full flow from fetch to test connection', async () => {
        server.use(
          ...mockFetchApiModels({ models: ['gpt-4', 'gpt-3.5-turbo'] }),
          ...mockTestApiModel({ success: true, response: 'Test successful!' })
        );

        const user = await renderCreateFormWithApiKey('sk-test123');
        await user.click(screen.getByTestId('fetch-models-button'));

        await waitFor(() => {
          expect(mockToast).toHaveBeenCalledWith({
            title: 'Models Fetched Successfully',
            description: 'Found 2 available models',
          });
        });

        expect(screen.getByTestId('available-model-gpt-4')).toBeInTheDocument();
        expect(screen.getByTestId('available-model-gpt-3.5-turbo')).toBeInTheDocument();

        await selectModel(user, 'gpt-4');
        expect(screen.getByTestId('selected-model-gpt-4')).toBeInTheDocument();

        const testButton = screen.getByTestId('test-connection-button');
        expect(testButton).not.toBeDisabled();

        await user.click(testButton);

        await waitFor(() => {
          expect(mockToast).toHaveBeenCalledWith({
            title: 'Connection Test Successful',
            description: 'Test successful!',
          });
        });
      });
    });

    describe('Edit Mode with Stored Credentials', () => {
      it('completes full flow using stored credentials', async () => {
        server.use(
          ...mockFetchApiModels({ models: ['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo'] }),
          ...mockTestApiModel({ success: true, response: 'Connection test successful' })
        );

        const user = await renderEditFormUsingStoredCreds();
        await user.click(screen.getByTestId('fetch-models-button'));

        await waitFor(() => {
          expect(mockToast).toHaveBeenCalledWith({
            title: 'Models Fetched Successfully',
            description: 'Found 3 available models',
          });
        });

        // In Edit mode, only gpt-4-turbo is available (gpt-4 and gpt-3.5-turbo already selected)
        expect(screen.getByTestId('available-model-gpt-4-turbo')).toBeInTheDocument();

        // Test button should already be enabled because models are already selected
        const testButton = screen.getByTestId('test-connection-button');
        expect(testButton).not.toBeDisabled();

        await user.click(testButton);

        await waitFor(() => {
          expect(mockToast).toHaveBeenCalledWith({
            title: 'Connection Test Successful',
            description: 'Connection test successful',
          });
        });
      });
    });
  });
});
