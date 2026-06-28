import NewModelRouter from '@/routes/models/router/new/index';
import EditModelRouter from '@/routes/models/router/edit/index';
import { ShellChromeProvider, useShellSlots } from '@/components/shell';
import { createWrapper } from '@/tests/wrapper';
import { render, screen, waitFor, within } from '@testing-library/react';
import { server } from '@/test-utils/msw-v2/setup';
import { afterAll, afterEach, beforeAll, describe, expect, it, vi } from 'vitest';

import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { mockModels } from '@/test-utils/msw-v2/handlers/models';
import { mockGetModelRouter } from '@/test-utils/msw-v2/handlers/model-routers';

// Edit route reads ?id via useSearch; the search mock is overridden per-describe below.
let searchMock: () => Record<string, unknown> = () => ({});
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
    useSearch: () => searchMock(),
  };
});

vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: vi.fn(), dismiss: () => {} }),
}));

beforeAll(() => server.listen({ onUnhandledRequest: 'error' }));
afterAll(() => server.close());
afterEach(() => {
  server.resetHandlers();
  vi.clearAllMocks();
  searchMock = () => ({});
});

function SlotsConsumer() {
  const { breadcrumb, rail } = useShellSlots();
  const crumbs = Array.isArray(breadcrumb) ? breadcrumb.map((b) => b.label).join(' / ') : '';
  return (
    <>
      <div data-testid="harness-breadcrumb">{crumbs}</div>
      <div data-testid="harness-rail">{rail}</div>
    </>
  );
}

const readyHandlers = () => [...mockAppInfoReady(), ...mockUserLoggedIn({ role: 'resource_user' })];

describe('New Model Router — V2 route chrome', () => {
  it('publishes the breadcrumb, the page container, and the live rail', async () => {
    server.use(...readyHandlers(), ...mockModels({ data: [], total: 0, page: 1, page_size: 30 }, { stub: true }));

    render(
      <ShellChromeProvider>
        <SlotsConsumer />
        <NewModelRouter />
      </ShellChromeProvider>,
      { wrapper: createWrapper() }
    );

    await waitFor(() => expect(screen.getByTestId('model-router-form')).toBeInTheDocument());
    expect(screen.getByTestId('harness-breadcrumb')).toHaveTextContent('Bodhi / Models / New Model Router');
    expect(screen.getByTestId('router-form-page')).toBeInTheDocument();
    // The "Routing & help" rail is published (its body root testid lives inside the rail slot).
    expect(within(screen.getByTestId('harness-rail')).getByTestId('router-rail')).toBeInTheDocument();
  });
});

describe('Edit Model Router — V2 route chrome', () => {
  it('publishes the Edit breadcrumb and prefills the loaded router', async () => {
    searchMock = () => ({ id: 'router-123' });
    server.use(
      ...readyHandlers(),
      ...mockModels({ data: [], total: 0, page: 1, page_size: 30 }, { stub: true }),
      ...mockGetModelRouter('router-123', { alias: 'my-stack' }, { stub: true })
    );

    render(
      <ShellChromeProvider>
        <SlotsConsumer />
        <EditModelRouter />
      </ShellChromeProvider>,
      { wrapper: createWrapper() }
    );

    await waitFor(() => expect(screen.getByTestId('model-router-form')).toBeInTheDocument());
    expect(screen.getByTestId('harness-breadcrumb')).toHaveTextContent('Bodhi / Models / Edit Model Router');
    expect(screen.getByTestId('router-form-page')).toBeInTheDocument();
    expect(screen.getByTestId('router-alias-input')).toHaveValue('my-stack');
  });

  it('shows an error page when no id is provided (and still publishes the breadcrumb)', async () => {
    searchMock = () => ({});
    server.use(...readyHandlers());

    render(
      <ShellChromeProvider>
        <SlotsConsumer />
        <EditModelRouter />
      </ShellChromeProvider>,
      { wrapper: createWrapper() }
    );

    await waitFor(() =>
      expect(screen.getByTestId('harness-breadcrumb')).toHaveTextContent('Bodhi / Models / Edit Model Router')
    );
    expect(screen.getByText('No model router ID provided')).toBeInTheDocument();
  });
});
