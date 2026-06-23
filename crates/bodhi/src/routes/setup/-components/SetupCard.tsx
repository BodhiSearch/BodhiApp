import { ReactNode } from 'react';

import { motion } from 'framer-motion';

import { itemVariants } from '@/routes/setup/-shared/types';

interface SetupCardProps {
  title?: string | ReactNode;
  description?: string | ReactNode;
  children: ReactNode;
  footer?: ReactNode;
  className?: string;
}

/** Calm wizard card: soft surface, generous padding, centered head. */
export function SetupCard({ title, description, children, footer, className }: SetupCardProps) {
  return (
    <motion.section
      variants={itemVariants}
      className={`setup-card overflow-hidden rounded-[var(--radius-xl)] border border-[hsl(var(--border-strong))] bg-card shadow-sm ${className ?? ''}`}
    >
      <div className="px-9 py-8">
        {title && (
          <header className="mb-6 text-center">
            {typeof title === 'string' ? (
              <h2 className="text-[26px] font-bold leading-tight tracking-tight">{title}</h2>
            ) : (
              title
            )}
            {description && (
              <div className="mx-auto mt-2 max-w-[50ch] text-[14.5px] leading-relaxed text-muted-foreground">
                {description}
              </div>
            )}
          </header>
        )}
        {children}
        {footer && <div className="mt-6">{footer}</div>}
      </div>
    </motion.section>
  );
}
