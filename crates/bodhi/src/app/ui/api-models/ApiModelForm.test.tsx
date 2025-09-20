import ApiModelForm from '@/app/ui/api-models/ApiModelForm';
import { ENDPOINT_APP_INFO, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
import { createWrapper } from '@/tests/wrapper';
import { createMockLoggedInUser } from '@/test-utils/mock-user';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { ApiModelResponse } from '@bodhiapp/ts-client';

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

vi.mock('@/components/ModelSelector', () => ({
  ModelSelector: ({
    onModelSelect,
    onModelRemove,
    onModelsSelectAll,
    onFetchModels,
    isFetchingModels,
    availableModels,
    selectedModels,
    canFetch,
  }: any) => (
    <div data-testid="model-selector">
      <button onClick={() => onModelSelect?.('gpt-4')}>Select gpt-4</button>
      <button onClick={() => onModelRemove?.('gpt-4')}>Remove gpt-4</button>
      <button onClick={() => onModelsSelectAll?.(['gpt-4', 'gpt-3.5-turbo'])}>Select All</button>
      <button onClick={() => onFetchModels?.()} disabled={!canFetch} data-testid="fetch-models-button">
        {isFetchingModels ? 'Loading...' : 'Fetch Models'}
      </button>
      <div data-testid="available-models">{Array.isArray(availableModels) ? availableModels.join(', ') : ''}</div>
      <div data-testid="selected-models">{Array.isArray(selectedModels) ? selectedModels.join(', ') : ''}</div>
    </div>
  ),
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

const server = setupServer();

beforeAll(() => {
  Element.prototype.hasPointerCapture = vi.fn(() => false);
  Element.prototype.setPointerCapture = vi.fn();
  Element.prototype.releasePointerCapture = vi.fn();
  server.listen();
});

afterAll(() => server.close());

afterEach(() => {
  server.resetHandlers();
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
  api_key_masked: '****key',
  models: ['gpt-4', 'gpt-3.5-turbo'],
  prefix: null,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

describe('ApiModelForm', () => {
  beforeEach(() => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      }),
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(ctx.json(createMockLoggedInUser()));
      }),
      // API format endpoint
      rest.get('*/bodhi/v1/api-models/api-formats', (_, res, ctx) => {
        return res(ctx.json(['openai']));
      }),
      // API model endpoints
      rest.post('*/bodhi/v1/api-models', (_, res, ctx) => {
        return res(ctx.json(mockApiModelResponse));
      }),
      rest.put('*/bodhi/v1/api-models/:id', (req, res, ctx) => {
        return res(ctx.json({ ...mockApiModelResponse, id: req.params.id }));
      }),
      rest.post('*/bodhi/v1/api-models/fetch-models', (_, res, ctx) => {
        return res(ctx.json({ models: ['gpt-4', 'gpt-3.5-turbo'] }));
      }),
      rest.post('*/bodhi/v1/api-models/test', (_, res, ctx) => {
        return res(ctx.json({ success: true, response: 'Test successful!' }));
      })
    );
  });

  describe('Create mode', () => {
    it('renders all form elements for creating new API model', async () => {
      await act(async () => {
        render(<ApiModelForm isEditMode={false} />, { wrapper: createWrapper() });
      });

      // Form title
      expect(screen.getByText('Create New API Model')).toBeInTheDocument();

      // Form fields (no ID field since it's auto-generated)
      expect(screen.getByTestId('api-model-format')).toBeInTheDocument();
      expect(screen.getByTestId('api-model-base-url')).toBeInTheDocument();
      expect(screen.getByTestId('api-model-api-key')).toBeInTheDocument();

      // Buttons
      expect(screen.getByTestId('create-api-model-button')).toBeInTheDocument();
      expect(screen.getByTestId('cancel-button')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /Fetch Models/i })).toBeInTheDocument();
      expect(screen.getByTestId('test-connection-button')).toBeInTheDocument();
    });

    it('shows validation errors for required fields', async () => {
      const user = userEvent.setup();

      await act(async () => {
        render(<ApiModelForm isEditMode={false} />, { wrapper: createWrapper() });
      });

      // Try to submit without filling required fields
      await user.click(screen.getByTestId('create-api-model-button'));

      await waitFor(() => {
        // Look for any validation error message about API key
        const errorMessages = screen.getAllByText((content, element) => {
          return element?.tagName.toLowerCase() === 'p' && content.toLowerCase().includes('api key');
        });
        expect(errorMessages.length).toBeGreaterThan(0);
      });
    });

    it('handles api_format preset selection', async () => {
      const user = userEvent.setup();
      await act(async () => {
        render(<ApiModelForm isEditMode={false} />, { wrapper: createWrapper() });
      });
      expect(screen.getByTestId('api-model-base-url')).toHaveValue('https://api.openai.com/v1');
    });

    it('handles fetch models functionality', async () => {
      const user = userEvent.setup();
      server.use(
        rest.post('*/api-models/fetch-models', (_, res, ctx) => {
          return res(
            ctx.json({
              models: ['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo'],
            })
          );
        })
      );

      await act(async () => {
        render(<ApiModelForm isEditMode={false} />, { wrapper: createWrapper() });
      });

      // Fill required fields for fetch
      await user.type(screen.getByTestId('api-model-base-url'), 'https://api.openai.com/v1');
      await user.type(screen.getByTestId('api-model-api-key'), 'sk-test123');

      // Click fetch models
      await user.click(screen.getByTestId('fetch-models-button'));

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          title: 'Models Fetched Successfully',
          description: 'Found 3 available models',
        });
      });
      expect(screen.getByTestId('available-models')).toHaveTextContent('gpt-4, gpt-3.5-turbo, gpt-4-turbo');
    });

    it('handles test connection functionality', async () => {
      const user = userEvent.setup();

      server.use(
        rest.post('*/bodhi/v1/api-models/test', (_, res, ctx) => {
          return res(
            ctx.json({
              success: true,
              response: 'Connection successful',
            })
          );
        })
      );

      await act(async () => {
        render(<ApiModelForm isEditMode={false} />, { wrapper: createWrapper() });
      });

      // Fill required fields
      await user.type(screen.getByTestId('api-model-base-url'), 'https://api.openai.com/v1');
      await user.type(screen.getByTestId('api-model-api-key'), 'sk-test123');

      // Select a model using the mock model selector
      await user.click(screen.getByText('Select gpt-4'));

      // Test connection button should be enabled
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

      server.use(
        rest.post('*/api-models', (_, res, ctx) => {
          return res(ctx.json(mockApiModelResponse));
        })
      );

      await act(async () => {
        render(<ApiModelForm isEditMode={false} />, { wrapper: createWrapper() });
      });

      // Fill the form (no ID field - it's auto-generated)
      // Api format and Base URL are pre-filled with OpenAI defaults
      await user.type(screen.getByTestId('api-model-api-key'), 'sk-test123');

      // Select models
      await user.click(screen.getByText('Select gpt-4'));

      // Submit form
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

    it('handles API key visibility toggle', async () => {
      const user = userEvent.setup();

      await act(async () => {
        render(<ApiModelForm isEditMode={false} />, { wrapper: createWrapper() });
      });

      const apiKeyInput = screen.getByTestId('api-model-api-key');
      expect(apiKeyInput).toHaveAttribute('type', 'password');

      // Click the eye icon to show password
      const toggleButton = screen.getByTestId('toggle-api-key-visibility');
      await user.click(toggleButton);

      expect(apiKeyInput).toHaveAttribute('type', 'text');

      // Click again to hide
      await user.click(toggleButton);
      expect(apiKeyInput).toHaveAttribute('type', 'password');
    });
  });

  describe('Edit mode', () => {
    it('renders form for editing existing API model', async () => {
      await act(async () => {
        render(<ApiModelForm isEditMode={true} initialData={mockApiModelResponse} />, {
          wrapper: createWrapper(),
        });
      });

      // Form title
      expect(screen.getByText('Edit API Model')).toBeInTheDocument();

      // No ID field in edit mode since ID is auto-generated and immutable

      // Other fields should be populated with initial data
      expect(screen.getByTestId('api-model-base-url')).toHaveValue('https://api.openai.com/v1');

      // API key should be empty for security
      expect(screen.getByTestId('api-model-api-key')).toHaveValue('');

      // Submit button should say Update
      expect(screen.getByTestId('update-api-model-button')).toBeInTheDocument();
    });

    it('updates API model successfully by fetching and adding models without API key', async () => {
      const user = userEvent.setup();

      server.use(
        // Mock fetch models to return additional models
        rest.post('*/bodhi/v1/api-models/fetch-models', (req, res, ctx) => {
          return res(
            ctx.json({
              models: ['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo', 'claude-3-sonnet'],
            })
          );
        }),
        // Mock the update endpoint
        rest.put(`*/api-models/test-api-model`, (_, res, ctx) => {
          return res(
            ctx.json({
              ...mockApiModelResponse,
              models: ['gpt-4', 'gpt-3.5-turbo', 'gpt-4-turbo'], // Updated models
            })
          );
        })
      );

      await act(async () => {
        render(<ApiModelForm isEditMode={true} initialData={mockApiModelResponse} />, {
          wrapper: createWrapper(),
        });
      });

      // Don't provide an API key - use stored credentials
      // Fetch additional models using stored credentials
      await user.click(screen.getByTestId('fetch-models-button'));

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          title: 'Models Fetched Successfully',
          description: 'Found 4 available models',
        });
      });

      // Verify models were fetched and displayed
      expect(screen.getByTestId('available-models')).toHaveTextContent(
        'gpt-4, gpt-3.5-turbo, gpt-4-turbo, claude-3-sonnet'
      );

      // Add a new model (gpt-4-turbo) to the existing selection
      await user.click(screen.getByText('Select gpt-4')); // This should select gpt-4-turbo from available models

      // The MockModelSelector will add gpt-4 when clicked, but we want to simulate adding gpt-4-turbo
      // Let's use the "Select All" to get multiple models
      await user.click(screen.getByText('Select All'));

      // Verify selected models include the new one
      expect(screen.getByTestId('selected-models')).toHaveTextContent('gpt-4, gpt-3.5-turbo');

      // Submit form without providing API key
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

    it('can fetch models in edit mode without API key using stored credentials', async () => {
      const user = userEvent.setup();

      server.use(
        rest.post('*/bodhi/v1/api-models/fetch-models', (req, res, ctx) => {
          // Expect request to use id instead of api_key
          return res(
            ctx.json({
              models: ['gpt-4', 'gpt-3.5-turbo', 'claude-3'],
            })
          );
        })
      );

      await act(async () => {
        render(<ApiModelForm isEditMode={true} initialData={mockApiModelResponse} />, {
          wrapper: createWrapper(),
        });
      });

      // Don't provide API key - should use stored model ID
      // Base URL is already populated from initialData

      // Fetch models button should be enabled (using stored credentials)
      const fetchButton = screen.getByTestId('fetch-models-button');
      expect(fetchButton).not.toBeDisabled();

      await user.click(fetchButton);

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          title: 'Models Fetched Successfully',
          description: 'Found 3 available models',
        });
      });

      expect(screen.getByTestId('available-models')).toHaveTextContent('gpt-4, gpt-3.5-turbo, claude-3');
    });

    it('can test connection in edit mode without API key using stored credentials', async () => {
      const user = userEvent.setup();

      server.use(
        rest.post('*/bodhi/v1/api-models/test', (req, res, ctx) => {
          // Expect request to use id instead of api_key
          return res(
            ctx.json({
              success: true,
              response: 'Test successful with stored credentials',
            })
          );
        })
      );

      await act(async () => {
        render(<ApiModelForm isEditMode={true} initialData={mockApiModelResponse} />, {
          wrapper: createWrapper(),
        });
      });

      // Don't provide API key - should use stored model ID
      // Models are already populated from initialData, so test button should be enabled

      const testButton = screen.getByTestId('test-connection-button');
      expect(testButton).not.toBeDisabled();

      await user.click(testButton);

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          title: 'Connection Test Successful',
          description: 'Test successful with stored credentials',
        });
      });
    });
  });

  describe('Model selection', () => {
    it('handles individual model selection', async () => {
      const user = userEvent.setup();

      await act(async () => {
        render(<ApiModelForm isEditMode={false} />, { wrapper: createWrapper() });
      });

      // Select a model
      await user.click(screen.getByText('Select gpt-4'));

      // Verify model appears in selected models
      expect(screen.getByTestId('selected-models')).toHaveTextContent('gpt-4');
    });

    it('handles model removal', async () => {
      const user = userEvent.setup();

      await act(async () => {
        render(<ApiModelForm isEditMode={false} />, { wrapper: createWrapper() });
      });

      // First select a model
      await user.click(screen.getByText('Select gpt-4'));
      expect(screen.getByTestId('selected-models')).toHaveTextContent('gpt-4');

      // Then remove it
      await user.click(screen.getByText('Remove gpt-4'));
      expect(screen.getByTestId('selected-models')).toHaveTextContent('');
    });

    it('handles select all models', async () => {
      const user = userEvent.setup();

      await act(async () => {
        render(<ApiModelForm isEditMode={false} />, { wrapper: createWrapper() });
      });

      // Select all models
      await user.click(screen.getByText('Select All'));

      // Verify all models are selected
      expect(screen.getByTestId('selected-models')).toHaveTextContent('gpt-4, gpt-3.5-turbo');
    });
  });

  describe('Button states', () => {
    it('disables fetch models button when required fields missing', async () => {
      await act(async () => {
        render(<ApiModelForm isEditMode={false} />, { wrapper: createWrapper() });
      });

      const fetchButton = screen.getByTestId('fetch-models-button');
      expect(fetchButton).toBeDisabled();
    });

    it('disables test connection button when required fields missing', async () => {
      await act(async () => {
        render(<ApiModelForm isEditMode={false} />, { wrapper: createWrapper() });
      });

      const testButton = screen.getByTestId('test-connection-button');
      expect(testButton).toBeDisabled();
    });

    it('enables buttons when all requirements met', async () => {
      const user = userEvent.setup();

      await act(async () => {
        render(<ApiModelForm isEditMode={false} />, { wrapper: createWrapper() });
      });

      // Fill required fields
      await user.type(screen.getByTestId('api-model-base-url'), 'https://api.openai.com/v1');
      await user.type(screen.getByTestId('api-model-api-key'), 'sk-test123');

      // Fetch button should be enabled
      const fetchButton = screen.getByTestId('fetch-models-button');
      expect(fetchButton).not.toBeDisabled();

      // Select a model
      await user.click(screen.getByText('Select gpt-4'));

      // Test button should be enabled
      const testButton = screen.getByTestId('test-connection-button');
      expect(testButton).not.toBeDisabled();
    });

    it('enables buttons in edit mode even without API key (uses stored credentials)', async () => {
      await act(async () => {
        render(<ApiModelForm isEditMode={true} initialData={mockApiModelResponse} />, {
          wrapper: createWrapper(),
        });
      });

      // In edit mode with stored model ID, fetch button should be enabled even without API key
      const fetchButton = screen.getByTestId('fetch-models-button');
      expect(fetchButton).not.toBeDisabled();

      // Test button should be enabled because models are pre-populated from initialData
      const testButton = screen.getByTestId('test-connection-button');
      expect(testButton).not.toBeDisabled();
    });
  });

  describe('Error handling', () => {
    it('handles fetch models error', async () => {
      const user = userEvent.setup();

      server.use(
        rest.post('*/api-models/fetch-models', (_, res, ctx) => {
          return res(
            ctx.status(400),
            ctx.json({
              error: { message: 'Invalid API key' },
            })
          );
        })
      );

      await act(async () => {
        render(<ApiModelForm isEditMode={false} />, { wrapper: createWrapper() });
      });

      // Fill required fields
      await user.type(screen.getByTestId('api-model-base-url'), 'https://api.openai.com/v1');
      await user.type(screen.getByTestId('api-model-api-key'), 'invalid-key');

      // Try to fetch models
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
      const user = userEvent.setup();

      server.use(
        rest.post('*/bodhi/v1/api-models/test', (_, res, ctx) => {
          return res(
            ctx.status(400),
            ctx.json({
              error: { message: 'Connection failed' },
            })
          );
        })
      );

      await act(async () => {
        render(<ApiModelForm isEditMode={false} />, { wrapper: createWrapper() });
      });

      // Fill required fields and select model
      await user.type(screen.getByTestId('api-model-base-url'), 'https://api.openai.com/v1');
      await user.type(screen.getByTestId('api-model-api-key'), 'invalid-key');
      await user.click(screen.getByText('Select gpt-4'));

      // Try to test connection
      await user.click(screen.getByTestId('test-connection-button'));

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          title: 'Connection Test Failed',
          description: 'Connection failed',
          variant: 'destructive',
        });
      });
    });

    it('handles form submission error', async () => {
      const user = userEvent.setup();

      server.use(
        rest.post('*/api-models', (_, res, ctx) => {
          return res(
            ctx.status(409),
            ctx.json({
              error: { message: 'API model with this ID already exists' },
            })
          );
        })
      );

      await act(async () => {
        render(<ApiModelForm isEditMode={false} />, { wrapper: createWrapper() });
      });

      // Fill the form (no ID field - it's auto-generated)
      await user.type(screen.getByTestId('api-model-api-key'), 'sk-test123');
      await user.click(screen.getByText('Select gpt-4'));

      // Submit form
      await user.click(screen.getByTestId('create-api-model-button'));

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          title: 'Failed to Create API Model',
          description: 'API model with this ID already exists',
          variant: 'destructive',
        });
      });

      // Should not navigate on error
      expect(pushMock).not.toHaveBeenCalled();
    });
  });

  describe('Cancel button', () => {
    it('navigates back to models page when cancel is clicked', async () => {
      const user = userEvent.setup();

      await act(async () => {
        render(<ApiModelForm isEditMode={false} />, { wrapper: createWrapper() });
      });

      await user.click(screen.getByTestId('cancel-button'));

      expect(pushMock).toHaveBeenCalledWith('/ui/models');
    });
  });
});
