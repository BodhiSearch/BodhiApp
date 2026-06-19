import { motion } from 'framer-motion';
import { ArrowRight } from 'lucide-react';

import { Button } from '@/components/ui/button';

import { itemVariants } from '../-shared/types';

interface SetupFooterProps {
  clarificationText: string;
  subText?: string;
  onContinue: () => void;
  buttonLabel?: string;
  buttonVariant?: 'default' | 'outline';
  buttonTestId?: string;
}

export function SetupFooter({
  clarificationText,
  subText,
  onContinue,
  buttonLabel = 'Continue',
  buttonVariant = 'default',
  buttonTestId = 'continue-button',
}: SetupFooterProps) {
  return (
    <motion.div variants={itemVariants} className="mt-6 space-y-5">
      <div className="rounded-[var(--radius-lg)] border border-border bg-muted/40 px-6 py-5 text-center">
        <p className="text-sm text-muted-foreground">{clarificationText}</p>
        {subText && <p className="mt-1 text-xs text-muted-foreground">{subText}</p>}
      </div>

      <div className="flex items-center justify-end">
        <Button data-testid={buttonTestId} variant={buttonVariant} size="lg" onClick={onContinue} className="gap-2">
          {buttonLabel}
          <ArrowRight className="h-4 w-4" />
        </Button>
      </div>
    </motion.div>
  );
}
