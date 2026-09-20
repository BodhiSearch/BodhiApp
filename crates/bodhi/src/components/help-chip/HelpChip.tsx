import { cloneElement, forwardRef, isValidElement, type ReactElement, type ReactNode } from 'react';

import { CircleHelp } from 'lucide-react';

import { Button, type ButtonProps } from '@/components/ui/button';

export interface HelpChipProps extends Omit<ButtonProps, 'variant' | 'size'> {
  icon?: ReactNode;
}

export const HelpChip = forwardRef<HTMLButtonElement, HelpChipProps>(function HelpChip(
  { icon, children, asChild, type, ...props },
  ref
) {
  const glyph = icon ?? <CircleHelp aria-hidden="true" />;

  if (asChild && isValidElement(children)) {
    const child = children as ReactElement<{ children?: ReactNode }>;
    return (
      <Button ref={ref} asChild variant="help" size="chip" {...props}>
        {cloneElement(child, undefined, glyph, child.props.children)}
      </Button>
    );
  }

  return (
    <Button ref={ref} type={type ?? 'button'} variant="help" size="chip" {...props}>
      {glyph}
      {children}
    </Button>
  );
});
