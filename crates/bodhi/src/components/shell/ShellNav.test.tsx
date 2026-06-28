import React from 'react';

import { AppShell } from '@/components/shell';
import { setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { fireEvent, render as rtlRender, screen, waitFor, type RenderOptions } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

setupMswV2();

// Forward `ref` (and pass through tabIndex/role/onClick/onKeyDown) so focus + roving-tabindex
// assertions work — the shared AppShell.test mock drops the ref.
vi.mock('@tanstack/react-router', async () => {
  const actual = await vi.importActual('@tanstack/react-router');
  return {
    ...actual,
    Link: React.forwardRef<HTMLAnchorElement, { to: string; children: React.ReactNode } & Record<string, unknown>>(
      ({ to, children, ...rest }, ref) => (
        <a ref={ref} href={to} {...rest}>
          {children}
        </a>
      )
    ),
  };
});

const render = (ui: React.ReactElement, options?: RenderOptions) =>
  rtlRender(ui, { wrapper: createWrapper(), ...options });

function renderNav(section = 'chat') {
  return render(
    <AppShell section={section}>
      <div>page content</div>
    </AppShell>
  );
}

// The expanded-sidebar nav order mirrors SHELL_NAV: chat, models, mcp, ...
async function openMenu() {
  const user = userEvent.setup();
  await user.click(screen.getByTestId('shell-nav-trigger'));
  return user;
}

describe('ShellNav keyboard navigation + auto-close', () => {
  it('opens and focuses the active item', async () => {
    renderNav('models');
    await openMenu();
    await waitFor(() => expect(screen.getByTestId('shell-nav-models')).toHaveFocus());
  });

  it('ArrowDown / ArrowUp move focus with no wrap at the ends', async () => {
    renderNav('chat'); // active = first item
    await openMenu();
    const chat = screen.getByTestId('shell-nav-chat');
    await waitFor(() => expect(chat).toHaveFocus());

    fireEvent.keyDown(chat, { key: 'ArrowDown' });
    expect(screen.getByTestId('shell-nav-models')).toHaveFocus();

    // ArrowUp at the first item stays put (no wrap to the last)
    fireEvent.keyDown(screen.getByTestId('shell-nav-models'), { key: 'ArrowUp' });
    expect(chat).toHaveFocus();
    fireEvent.keyDown(chat, { key: 'ArrowUp' });
    expect(chat).toHaveFocus();
  });

  it('Home / End jump to first / last item', async () => {
    renderNav('models');
    await openMenu();
    const models = screen.getByTestId('shell-nav-models');
    await waitFor(() => expect(models).toHaveFocus());

    fireEvent.keyDown(models, { key: 'End' });
    const last = screen.getByTestId('shell-nav-settings');
    expect(last).toHaveFocus();

    fireEvent.keyDown(last, { key: 'Home' });
    expect(screen.getByTestId('shell-nav-chat')).toHaveFocus();
  });

  it('clicking an item closes the menu (auto-close)', async () => {
    renderNav('chat');
    const user = await openMenu();
    expect(screen.getByTestId('shell-nav-models')).toBeInTheDocument();
    await user.click(screen.getByTestId('shell-nav-models'));
    await waitFor(() => expect(screen.queryByTestId('shell-nav-models')).not.toBeInTheDocument());
  });

  it('Escape closes the menu and returns focus to the trigger', async () => {
    renderNav('chat');
    await openMenu();
    const chat = screen.getByTestId('shell-nav-chat');
    await waitFor(() => expect(chat).toHaveFocus());

    fireEvent.keyDown(chat, { key: 'Escape' });
    await waitFor(() => expect(screen.queryByTestId('shell-nav-chat')).not.toBeInTheDocument());
    expect(screen.getByTestId('shell-nav-trigger')).toHaveFocus();
  });

  it('ignores modifier-held arrow keys', async () => {
    renderNav('chat');
    await openMenu();
    const chat = screen.getByTestId('shell-nav-chat');
    await waitFor(() => expect(chat).toHaveFocus());

    fireEvent.keyDown(chat, { key: 'ArrowDown', metaKey: true });
    expect(chat).toHaveFocus();
  });

  it('exposes menu a11y: aria-expanded toggles, items are menuitems with roving tabindex', async () => {
    renderNav('models');
    const trigger = screen.getByTestId('shell-nav-trigger');
    expect(trigger).toHaveAttribute('aria-haspopup', 'menu');
    expect(trigger).toHaveAttribute('aria-expanded', 'false');

    await openMenu();
    expect(trigger).toHaveAttribute('aria-expanded', 'true');

    const active = screen.getByTestId('shell-nav-models');
    expect(active).toHaveAttribute('role', 'menuitem');
    // only the focused/active item is tabbable
    expect(active).toHaveAttribute('tabindex', '0');
    expect(screen.getByTestId('shell-nav-chat')).toHaveAttribute('tabindex', '-1');
  });
});
