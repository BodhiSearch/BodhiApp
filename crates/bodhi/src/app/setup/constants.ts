/**
 * Setup flow constants for the 7-step onboarding process
 */

export const SETUP_STEPS = {
  WELCOME: 1,
  RESOURCE_ADMIN: 2,
  DOWNLOAD_MODELS: 3,
  API_MODELS: 4,
  TOOLS: 5,
  BROWSER_EXTENSION: 6,
  COMPLETE: 7,
} as const;

export const SETUP_STEP_LABELS = [
  'Get Started',
  'Login & Setup',
  'Local Models',
  'API Models',
  'Tools',
  'Extension',
  'All Done!',
];

export const SETUP_TOTAL_STEPS = 7;

export type SetupStep = (typeof SETUP_STEPS)[keyof typeof SETUP_STEPS];
