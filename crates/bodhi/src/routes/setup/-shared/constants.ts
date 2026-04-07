/**
 * Setup flow constants for the 6-step onboarding process
 */

export const SETUP_STEPS = {
  WELCOME: 1,
  RESOURCE_ADMIN: 2,
  DOWNLOAD_MODELS: 3,
  API_MODELS: 4,
  BROWSER_EXTENSION: 5,
  COMPLETE: 6,
} as const;

export const SETUP_STEP_LABELS = [
  'Get Started',
  'Login & Setup',
  'Local Models',
  'API Models',
  'Extension',
  'All Done!',
];

export const SETUP_TOTAL_STEPS = 6;

export type SetupStep = (typeof SETUP_STEPS)[keyof typeof SETUP_STEPS];
