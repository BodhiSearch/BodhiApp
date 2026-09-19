import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type ReactNode } from 'react';

import { ChevronDown, CircleHelp, X } from 'lucide-react';

import { useShell } from '@/components/shell/ShellContext';
import { cn } from '@/lib/utils';

import './faq-rail.css';

/**
 * A page's in-context help, shown in the shell's right rail.
 *
 * The rail is collapsed by default, so the page can point at a specific answer:
 * `reveal(id)` opens the rail, expands that entry, scrolls to it and flashes it.
 * That is what makes an error message actionable rather than merely descriptive.
 *
 * Nothing here knows about any particular page — pass `groups` as data.
 */

export interface FaqEntry {
  /** Stable DOM id; also the deep-link target used by `reveal`. */
  id: string;
  question: string;
  answer: ReactNode;
}

export interface FaqGroup {
  label: string;
  entries: FaqEntry[];
}

/** A request to reveal one entry. `seq` makes repeat requests for the same id distinct. */
export interface FaqRevealRequest {
  id: string;
  seq: number;
}

export interface FaqRailProps {
  groups: readonly FaqGroup[];
  open: readonly string[];
  flash: string | null;
  request: FaqRevealRequest | null;
  onToggle: (id: string) => void;
  subtitle?: string;
}

export interface UseFaqRailResult {
  /** Spread straight onto `<FaqRail groups={...} />`. */
  faqProps: Omit<FaqRailProps, 'groups' | 'subtitle'>;
  /** Open the rail on the given entry. Safe to call from anywhere on the page. */
  reveal: (id: string) => void;
}

const FLASH_MS = 1400;

/**
 * Owns which entries are expanded and the reveal request. The hook deliberately
 * does not open the rail itself: the rail may not be mounted when a page action
 * fires, so it records the request and `<FaqRail>` acts on it.
 */
export function useFaqRail(): UseFaqRailResult {
  const [open, setOpen] = useState<string[]>([]);
  const [flash, setFlash] = useState<string | null>(null);
  const [request, setRequest] = useState<FaqRevealRequest | null>(null);
  const flashTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(
    () => () => {
      if (flashTimer.current) clearTimeout(flashTimer.current);
    },
    []
  );

  const onToggle = useCallback((id: string) => {
    setOpen((ids) => (ids.includes(id) ? ids.filter((each) => each !== id) : [...ids, id]));
  }, []);

  const reveal = useCallback((id: string) => {
    setOpen((ids) => (ids.includes(id) ? ids : [...ids, id]));
    setFlash(id);
    setRequest((previous) => ({ id, seq: (previous?.seq ?? 0) + 1 }));
    if (flashTimer.current) clearTimeout(flashTimer.current);
    flashTimer.current = setTimeout(() => setFlash(null), FLASH_MS);
  }, []);

  const faqProps = useMemo(() => ({ open, flash, request, onToggle }), [open, flash, request, onToggle]);
  return { faqProps, reveal };
}

const FaqRevealContext = createContext<(id: string) => void>(() => {});

/**
 * Makes `reveal` available to anything inside, so a panel buried a few levels
 * down can link into the rail without the page threading a prop through every
 * component in between.
 */
export function FaqRevealProvider({ reveal, children }: { reveal: (id: string) => void; children: ReactNode }) {
  return <FaqRevealContext.Provider value={reveal}>{children}</FaqRevealContext.Provider>;
}

export function useFaqReveal(): (id: string) => void {
  return useContext(FaqRevealContext);
}

/** The rail's own title row. Collapses on desktop, closes the drawer on mobile. */
export function FaqRailHeader({ title = 'Help & debugging' }: { title?: string }) {
  const { collapseRail, closeRail, isMobile } = useShell();
  return (
    <div className="faq-railhead" data-testid="faq-rail-header">
      <span className="faq-railhead-t">
        <CircleHelp className="h-3.5 w-3.5 shrink-0" aria-hidden="true" />
        <span className="truncate">{title}</span>
      </span>
      <button
        type="button"
        onClick={() => (isMobile ? closeRail() : collapseRail())}
        title="Close"
        aria-label="Close help"
        className="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground"
        data-testid="faq-rail-close"
      >
        <X className="h-4 w-4" aria-hidden="true" />
      </button>
    </div>
  );
}

function scrollToEntry(id: string) {
  // Run after the expansion has committed, or the target is still zero-height.
  setTimeout(() => {
    const element = document.getElementById(id);
    // jsdom has no layout and no scrollIntoView; guard rather than branch on env.
    element?.scrollIntoView?.({ block: 'start', behavior: 'smooth' });
  }, 0);
}

export function FaqRail({ groups, open, flash, request, onToggle, subtitle }: FaqRailProps) {
  const { openRail } = useShell();
  const seq = request?.seq ?? 0;
  const targetId = request?.id;

  useEffect(() => {
    if (!seq || !targetId) return;
    openRail();
    scrollToEntry(targetId);
    // Keyed on seq so asking for the same entry twice still re-opens and re-scrolls.
  }, [seq, targetId, openRail]);

  return (
    <div className="faq-rail" data-testid="faq-rail">
      {subtitle && <p className="faq-rail-sub">{subtitle}</p>}
      {groups.map((group) => (
        <div key={group.label} className="faq-group">
          <div className="faq-glabel">{group.label}</div>
          {group.entries.map((entry) => {
            const isOpen = open.includes(entry.id);
            return (
              <div
                key={entry.id}
                id={entry.id}
                data-testid={`faq-entry-${entry.id}`}
                data-open={isOpen ? 'true' : 'false'}
                className={cn('faq-item', isOpen && 'is-open', flash === entry.id && 'is-flash')}
              >
                <button
                  type="button"
                  onClick={() => onToggle(entry.id)}
                  aria-expanded={isOpen}
                  aria-controls={`${entry.id}-answer`}
                  className="faq-q"
                >
                  <span className="min-w-0">{entry.question}</span>
                  <span className="faq-chev">
                    <ChevronDown className="h-[15px] w-[15px]" aria-hidden="true" />
                  </span>
                </button>
                {isOpen && (
                  <div id={`${entry.id}-answer`} className="faq-a">
                    {entry.answer}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      ))}
    </div>
  );
}

/**
 * A shell command inside an answer: a fixed OS column beside the command itself,
 * stacking on narrow rails. Exported so pages write content, not layout.
 */
export function FaqCommand({ os, children }: { os: string; children: ReactNode }) {
  return (
    <div className="faq-cmd">
      <span className="faq-cmd-os">{os}</span>
      <code className="min-w-0">{children}</code>
    </div>
  );
}

/**
 * The affordance that turns a problem into an answer. Put one in any error or
 * attention panel and point it at the entry that explains the situation.
 */
export function FaqLink({
  id,
  reveal,
  children = 'Why this happens',
  className,
}: {
  id: string;
  /** Defaults to the nearest `FaqRevealProvider`. */
  reveal?: (id: string) => void;
  children?: ReactNode;
  className?: string;
}) {
  const fromContext = useFaqReveal();
  const onReveal = reveal ?? fromContext;
  return (
    <button
      type="button"
      onClick={() => onReveal(id)}
      data-testid={`faq-link-${id}`}
      className={cn('text-sm font-medium underline underline-offset-2 hover:no-underline', className)}
    >
      {children}
    </button>
  );
}
