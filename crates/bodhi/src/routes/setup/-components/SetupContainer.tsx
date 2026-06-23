import { ReactNode } from 'react';

import { motion } from 'framer-motion';

import { SETUP_STEP_LABELS, SETUP_TOTAL_STEPS } from '@/routes/setup/-shared/constants';
import { containerVariants } from '@/routes/setup/-shared/types';

import './setup-wizard.css';
import { BodhiLogo } from './BodhiLogo';
import { SetupProgress } from './SetupProgress';
import { useSetupContext } from './SetupProvider';
import { SetupThemeToggle } from './SetupThemeToggle';

interface SetupContainerProps {
  children: ReactNode;
  showLogo?: boolean;
  showProgress?: boolean;
  /** Wide column for the model-recommendation grid (step 3). */
  wide?: boolean;
  'data-testid'?: string;
}

export function SetupContainer({
  children,
  showLogo = true,
  showProgress = true,
  wide = false,
  'data-testid': dataTestId,
}: SetupContainerProps) {
  const { currentStep } = useSetupContext();

  return (
    <div className="setup-wash">
      <SetupThemeToggle />
      <motion.div
        data-testid={dataTestId}
        className={`relative mx-auto w-full px-6 pb-24 pt-12 ${wide ? 'max-w-4xl' : 'max-w-2xl'}`}
        variants={containerVariants}
        initial="hidden"
        animate="visible"
      >
        {showLogo && <BodhiLogo />}
        {showProgress && (
          <SetupProgress currentStep={currentStep} totalSteps={SETUP_TOTAL_STEPS} stepLabels={SETUP_STEP_LABELS} />
        )}
        {children}
      </motion.div>
    </div>
  );
}
