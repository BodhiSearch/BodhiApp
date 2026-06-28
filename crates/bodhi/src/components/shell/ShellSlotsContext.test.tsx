import { ShellSlotsProvider, useShellChrome, useShellSlots } from '@/components/shell/ShellSlotsContext';
import { render, screen } from '@testing-library/react';
import { useState } from 'react';
import { describe, expect, it } from 'vitest';
import userEvent from '@testing-library/user-event';

/** Renders the currently-published slots so the test can assert on them. */
function SlotsProbe() {
  const slots = useShellSlots();
  return (
    <div data-testid="probe">
      <span data-testid="probe-actions">{slots.headerActions ?? 'none'}</span>
      <span data-testid="probe-sidebar">{slots.sidebar ?? 'none'}</span>
    </div>
  );
}

function Publisher({ label }: { label: string }) {
  useShellChrome({ headerActions: <span>{label}</span> });
  return <div>publisher</div>;
}

function SidebarPublisher({ label }: { label: string }) {
  useShellChrome({ sidebar: <span>{label}</span> });
  return <div>sidebar-publisher</div>;
}

function LayoutProbe() {
  const slots = useShellSlots();
  return (
    <div>
      <span data-testid="probe-main-scroll">{String(slots.mainScroll)}</span>
      <span data-testid="probe-rail-scroll">{String(slots.railScroll)}</span>
      <span data-testid="probe-content-class">{slots.contentClass ?? 'none'}</span>
      <span data-testid="probe-rail-width">{String(slots.railWidth)}</span>
      <span data-testid="probe-section">{slots.section ?? 'none'}</span>
    </div>
  );
}

function LayoutPublisher() {
  useShellChrome({
    mainScroll: false,
    railScroll: false,
    contentClass: 'flush',
    railWidth: 360,
    sidebarWidth: 260,
    resizeKey: 'chat',
    section: 'chat',
  });
  return <div>layout-publisher</div>;
}

describe('ShellSlotsContext', () => {
  it('publishes a screen-provided slot to the root consumer', () => {
    render(
      <ShellSlotsProvider>
        <SlotsProbe />
        <Publisher label="New Token" />
      </ShellSlotsProvider>
    );
    expect(screen.getByTestId('probe-actions')).toHaveTextContent('New Token');
  });

  it('clears the published slot when the publishing screen unmounts', async () => {
    const user = userEvent.setup();

    function Harness() {
      const [show, setShow] = useState(true);
      return (
        <ShellSlotsProvider>
          <button onClick={() => setShow(false)}>hide</button>
          <SlotsProbe />
          {show && <Publisher label="New Token" />}
        </ShellSlotsProvider>
      );
    }

    render(<Harness />);
    expect(screen.getByTestId('probe-actions')).toHaveTextContent('New Token');

    await user.click(screen.getByRole('button', { name: 'hide' }));
    expect(screen.getByTestId('probe-actions')).toHaveTextContent('none');
  });

  it('returns empty slots with no publisher mounted', () => {
    render(
      <ShellSlotsProvider>
        <SlotsProbe />
      </ShellSlotsProvider>
    );
    expect(screen.getByTestId('probe-actions')).toHaveTextContent('none');
    expect(screen.getByTestId('probe-sidebar')).toHaveTextContent('none');
  });

  it('round-trips layout-override fields (mainScroll/railScroll/contentClass/railWidth/section) and clears them on unmount', async () => {
    const user = userEvent.setup();

    function Harness() {
      const [show, setShow] = useState(true);
      return (
        <ShellSlotsProvider>
          <button onClick={() => setShow(false)}>hide</button>
          <LayoutProbe />
          {show && <LayoutPublisher />}
        </ShellSlotsProvider>
      );
    }

    render(<Harness />);
    expect(screen.getByTestId('probe-main-scroll')).toHaveTextContent('false');
    expect(screen.getByTestId('probe-rail-scroll')).toHaveTextContent('false');
    expect(screen.getByTestId('probe-content-class')).toHaveTextContent('flush');
    expect(screen.getByTestId('probe-rail-width')).toHaveTextContent('360');
    expect(screen.getByTestId('probe-section')).toHaveTextContent('chat');

    await user.click(screen.getByRole('button', { name: 'hide' }));
    expect(screen.getByTestId('probe-main-scroll')).toHaveTextContent('undefined');
    expect(screen.getByTestId('probe-content-class')).toHaveTextContent('none');
    expect(screen.getByTestId('probe-section')).toHaveTextContent('none');
  });

  it('publishes a screen-provided sidebar slot and clears it on unmount', async () => {
    const user = userEvent.setup();

    function Harness() {
      const [show, setShow] = useState(true);
      return (
        <ShellSlotsProvider>
          <button onClick={() => setShow(false)}>hide</button>
          <SlotsProbe />
          {show && <SidebarPublisher label="Settings Groups" />}
        </ShellSlotsProvider>
      );
    }

    render(<Harness />);
    expect(screen.getByTestId('probe-sidebar')).toHaveTextContent('Settings Groups');

    await user.click(screen.getByRole('button', { name: 'hide' }));
    expect(screen.getByTestId('probe-sidebar')).toHaveTextContent('none');
  });
});
