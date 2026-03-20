'use client';

import { createContext, useContext, ReactNode } from 'react';

import { usePathname } from 'next/navigation';

import { SETUP_STEPS } from '../constants';

interface SetupContextType {
  currentStep: number;
  isFirstStep: boolean;
  isLastStep: boolean;
  getStepFromPath: (path: string) => number;
}

const SetupContext = createContext<SetupContextType | undefined>(undefined);

export function SetupProvider({ children }: { children: ReactNode }) {
  const pathname = usePathname();

  const getStepFromPath = (path: string): number => {
    if (path.includes('/setup/resource-admin')) return SETUP_STEPS.RESOURCE_ADMIN;
    if (path.includes('/setup/download-models')) return SETUP_STEPS.DOWNLOAD_MODELS;
    if (path.includes('/setup/api-models')) return SETUP_STEPS.API_MODELS;
    if (path.includes('/setup/tools')) return SETUP_STEPS.TOOLS;
    if (path.includes('/setup/browser-extension')) return SETUP_STEPS.BROWSER_EXTENSION;
    if (path.includes('/setup/complete')) return SETUP_STEPS.COMPLETE;
    return SETUP_STEPS.WELCOME;
  };

  const currentStep = getStepFromPath(pathname);
  const isFirstStep = currentStep === SETUP_STEPS.WELCOME;
  const isLastStep = currentStep === SETUP_STEPS.COMPLETE;

  return (
    <SetupContext.Provider value={{ currentStep, isFirstStep, isLastStep, getStepFromPath }}>
      {children}
    </SetupContext.Provider>
  );
}

export function useSetupContext() {
  const context = useContext(SetupContext);
  if (!context) {
    throw new Error('useSetupContext must be used within SetupProvider');
  }
  return context;
}
