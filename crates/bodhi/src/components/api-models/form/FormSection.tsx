import { ReactNode } from 'react';

/** Red `*` required marker — the standard, low-verbosity indicator (replaces the
 *  old "Required" badge) used across form field labels. */
export function RequiredMark() {
  return (
    <span className="text-destructive" aria-hidden="true">
      *
    </span>
  );
}

interface FormSectionProps {
  title: string;
  description?: string;
  children: ReactNode;
}

/** A labeled form section: an uppercase section heading over its fields, giving the
 *  form visible structure (Provider Connection / Request Routing / Model Selection). */
export function FormSection({ title, description, children }: FormSectionProps) {
  return (
    <section className="space-y-4">
      <div>
        <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">{title}</h3>
        {description && <p className="mt-1 text-sm text-muted-foreground">{description}</p>}
      </div>
      {children}
    </section>
  );
}
