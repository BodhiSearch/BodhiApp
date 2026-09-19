import { useEffect, useState, type ReactNode } from 'react';

import {
  AlertOctagon,
  AlertTriangle,
  Check,
  CheckCircle2,
  ChevronUp,
  CircleDashed,
  Copy,
  Info,
  Lock,
  Pencil,
  RefreshCw,
  ShieldAlert,
  X,
} from 'lucide-react';

import { Input } from '@/components/ui/input';
import type { Tone } from '@/routes/tunnels/-shared/remoteAccessState';

const PILL_TONE: Record<Tone, string> = {
  idle: 'bg-muted text-muted-foreground',
  checking: 'bg-blue-500/10 text-blue-700 dark:text-blue-300',
  ok: 'bg-green-500/10 text-green-700 dark:text-green-400',
  attn: 'bg-amber-500/10 text-amber-800 dark:text-amber-300',
  fail: 'bg-destructive/10 text-destructive',
};

const PILL_ICON: Record<Tone, typeof Check> = {
  idle: CircleDashed,
  checking: RefreshCw,
  ok: Check,
  attn: AlertTriangle,
  fail: X,
};

export function StatusPill({
  tone = 'idle',
  label,
  spin,
  testId,
}: {
  tone?: Tone;
  label: string;
  spin?: boolean;
  testId?: string;
}) {
  const Icon = PILL_ICON[tone];
  return (
    <span
      data-testid={testId}
      data-test-state={tone}
      className={`inline-flex shrink-0 items-center gap-1.5 rounded-full px-2 py-1 text-xs font-medium ${PILL_TONE[tone]}`}
    >
      <Icon className={`h-3 w-3 ${spin ? 'animate-spin motion-reduce:animate-none' : ''}`} aria-hidden="true" />
      {label}
    </span>
  );
}

type NoteTone = 'info' | 'warn' | 'ok' | 'fail' | 'checking';

const NOTE_TONE: Record<NoteTone, { box: string; icon: typeof Info; iconClass: string }> = {
  info: { box: 'border-border bg-muted/50', icon: Info, iconClass: 'text-muted-foreground' },
  warn: {
    box: 'border-amber-500/40 bg-amber-500/5',
    icon: ShieldAlert,
    iconClass: 'text-amber-700 dark:text-amber-300',
  },
  ok: {
    box: 'border-green-500/40 bg-green-500/5',
    icon: CheckCircle2,
    iconClass: 'text-green-700 dark:text-green-400',
  },
  fail: { box: 'border-destructive/50 bg-destructive/5', icon: AlertOctagon, iconClass: 'text-destructive' },
  checking: { box: 'border-blue-500/40 bg-blue-500/5', icon: RefreshCw, iconClass: 'text-blue-700 dark:text-blue-300' },
};

export function Note({
  tone = 'info',
  title,
  children,
  testId,
}: {
  tone?: NoteTone;
  title?: ReactNode;
  children?: ReactNode;
  testId?: string;
}) {
  const { box, icon: Icon, iconClass } = NOTE_TONE[tone];
  return (
    <div
      data-testid={testId}
      data-test-state={tone}
      aria-live="polite"
      className={`flex gap-3 rounded-md border p-3 text-sm ${box}`}
    >
      <Icon
        className={`mt-0.5 h-4 w-4 shrink-0 ${iconClass} ${tone === 'checking' ? 'animate-spin motion-reduce:animate-none' : ''}`}
        aria-hidden="true"
      />
      <div className="min-w-0 space-y-2">
        {title && <p className="font-medium">{title}</p>}
        {children}
      </div>
    </div>
  );
}

const NODE_TONE: Record<Tone, string> = {
  idle: 'border-border text-muted-foreground',
  checking: 'border-blue-500/40 bg-blue-500/10 text-blue-700 dark:text-blue-300',
  ok: 'border-green-500/40 bg-green-500/10 text-green-700 dark:text-green-400',
  attn: 'border-amber-500/40 bg-amber-500/10 text-amber-800 dark:text-amber-300',
  fail: 'border-destructive/50 bg-destructive/10 text-destructive',
};

/**
 * One rung of the setup ladder. A finished step folds to a single line so focus
 * stays on the current one; `lockFolded` keeps it shut because the tunnel is up
 * and the local setup underneath it cannot be re-cut.
 */
export function Step({
  n,
  tone = 'idle',
  title,
  pill,
  spin,
  status,
  children,
  collapsible,
  summary,
  defaultFolded,
  lockFolded,
  action,
}: {
  n: number;
  tone?: Tone;
  title: string;
  pill?: string;
  spin?: boolean;
  status?: ReactNode;
  children?: ReactNode;
  collapsible?: boolean;
  summary?: string;
  defaultFolded?: boolean;
  lockFolded?: boolean;
  action?: ReactNode;
}) {
  const done = tone === 'ok';
  const [open, setOpen] = useState(false);
  useEffect(() => {
    setOpen(false);
  }, [defaultFolded, lockFolded]);
  const folded = Boolean(collapsible && done && (lockFolded || (defaultFolded && !open)));

  return (
    <section
      data-testid={`tunnel-step-${n}`}
      data-test-state={tone}
      data-test-folded={folded ? 'true' : 'false'}
      className="grid grid-cols-[auto_1fr] gap-x-3 sm:gap-x-4"
    >
      <div
        className={`mt-0.5 flex h-7 w-7 shrink-0 items-center justify-center rounded-full border text-sm font-medium ${NODE_TONE[done ? 'ok' : tone]}`}
        aria-hidden="true"
      >
        {done ? <Check className="h-3.5 w-3.5" /> : n}
      </div>

      {folded ? (
        <button
          type="button"
          data-testid={`tunnel-step-${n}-unfold`}
          aria-expanded={false}
          disabled={lockFolded}
          onClick={() => !lockFolded && setOpen(true)}
          className="flex min-w-0 flex-wrap items-center gap-x-3 gap-y-1 pb-6 text-left disabled:cursor-default"
        >
          <span className="text-base font-medium">{title}</span>
          {summary && <span className="min-w-0 truncate font-mono text-xs text-muted-foreground">{summary}</span>}
          <span className="ml-auto">{action}</span>
        </button>
      ) : (
        <div className="min-w-0 space-y-3 pb-6">
          <div className="flex flex-wrap items-center gap-2">
            <h2 className="text-base font-medium">{title}</h2>
            {pill && <StatusPill tone={tone} label={pill} spin={spin} testId={`tunnel-step-${n}-pill`} />}
            {collapsible && done && (defaultFolded || open) && (
              <button
                type="button"
                data-testid={`tunnel-step-${n}-fold`}
                aria-expanded={true}
                onClick={() => setOpen(false)}
                className="ml-auto inline-flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground"
              >
                <ChevronUp className="h-3 w-3" aria-hidden="true" /> Collapse
              </button>
            )}
          </div>
          {status && <div className="text-sm text-muted-foreground">{status}</div>}
          {children}
        </div>
      )}
    </section>
  );
}

/** Folded steps offer Edit; while remote access is on, changing them is refused. */
export function EditAction({ locked, onLocked }: { locked: boolean; onLocked: () => void }) {
  if (locked) {
    return (
      <span
        data-testid="tunnel-step-locked"
        className="inline-flex items-center gap-1 text-xs text-muted-foreground"
        onClick={(event) => {
          event.stopPropagation();
          onLocked();
        }}
      >
        <Lock className="h-3 w-3" aria-hidden="true" /> Locked
      </span>
    );
  }
  return (
    <span className="inline-flex items-center gap-1 text-xs text-muted-foreground">
      <Pencil className="h-3 w-3" aria-hidden="true" /> Edit
    </span>
  );
}

export function PathField({
  id,
  label,
  value,
  onChange,
  detected,
  placeholder,
  foundHint,
  missingHint,
  error,
  disabled,
}: {
  id: string;
  label: string;
  value: string;
  onChange: (value: string) => void;
  detected?: string | null;
  placeholder: string;
  foundHint: ReactNode;
  missingHint: ReactNode;
  error?: string | null;
  disabled?: boolean;
}) {
  return (
    <div className="space-y-2">
      <label className="text-sm font-medium" htmlFor={id}>
        {label} <span className="font-normal text-muted-foreground">· optional</span>
      </label>
      <Input
        id={id}
        data-testid={id}
        autoComplete="off"
        spellCheck={false}
        className="font-mono text-sm"
        placeholder={detected ?? placeholder}
        value={value}
        disabled={disabled}
        onChange={(event) => onChange(event.target.value)}
      />
      <p className="text-xs text-muted-foreground">{detected ? foundHint : missingHint}</p>
      {error && (
        <p data-testid={`${id}-error`} className="flex items-center gap-1.5 text-sm text-destructive">
          <AlertOctagon className="h-3.5 w-3.5 shrink-0" aria-hidden="true" /> {error}
        </p>
      )}
    </div>
  );
}

export function CommandLine({ children, onCopy }: { children: string; onCopy: (value: string) => void }) {
  return (
    <div className="flex items-center gap-2 rounded-md border bg-muted/50 p-2">
      <code data-testid="tunnel-login-command" className="min-w-0 flex-1 break-all font-mono text-xs">
        {children}
      </code>
      <button
        type="button"
        onClick={() => onCopy(children)}
        aria-label={`Copy command ${children}`}
        className="inline-flex shrink-0 items-center gap-1 rounded px-1.5 py-1 text-xs text-muted-foreground hover:text-foreground"
      >
        <Copy className="h-3 w-3" aria-hidden="true" /> Copy
      </button>
    </div>
  );
}

const STAGES = ['Starting up', 'Connecting to Cloudflare', 'Connected'];

export function Progress({ at }: { at: number }) {
  return (
    <ol data-testid="tunnel-progress" className="space-y-2 rounded-md border bg-muted/30 p-3">
      {STAGES.map((label, index) => {
        const state = index < at ? 'done' : index === at ? 'now' : 'next';
        return (
          <li
            key={label}
            data-test-state={state}
            className={`flex items-center gap-2 text-sm ${state === 'next' ? 'text-muted-foreground' : ''}`}
          >
            <span className="flex h-4 w-4 items-center justify-center" aria-hidden="true">
              {state === 'done' ? (
                <Check className="h-3.5 w-3.5 text-green-600 dark:text-green-400" />
              ) : state === 'now' ? (
                <RefreshCw className="h-3.5 w-3.5 animate-spin text-blue-600 motion-reduce:animate-none dark:text-blue-300" />
              ) : (
                <span className="h-1.5 w-1.5 rounded-full bg-muted-foreground/40" />
              )}
            </span>
            {label}
          </li>
        );
      })}
    </ol>
  );
}
