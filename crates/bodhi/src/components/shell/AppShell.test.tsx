import { AppShell } from '@/components/shell';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { render as rtlRender, screen, waitFor, type RenderOptions } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

setupMswV2();

vi.mock('@tanstack/react-router', async () => {
  const actual = await vi.importActual('@tanstack/react-router');
  return {
    ...actual,
    Link: ({ to, children, ...rest }: { to: string; children: React.ReactNode } & Record<string, unknown>) => (
      <a href={to} {...rest}>
        {children}
      </a>
    ),
  };
});

// ShellNav reads app-info (for the multi-tenant nav filter), so every render needs a QueryClient.
// With no app-info handler the query is idle → deployment is undefined → default (non-MT) nav.
const render = (ui: React.ReactElement, options?: RenderOptions) =>
  rtlRender(ui, { wrapper: createWrapper(), ...options });

describe('AppShell', () => {
  it('renders the nav with the active section highlighted', async () => {
    const user = userEvent.setup();
    render(
      <AppShell section="models" subPage="my-models">
        <div>page content</div>
      </AppShell>
    );

    // the current section is shown in the trigger; open the dropdown to see entries
    await user.click(screen.getByTestId('shell-nav-trigger'));

    const modelsNav = screen.getByTestId('shell-nav-models');
    expect(modelsNav).toBeInTheDocument();
    expect(modelsNav).toHaveClass('on');

    // a non-active section is also present but not highlighted
    const chatNav = screen.getByTestId('shell-nav-chat');
    expect(chatNav).not.toHaveClass('on');
  });

  it('renders sub-page items for the active section with the active sub-page highlighted', () => {
    render(
      <AppShell section="models" subPage="new-api-model">
        <div>page content</div>
      </AppShell>
    );

    expect(screen.getByTestId('shell-sub-my-models')).toBeInTheDocument();
    expect(screen.getByTestId('shell-sub-new-local-model')).toBeInTheDocument();
    // Explore · Local Models is visible by default (standalone deployment).
    expect(screen.getByTestId('shell-sub-explore-local')).toBeInTheDocument();

    const active = screen.getByTestId('shell-sub-new-api-model');
    expect(active).toBeInTheDocument();
    expect(active).toHaveClass('on');
  });

  it('hides the Explore · Local Models sub-page in multi-tenant deployments', async () => {
    server.use(...mockAppInfo({ deployment: 'multi_tenant' }, { stub: true }));
    render(
      <AppShell section="models" subPage="my-models">
        <div>page content</div>
      </AppShell>
    );
    // The local-catalog feature is hidden (no downloads in multi-tenant); siblings remain.
    await waitFor(() => expect(screen.queryByTestId('shell-sub-explore-local')).not.toBeInTheDocument());
    expect(screen.getByTestId('shell-sub-my-models')).toBeInTheDocument();
    expect(screen.getByTestId('shell-sub-new-api-model')).toBeInTheDocument();
  });

  it('does not render sub-pages from other sections', () => {
    render(
      <AppShell section="chat">
        <div>page content</div>
      </AppShell>
    );

    expect(screen.queryByTestId('shell-sub-my-models')).not.toBeInTheDocument();
    expect(screen.queryByTestId('shell-sub-discover')).not.toBeInTheDocument();
  });

  it('renders the page content and breadcrumb', () => {
    render(
      <AppShell
        section="mcp"
        breadcrumb={[
          { label: 'MCP', href: '/mcps/' },
          { label: 'Discover', current: true },
        ]}
      >
        <div>my page body</div>
      </AppShell>
    );

    expect(screen.getByText('my page body')).toBeInTheDocument();
    expect(screen.getByText('Discover')).toBeInTheDocument();
  });

  it('auto-opens the rail when rail content appears (e.g. selecting a row)', () => {
    const { rerender } = render(
      <AppShell section="api-keys" railDefaultOpen={false}>
        <div>page content</div>
      </AppShell>
    );
    // no rail content yet
    expect(screen.queryByTestId('the-rail')).not.toBeInTheDocument();

    // a screen publishes rail content → the rail column should open and show it
    rerender(
      <AppShell section="api-keys" railDefaultOpen={false} rail={<div data-testid="the-rail">details</div>}>
        <div>page content</div>
      </AppShell>
    );
    const rail = screen.getByTestId('the-rail');
    expect(rail).toBeInTheDocument();
    // the rail column is not collapsed (width 0) — its containing shell lacks rail-collapsed
    const shell = rail.closest('.shell');
    expect(shell).not.toHaveClass('rail-collapsed');
  });

  it('opens the mobile rail drawer when rail content appears (rail-open class)', () => {
    // On mobile the rail is a fixed drawer gated by `.shell.rail-open`; publishing rail
    // content must add that class so the drawer slides in on the first row select.
    const originalMatchMedia = window.matchMedia;
    window.matchMedia = ((query: string) =>
      ({
        matches: query.includes('max-width:767px'),
        media: query,
        onchange: null,
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        addListener: vi.fn(),
        removeListener: vi.fn(),
        dispatchEvent: vi.fn(),
      }) as unknown as MediaQueryList) as typeof window.matchMedia;
    try {
      const { rerender } = render(
        <AppShell section="api-keys" railDefaultOpen={false}>
          <div>page content</div>
        </AppShell>
      );
      rerender(
        <AppShell section="api-keys" railDefaultOpen={false} rail={<div data-testid="m-rail">details</div>}>
          <div>page content</div>
        </AppShell>
      );

      const shell = screen.getByTestId('m-rail').closest('.shell');
      expect(shell).toHaveClass('rail-open');
    } finally {
      window.matchMedia = originalMatchMedia;
    }
  });

  it('collapses the sidebar to the icon rail when the toggle is clicked', async () => {
    const user = userEvent.setup();
    render(
      <AppShell section="models" subPage="my-models">
        <div>page content</div>
      </AppShell>
    );

    // expanded: sub-page link carries its label text inline
    expect(screen.getByText('My Models')).toBeInTheDocument();

    const toggle = screen.getByTitle('Collapse sidebar');
    await user.click(toggle);

    // icon-rail variant: the section button + sub-page icon buttons are still
    // present via data-testid, but the inline text label is gone.
    expect(screen.getByTestId('shell-nav-models')).toBeInTheDocument();
    expect(screen.getByTestId('shell-sub-my-models')).toBeInTheDocument();
    expect(screen.queryByText('My Models')).not.toBeInTheDocument();
  });
});
