import NewApiModel from '@/app/ui/api-models/new/page';
import { ENDPOINT_APP_INFO, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen } from '@testing-library/react';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { afterAll, afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

// Mock the ApiModelForm component since we've already tested it
vi.mock('../ApiModelForm', () => ({
  default: ({ isEditMode }: { isEditMode: boolean }) => (
    <div data-testid="api-model-form">
      <div data-testid="edit-mode">{isEditMode ? 'true' : 'false'}</div>
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

const server = setupServer();

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => server.resetHandlers());

describe('NewApiModel Page', () => {
  beforeEach(() => {
    server.use(
      rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ status: 'ready' }));
      }),
      rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
        return res(ctx.json({ logged_in: true, email: 'test@example.com' }));
      })
    );
  });

  it('renders the new API model page correctly', async () => {
    await act(async () => {
      render(<NewApiModel />, { wrapper: createWrapper() });
    });

    // Check that AppInitializer is configured correctly
    const appInitializer = screen.getByTestId('app-initializer');
    expect(appInitializer).toHaveAttribute('data-allowed-status', 'ready');
    expect(appInitializer).toHaveAttribute('data-authenticated', 'true');

    // Check that ApiModelForm is rendered in create mode
    const apiModelForm = screen.getByTestId('api-model-form');
    expect(apiModelForm).toBeInTheDocument();

    const editMode = screen.getByTestId('edit-mode');
    expect(editMode).toHaveTextContent('false');

    // Check that the correct heading is shown
    expect(screen.getByText('Create New API Model')).toBeInTheDocument();
  });

  it('passes correct props to ApiModelForm', async () => {
    await act(async () => {
      render(<NewApiModel />, { wrapper: createWrapper() });
    });

    // Verify that isEditMode is false
    expect(screen.getByTestId('edit-mode')).toHaveTextContent('false');
  });
});
