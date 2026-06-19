import { LucideIcon } from 'lucide-react';

/** Centered rounded icon tile used atop a setup card (steps 2 & 5). */
export function SetupCardIcon({ icon: Icon }: { icon: LucideIcon }) {
  return (
    <div className="mx-auto mb-4 flex h-[38px] w-[38px] items-center justify-center rounded-[var(--radius-md)] bg-primary/[0.14] text-[hsl(var(--primary-hover))]">
      <Icon className="h-[19px] w-[19px]" />
    </div>
  );
}
