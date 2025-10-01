'use client';

import { ReactNode } from 'react';
import { motion } from 'framer-motion';
import { SetupProgress } from '../SetupProgress';
import { BodhiLogo } from '../BodhiLogo';
import { useSetupContext } from './SetupProvider';
import { SETUP_STEP_LABELS, SETUP_TOTAL_STEPS } from '../constants';

interface SetupContainerProps {
  children: ReactNode;
  showLogo?: boolean;
  showProgress?: boolean;
}

const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.1,
    },
  },
};

export function SetupContainer({ children, showLogo = true, showProgress = true }: SetupContainerProps) {
  const { currentStep } = useSetupContext();

  return (
    <motion.div
      className="mx-auto max-w-4xl space-y-8 p-4 md:p-8"
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
  );
}
