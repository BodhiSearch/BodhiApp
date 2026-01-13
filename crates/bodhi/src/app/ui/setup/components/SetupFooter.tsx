'use client';

import { motion } from 'framer-motion';

import { itemVariants } from '@/app/ui/setup/types';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';

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
  buttonVariant = 'outline',
  buttonTestId = 'continue-button',
}: SetupFooterProps) {
  return (
    <motion.div variants={itemVariants} className="space-y-4">
      {/* Clarification Card */}
      <Card className="bg-muted/30">
        <CardContent className="py-6">
          <div className="text-center space-y-2">
            <p className="text-sm text-muted-foreground">{clarificationText}</p>
            {subText && <p className="text-xs text-muted-foreground">{subText}</p>}
          </div>
        </CardContent>
      </Card>

      {/* Continue Button */}
      <div className="flex justify-end">
        <Button data-testid={buttonTestId} variant={buttonVariant} onClick={onContinue}>
          {buttonLabel}
        </Button>
      </div>
    </motion.div>
  );
}
