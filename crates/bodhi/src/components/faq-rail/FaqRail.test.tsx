import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { type ReactNode } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { FaqLink, FaqRail, FaqRailHeader, useFaqRail, type FaqGroup } from '@/components/faq-rail';
import { ShellContext, type ShellContextValue } from '@/components/shell/ShellContext';

const openRail = vi.fn();
const closeRail = vi.fn();
const collapseRail = vi.fn();

function Shell({ children, isMobile = false }: { children: ReactNode; isMobile?: boolean }) {
  const value: ShellContextValue = {
    collapsed: false,
    isMobile,
    openPop: null,
    setOpenPop: () => {},
    openRail,
    closeRail,
    collapseRail,
  };
  return <ShellContext.Provider value={value}>{children}</ShellContext.Provider>;
}

const GROUPS: FaqGroup[] = [
  {
    label: 'setup',
    entries: [
      { id: 'faq-install', question: 'How do I install it?', answer: <p>Use your package manager.</p> },
      { id: 'faq-path', question: 'Where does it look?', answer: <p>On your PATH.</p> },
    ],
  },
  {
    label: 'running',
    entries: [{ id: 'faq-dns', question: 'Why the DNS warning?', answer: <p>A record already exists.</p> }],
  },
];

/** A page that shows an error carrying a deep link into the rail. */
function Page({ isMobile = false }: { isMobile?: boolean }) {
  const { faqProps, reveal } = useFaqRail();
  return (
    <Shell isMobile={isMobile}>
      <FaqRailHeader />
      <div data-testid="page-error">
        Something went wrong. <FaqLink id="faq-dns" reveal={reveal} />
      </div>
      <FaqRail groups={GROUPS} subtitle="Everything in one place." {...faqProps} />
    </Shell>
  );
}

beforeEach(() => {
  openRail.mockClear();
  closeRail.mockClear();
  collapseRail.mockClear();
});

describe('FaqRail', () => {
  it('lists every entry collapsed, so the rail opens short', () => {
    render(<Page />);

    expect(screen.getByText('setup')).toBeInTheDocument();
    expect(screen.getByText('running')).toBeInTheDocument();
    for (const id of ['faq-install', 'faq-path', 'faq-dns']) {
      expect(screen.getByTestId(`faq-entry-${id}`)).toHaveAttribute('data-open', 'false');
    }
    expect(screen.queryByText('A record already exists.')).not.toBeInTheDocument();
  });

  it('expands and collapses an entry on click', async () => {
    const user = userEvent.setup();
    render(<Page />);
    const entry = screen.getByTestId('faq-entry-faq-install');

    await user.click(within(entry).getByRole('button', { name: /how do i install it/i }));
    expect(entry).toHaveAttribute('data-open', 'true');
    expect(screen.getByText('Use your package manager.')).toBeInTheDocument();

    await user.click(within(entry).getByRole('button', { name: /how do i install it/i }));
    expect(entry).toHaveAttribute('data-open', 'false');
  });

  it('opens the rail on the relevant answer when an error links to it', async () => {
    const user = userEvent.setup();
    const scrollIntoView = vi.fn();
    Element.prototype.scrollIntoView = scrollIntoView;
    render(<Page />);

    await user.click(within(screen.getByTestId('page-error')).getByTestId('faq-link-faq-dns'));

    expect(openRail).toHaveBeenCalled();
    expect(screen.getByTestId('faq-entry-faq-dns')).toHaveAttribute('data-open', 'true');
    expect(screen.getByText('A record already exists.')).toBeInTheDocument();
    await waitFor(() => expect(scrollIntoView).toHaveBeenCalled());
    // Unrelated entries stay shut, so the answer asked for is the one in view.
    expect(screen.getByTestId('faq-entry-faq-install')).toHaveAttribute('data-open', 'false');
  });

  it('re-opens the rail when the same answer is asked for twice', async () => {
    const user = userEvent.setup();
    render(<Page />);
    const link = screen.getByTestId('faq-link-faq-dns');

    await user.click(link);
    expect(openRail).toHaveBeenCalledTimes(1);

    // The user collapsed the rail in between; asking again must bring it back
    // even though the entry is already expanded.
    await user.click(link);
    await waitFor(() => expect(openRail).toHaveBeenCalledTimes(2));
    expect(screen.getByTestId('faq-entry-faq-dns')).toHaveAttribute('data-open', 'true');
  });

  it('collapses the rail from its header on desktop and closes the drawer on mobile', async () => {
    const user = userEvent.setup();
    const { unmount } = render(<Page />);

    await user.click(screen.getByTestId('faq-rail-close'));
    expect(collapseRail).toHaveBeenCalled();
    expect(closeRail).not.toHaveBeenCalled();
    unmount();

    render(<Page isMobile />);
    await user.click(screen.getByTestId('faq-rail-close'));
    expect(closeRail).toHaveBeenCalled();
  });
});
