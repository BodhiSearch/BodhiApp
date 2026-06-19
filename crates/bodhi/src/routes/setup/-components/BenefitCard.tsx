interface BenefitCardProps {
  title: string;
  description: string;
  icon: string;
  isNew?: boolean;
}

export function BenefitCard({ title, description, icon, isNew }: BenefitCardProps) {
  const testId = `benefit-card-${title.toLowerCase().replace(/\s+/g, '-')}`;

  return (
    <article
      data-testid={testId}
      className="relative h-full rounded-[var(--radius-lg)] border border-border bg-card p-5 transition-all duration-200 hover:-translate-y-0.5 hover:border-primary/45 hover:shadow-md"
    >
      {isNew && (
        <span className="absolute right-4 top-4 rounded-full bg-[hsl(var(--accent)/0.12)] px-2 py-0.5 text-[9.5px] font-bold uppercase tracking-wider text-[hsl(var(--accent))]">
          NEW
        </span>
      )}
      <div className="mb-3.5 flex h-[38px] w-[38px] items-center justify-center rounded-[var(--radius-md)] bg-primary/[0.14] text-xl text-[hsl(var(--primary-hover))]">
        {icon}
      </div>
      <h3 className="mb-1.5 text-base font-semibold tracking-tight">{title}</h3>
      <p className="text-[13px] leading-relaxed text-muted-foreground">{description}</p>
    </article>
  );
}
