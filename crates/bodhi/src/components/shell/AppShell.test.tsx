import { AppShell } from '@/components/shell';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

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

describe('AppShell', () => {
  it('renders the nav with the active section highlighted', async () => {
    const user = userEvent.setup();
    render(
      <AppShell section="models" subPage="all-models">
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

    expect(screen.getByTestId('shell-sub-all-models')).toBeInTheDocument();
    expect(screen.getByTestId('shell-sub-new-local-model')).toBeInTheDocument();

    const active = screen.getByTestId('shell-sub-new-api-model');
    expect(active).toBeInTheDocument();
    expect(active).toHaveClass('on');
  });

  it('does not render sub-pages from other sections', () => {
    render(
      <AppShell section="chat">
        <div>page content</div>
      </AppShell>
    );

    expect(screen.queryByTestId('shell-sub-all-models')).not.toBeInTheDocument();
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

  it('collapses the sidebar to the icon rail when the toggle is clicked', async () => {
    const user = userEvent.setup();
    render(
      <AppShell section="models" subPage="all-models">
        <div>page content</div>
      </AppShell>
    );

    // expanded: sub-page link carries its label text inline
    expect(screen.getByText('All Models')).toBeInTheDocument();

    const toggle = screen.getByTitle('Collapse sidebar');
    await user.click(toggle);

    // icon-rail variant: the section button + sub-page icon buttons are still
    // present via data-testid, but the inline text label is gone.
    expect(screen.getByTestId('shell-nav-models')).toBeInTheDocument();
    expect(screen.getByTestId('shell-sub-all-models')).toBeInTheDocument();
    expect(screen.queryByText('All Models')).not.toBeInTheDocument();
  });
});
