'use client';

import { Button } from '@/components/ui/button';
import { useSetupContext } from './SetupProvider';
import { ChevronLeft, ChevronRight } from 'lucide-react';

interface SetupNavigationProps {
  onBack?: () => void;
  onNext?: () => void;
  onSkip?: () => void;
  backLabel?: string;
  nextLabel?: string;
  skipLabel?: string;
  showBack?: boolean;
  showNext?: boolean;
  showSkip?: boolean;
  nextDisabled?: boolean;
  backDisabled?: boolean;
  className?: string;
}

export function SetupNavigation({
  onBack,
  onNext,
  onSkip,
  backLabel = 'Back',
  nextLabel = 'Continue',
  skipLabel = 'Skip for Now',
  showBack = true,
  showNext = true,
  showSkip = false,
  nextDisabled = false,
  backDisabled = false,
  className = '',
}: SetupNavigationProps) {
  const { isFirstStep } = useSetupContext();

  return (
    <div className={`flex items-center justify-between ${className}`}>
      <div>
        {showBack && !isFirstStep && (
          <Button variant="outline" onClick={onBack} disabled={backDisabled}>
            <ChevronLeft className="mr-2 h-4 w-4" />
            {backLabel}
          </Button>
        )}
      </div>
      <div className="flex gap-4">
        {showSkip && (
          <Button variant="outline" onClick={onSkip}>
            {skipLabel}
          </Button>
        )}
        {showNext && (
          <Button onClick={onNext} disabled={nextDisabled}>
            {nextLabel}
            <ChevronRight className="ml-2 h-4 w-4" />
          </Button>
        )}
      </div>
    </div>
  );
}
