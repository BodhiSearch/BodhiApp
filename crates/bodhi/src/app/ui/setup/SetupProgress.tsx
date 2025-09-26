'use client';

import { motion } from 'framer-motion';
import { Check } from 'lucide-react';

interface SetupProgressProps {
  currentStep: number;
  totalSteps: number;
  stepLabels?: string[];
  compact?: boolean;
}

export function SetupProgress({ currentStep, totalSteps, stepLabels, compact = false }: SetupProgressProps) {
  const progressPercent = (currentStep / totalSteps) * 100;

  const getStepStatus = (index: number) => {
    const stepNumber = index + 1;
    const isCompleted = stepNumber < currentStep;
    const isCurrent = stepNumber === currentStep;
    const isPending = stepNumber > currentStep;

    return { isCompleted, isCurrent, isPending };
  };

  const getStepDataStatus = (index: number) => {
    const { isCompleted, isCurrent, isPending } = getStepStatus(index);

    if (isCompleted) return 'completed';
    if (isCurrent) return 'current';
    if (isPending) return 'pending';
    return 'pending';
  };

  return (
    <div className="sticky top-0 z-10 bg-background/80 backdrop-blur-sm p-4" data-testid="setup-progress">
      <div className="mx-auto max-w-2xl">
        {/* Progress bar */}
        <div className="relative">
          <div className="absolute left-0 top-1/2 h-1 w-full -translate-y-1/2 bg-muted">
            <motion.div
              className="h-full bg-primary"
              initial={{ width: 0 }}
              animate={{ width: `${progressPercent}%` }}
              transition={{ duration: 0.5 }}
              style={{ width: `${progressPercent}%` }}
              data-testid="progress-bar"
              data-progress-percent={progressPercent}
              role="progressbar"
              aria-valuenow={currentStep}
              aria-valuemin={1}
              aria-valuemax={totalSteps}
              aria-label="Setup progress"
            />
          </div>

          {/* Step indicators */}
          <div className="relative flex justify-between">
            {Array.from({ length: totalSteps }).map((_, index) => {
              const { isCompleted, isCurrent } = getStepStatus(index);

              return (
                <motion.div
                  key={index}
                  data-testid={`step-indicator-${index + 1}`}
                  data-completed={isCompleted}
                  data-current={isCurrent}
                  data-status={getStepDataStatus(index)}
                  className={`flex h-8 w-8 items-center justify-center rounded-full ${
                    isCompleted || isCurrent ? 'bg-primary' : 'bg-muted'
                  }`}
                  initial={{ scale: 0 }}
                  animate={{ scale: 1 }}
                  transition={{ delay: index * 0.1 }}
                >
                  {isCompleted ? (
                    <Check className="h-4 w-4 text-primary-foreground" />
                  ) : (
                    <span className={`text-sm ${isCurrent ? 'text-primary-foreground' : 'text-muted-foreground'}`}>
                      {index + 1}
                    </span>
                  )}
                </motion.div>
              );
            })}
          </div>
        </div>

        {/* Step labels (if provided and not compact) */}
        {stepLabels && !compact && (
          <div className="relative mt-3 hidden sm:block">
            <div className="flex justify-between">
              {stepLabels.map((label, index) => {
                if (index >= totalSteps) return null;
                const { isCurrent } = getStepStatus(index);

                return (
                  <div key={index} className="flex-1 text-center" style={{ maxWidth: `${100 / totalSteps}%` }}>
                    <span
                      data-testid={`step-label-${index + 1}`}
                      className={`text-xs truncate ${isCurrent ? 'text-primary font-medium' : 'text-muted-foreground'}`}
                      title={label}
                    >
                      {label}
                    </span>
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {/* Step counter */}
        <div className="mt-2 text-center text-sm text-muted-foreground" data-testid="step-counter">
          <p>
            Step {currentStep} of {totalSteps}
          </p>
          {stepLabels && compact && <p className="mt-1 font-medium text-foreground">{stepLabels[currentStep - 1]}</p>}
        </div>
      </div>
    </div>
  );
}
