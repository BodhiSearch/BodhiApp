import EditApiModel from '@/app/ui/api-models/edit/page';
import { ENDPOINT_APP_INFO, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
import { createMockLoggedInUser } from '@/test-utils/mock-user';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen } from '@testing-library/react';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { ApiModelResponse } from '@bodhiapp/ts-client';

const mockApiModel: ApiModelResponse = {
  id: 'test-api-model',
  api_format: 'openai',
  base_url: 'https://api.openai.com/v1',
  api_key_masked: '****key',
  models: ['gpt-4', 'gpt-3.5-turbo'],
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

// Mock the ApiModelForm component since we've already tested it
vi.mock('@/app/ui/api-models/ApiModelForm', () => ({
  default: ({ isEditMode, initialData }: { isEditMode: boolean; initialData?: ApiModelResponse }) => (
    <div data-testid="api-model-form">
      <div data-testid="edit-mode">{isEditMode ? 'true' : 'false'}</div>
      <div data-testid="initial-data">{initialData?.id || 'no-data'}</div>
      <h1>{isEditMode ? 'Edit API Model' : 'Create New API Model'}</h1>
    </div>
  ),
}));

// Mock AppInitializer to bypass authentication and app status checks
vi.mock('@/components/AppInitializer', () => ({
  default: ({ children, allowedStatus, authenticated }: any) => (
    <div data-testid="app-initializer" data-allowed-status={allowedStatus} data-authenticated={authenticated}>
      {children}
    </div>
  ),
}));

// Mock useSearchParams to simulate URL parameters
const mockSearchParams = vi.fn();
vi.mock('next/navigation', () => ({
  useSearchParams: () => ({
    get: mockSearchParams,
  }),
}));

const server = setupServer();

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => {
  server.resetHandlers();
  vi.clearAllMocks();
});

describe('EditApiModel Page', () => {
  beforeEach(() => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      }),
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(ctx.json(createMockLoggedInUser()));
      }),
      rest.get('*/api-models/test-api-model', (_, res, ctx) => {
        return res(ctx.json(mockApiModel));
      })
    );
  });

  it('renders the edit API model page correctly when API model is found', async () => {
    // Mock the id parameter
    mockSearchParams.mockReturnValue('test-api-model');

    await act(async () => {
      render(<EditApiModel />, { wrapper: createWrapper() });
    });

    // Wait for the API call to complete and data to load
    await screen.findByTestId('api-model-form');

    // Check that AppInitializer is configured correctly
    const appInitializer = screen.getByTestId('app-initializer');
    expect(appInitializer).toHaveAttribute('data-allowed-status', 'ready');
    expect(appInitializer).toHaveAttribute('data-authenticated', 'true');

    // Check that ApiModelForm is rendered in edit mode
    const apiModelForm = screen.getByTestId('api-model-form');
    expect(apiModelForm).toBeInTheDocument();

    const editMode = screen.getByTestId('edit-mode');
    expect(editMode).toHaveTextContent('true');

    // Check that initial data is passed to the form
    const initialData = screen.getByTestId('initial-data');
    expect(initialData).toHaveTextContent('test-api-model');

    // Check that the correct heading is shown
    expect(screen.getByText('Edit API Model')).toBeInTheDocument();
  });

  it('shows loading state when API model is being fetched', async () => {
    // Mock the id parameter
    mockSearchParams.mockReturnValue('test-api-model');

    // Mock a slow API response
    server.use(
      rest.get('*/api-models/test-api-model', (_, res, ctx) => {
        return res(ctx.delay(1000), ctx.json(mockApiModel));
      })
    );

    await act(async () => {
      render(<EditApiModel />, { wrapper: createWrapper() });
    });

    // Should show loading state initially
    expect(screen.getByText('Loading API model...')).toBeInTheDocument();
  });

  it('shows error state when API model is not found', async () => {
    // Mock the id parameter
    mockSearchParams.mockReturnValue('non-existent-model');

    server.use(
      rest.get('*/api-models/non-existent-model', (_, res, ctx) => {
        return res(
          ctx.status(404),
          ctx.json({
            error: { message: 'API model not found' },
          })
        );
      })
    );

    await act(async () => {
      render(<EditApiModel />, { wrapper: createWrapper() });
    });

    // Should show error message
    await screen.findByText('API model not found');
  });

  it('shows error state when no id parameter is provided', async () => {
    // Mock missing id parameter
    mockSearchParams.mockReturnValue(null);

    await act(async () => {
      render(<EditApiModel />, { wrapper: createWrapper() });
    });

    // Should show error message for missing ID
    expect(screen.getByText('No API model ID provided')).toBeInTheDocument();
  });

  it('makes API call with correct URL when id is provided', async () => {
    const apiCall = vi.fn();
    mockSearchParams.mockReturnValue('test-model-123');

    server.use(
      rest.get('*/api-models/test-model-123', (req, res, ctx) => {
        apiCall(req.url.toString());
        return res(ctx.json(mockApiModel));
      })
    );

    await act(async () => {
      render(<EditApiModel />, { wrapper: createWrapper() });
    });

    await screen.findByTestId('api-model-form');

    // Verify the correct API endpoint was called
    expect(apiCall).toHaveBeenCalledWith(expect.stringContaining('/api-models/test-model-123'));
  });
});
