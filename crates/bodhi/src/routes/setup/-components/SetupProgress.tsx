import { Fragment } from 'react';

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
    return {
      isCompleted: stepNumber < currentStep,
      isCurrent: stepNumber === currentStep,
    };
  };

  const getStepDataStatus = (index: number) => {
    const { isCompleted, isCurrent } = getStepStatus(index);
    if (isCompleted) return 'completed';
    if (isCurrent) return 'current';
    return 'pending';
  };

  return (
    <div className="mb-8" data-testid="setup-progress">
      {/* Visually-hidden progressbar carries the ARIA + percent contract. */}
      <span
        className="sr-only"
        data-testid="progress-bar"
        data-progress-percent={progressPercent}
        role="progressbar"
        aria-valuenow={currentStep}
        aria-valuemin={1}
        aria-valuemax={totalSteps}
        aria-label="Setup progress"
      />

      <nav aria-label="Setup progress" className="flex items-start justify-center">
        {Array.from({ length: totalSteps }).map((_, index) => {
          const { isCompleted, isCurrent } = getStepStatus(index);
          const connectorFilled = index <= currentStep - 1;

          return (
            <Fragment key={index}>
              {index > 0 && (
                <span
                  className={`mt-4 h-0.5 w-3 min-w-3 max-w-[52px] flex-1 rounded-sm transition-colors duration-200 ${
                    connectorFilled ? 'bg-primary' : 'bg-border'
                  }`}
                />
              )}

              <div className="flex w-[86px] flex-none flex-col items-center gap-2 text-center">
                <motion.div
                  data-testid={`step-indicator-${index + 1}`}
                  data-completed={isCompleted}
                  data-current={isCurrent}
                  data-status={getStepDataStatus(index)}
                  className={`relative flex h-[34px] w-[34px] items-center justify-center rounded-full border-[1.5px] text-[13px] font-semibold tabular-nums transition-colors duration-200 ${
                    isCompleted || isCurrent
                      ? 'border-primary bg-primary text-primary-foreground'
                      : 'border-border bg-[hsl(var(--surface-3))] text-muted-foreground'
                  } ${isCurrent ? 'setup-node-halo' : ''}`}
                  initial={{ scale: 0 }}
                  animate={{ scale: 1 }}
                  transition={{ delay: index * 0.06 }}
                >
                  {isCompleted ? <Check className="h-4 w-4" strokeWidth={3} /> : <span>{index + 1}</span>}
                </motion.div>

                {stepLabels && !compact && stepLabels[index] !== undefined && (
                  <span
                    data-testid={`step-label-${index + 1}`}
                    title={stepLabels[index]}
                    className={`hidden max-w-full truncate px-1 text-[11.5px] leading-tight sm:block ${
                      isCurrent
                        ? 'font-semibold text-[hsl(var(--primary-hover))]'
                        : isCompleted
                          ? 'text-foreground'
                          : 'text-muted-foreground'
                    }`}
                  >
                    {stepLabels[index]}
                  </span>
                )}
              </div>
            </Fragment>
          );
        })}
      </nav>

      <div className="mt-3 text-center text-[12.5px] text-muted-foreground" data-testid="step-counter">
        <p>
          Step {currentStep} of {totalSteps}
        </p>
        {stepLabels && compact && stepLabels[currentStep - 1] !== undefined && (
          <p className="mt-1 font-medium text-foreground">{stepLabels[currentStep - 1]}</p>
        )}
      </div>
    </div>
  );
}
