import ModelRouterForm from '@/routes/models/router/-components/ModelRouterForm';
import { mockModels } from '@/test-utils/msw-v2/handlers/models';
import { mockCreateModelRouter } from '@/test-utils/msw-v2/handlers/model-routers';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { AliasResponse } from '@bodhiapp/ts-client';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

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
  };
});

const mockToast = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast, dismiss: () => {} }),
}));
vi.mock('@/components/ui/toaster', () => ({ Toaster: () => null }));

Object.assign(window.HTMLElement.prototype, {
  scrollIntoView: vi.fn(),
  releasePointerCapture: vi.fn(),
  hasPointerCapture: vi.fn(),
  setPointerCapture: vi.fn(),
});

setupMswV2();

beforeAll(() => {
  Element.prototype.hasPointerCapture = vi.fn(() => false);
  Element.prototype.setPointerCapture = vi.fn();
  Element.prototype.releasePointerCapture = vi.fn();
});

afterEach(() => vi.clearAllMocks());
beforeEach(() => {
  navigateMock.mockClear();
  mockToast.mockClear();
});

const localAlias: AliasResponse = {
  source: 'user',
  alias: 'llama3:instruct',
  repo: 'meta/llama3',
  filename: 'llama3.gguf',
  snapshot: 'main',
} as AliasResponse;

describe('ModelRouterForm', () => {
  it('creates a router with one local target', async () => {
    server.use(
      ...mockModels({ data: [localAlias], total: 1, page: 1, page_size: 30 }, { stub: true }),
      ...mockCreateModelRouter({ alias: 'my-stack' })
    );
    const user = userEvent.setup();
    render(<ModelRouterForm mode="create" />, { wrapper: createWrapper() });

    // name
    await user.type(screen.getByTestId('router-alias-input'), 'my-stack');

    // add a target
    await user.click(screen.getByTestId('add-target'));
    expect(screen.getByTestId('target-row-0')).toBeInTheDocument();

    // pick the local alias
    await user.click(screen.getByTestId('target-alias-0'));
    await user.click(await screen.findByText('llama3:instruct'));

    // local alias pins its own model (read-only)
    await waitFor(() => {
      expect(screen.getByTestId('target-model-0')).toHaveValue('llama3:instruct');
    });

    // submit
    await user.click(screen.getByTestId('router-submit'));

    await waitFor(() => {
      expect(navigateMock).toHaveBeenCalledWith({ to: '/models/' });
    });
  });

  it('validates that an alias name is required before submit reaches the server', async () => {
    server.use(...mockModels({ data: [localAlias], total: 1, page: 1, page_size: 30 }, { stub: true }));
    const user = userEvent.setup();
    render(<ModelRouterForm mode="create" />, { wrapper: createWrapper() });

    // No targets initially, helper text shown
    expect(screen.getByTestId('no-targets')).toBeInTheDocument();

    // add + remove a target toggles the empty state
    await user.click(screen.getByTestId('add-target'));
    expect(screen.getByTestId('target-row-0')).toBeInTheDocument();
    await user.click(screen.getByTestId('target-remove-0'));
    expect(screen.getByTestId('no-targets')).toBeInTheDocument();
  });
});
