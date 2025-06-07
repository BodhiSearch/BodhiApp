import { motion } from 'framer-motion';
import { Check } from 'lucide-react';

interface SetupProgressProps {
  currentStep: number;
  totalSteps: number;
}

export function SetupProgress({ currentStep, totalSteps }: SetupProgressProps) {
  return (
    <div className="sticky top-0 z-10 bg-background/80 backdrop-blur-sm p-4">
      <div className="mx-auto max-w-2xl">
        {/* Progress bar */}
        <div className="relative">
          <div className="absolute left-0 top-1/2 h-1 w-full -translate-y-1/2 bg-muted">
            <motion.div
              className="h-full bg-primary"
              initial={{ width: 0 }}
              animate={{ width: `${(currentStep / totalSteps) * 100}%` }}
              transition={{ duration: 0.5 }}
            />
          </div>

          {/* Step indicators */}
          <div className="relative flex justify-between">
            {Array.from({ length: totalSteps }).map((_, index) => {
              const isCompleted = index < currentStep - 1;
              const isCurrent = index === currentStep - 1;

              return (
                <motion.div
                  key={index}
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
                    <span
                      className={`text-sm ${
                        isCurrent
                          ? 'text-primary-foreground'
                          : 'text-muted-foreground'
                      }`}
                    >
                      {index + 1}
                    </span>
                  )}
                </motion.div>
              );
            })}
          </div>
        </div>

        {/* Step label */}
        <p className="mt-2 text-center text-sm text-muted-foreground">
          Step {currentStep} of {totalSteps}
        </p>
      </div>
    </div>
  );
}
