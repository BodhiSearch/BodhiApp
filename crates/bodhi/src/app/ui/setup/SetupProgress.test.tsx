import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { SetupProgress } from './SetupProgress';
import { SETUP_STEP_LABELS } from './constants';

describe('SetupProgress', () => {
  describe('backward compatibility with 4 steps', () => {
    it('renders with 4 steps for existing flow', () => {
      render(<SetupProgress currentStep={2} totalSteps={4} />);
      const steps = screen.getAllByTestId(/^step-indicator-/);
      expect(steps).toHaveLength(4);
    });

    it('shows correct progress percentage with 4 steps', () => {
      render(<SetupProgress currentStep={2} totalSteps={4} />);
      const progressBar = screen.getByTestId('progress-bar');
      expect(progressBar).toHaveAttribute('data-progress-percent', '50');
    });

    it('displays step count text for 4 steps', () => {
      render(<SetupProgress currentStep={2} totalSteps={4} />);
      expect(screen.getByText('Step 2 of 4')).toBeInTheDocument();
    });
  });

  describe('new 6-step flow', () => {
    it('renders with 6 steps for new flow', () => {
      render(<SetupProgress currentStep={1} totalSteps={6} />);
      const steps = screen.getAllByTestId(/^step-indicator-/);
      expect(steps).toHaveLength(6);
    });

    it('shows correct progress percentage with 6 steps', () => {
      render(<SetupProgress currentStep={3} totalSteps={6} />);
      const progressBar = screen.getByTestId('progress-bar');
      expect(progressBar).toHaveAttribute('data-progress-percent', '50');
    });

    it('displays step count text for 6 steps', () => {
      render(<SetupProgress currentStep={2} totalSteps={6} />);
      expect(screen.getByText('Step 2 of 6')).toBeInTheDocument();
    });
  });

  describe('step indicators', () => {
    it('highlights current step', () => {
      render(<SetupProgress currentStep={3} totalSteps={6} />);
      const currentStep = screen.getByTestId('step-indicator-3');
      expect(currentStep).toHaveAttribute('data-current', 'true');
    });

    it('shows checkmarks for completed steps', () => {
      render(<SetupProgress currentStep={3} totalSteps={6} />);
      const completedStep1 = screen.getByTestId('step-indicator-1');
      const completedStep2 = screen.getByTestId('step-indicator-2');
      expect(completedStep1).toHaveAttribute('data-completed', 'true');
      expect(completedStep2).toHaveAttribute('data-completed', 'true');
    });

    it('shows upcoming steps as incomplete', () => {
      render(<SetupProgress currentStep={2} totalSteps={6} />);
      const upcomingStep = screen.getByTestId('step-indicator-4');
      expect(upcomingStep).toHaveAttribute('data-completed', 'false');
      expect(upcomingStep).toHaveAttribute('data-current', 'false');
    });

    it('shows step numbers for incomplete steps', () => {
      render(<SetupProgress currentStep={2} totalSteps={6} />);
      const currentStep = screen.getByTestId('step-indicator-2');
      expect(currentStep).toHaveTextContent('2');
    });

    it('shows checkmark icons for completed steps', () => {
      render(<SetupProgress currentStep={3} totalSteps={6} />);
      const completedStep = screen.getByTestId('step-indicator-1');
      const checkIcon = completedStep.querySelector('svg');
      expect(checkIcon).toBeInTheDocument();
    });
  });

  describe('step labels', () => {
    it('displays step labels when provided', () => {
      render(<SetupProgress currentStep={1} totalSteps={6} stepLabels={SETUP_STEP_LABELS} />);
      expect(screen.getByText('Get Started')).toBeInTheDocument();
      expect(screen.getByText('Login & Setup')).toBeInTheDocument();
      expect(screen.getByText('Local Models')).toBeInTheDocument();
      expect(screen.getByText('API Models')).toBeInTheDocument();
      expect(screen.getByText('Extension')).toBeInTheDocument();
      expect(screen.getByText('All Done!')).toBeInTheDocument();
    });

    it('works without step labels', () => {
      render(<SetupProgress currentStep={1} totalSteps={6} />);
      const steps = screen.getAllByTestId(/^step-indicator-/);
      expect(steps).toHaveLength(6);
      expect(screen.queryByText('Get Started')).not.toBeInTheDocument();
    });

    it('handles mismatched label count gracefully', () => {
      const shortLabels = ['Step 1', 'Step 2'];
      render(<SetupProgress currentStep={1} totalSteps={6} stepLabels={shortLabels} />);
      expect(screen.getByText('Step 1')).toBeInTheDocument();
      expect(screen.getByText('Step 2')).toBeInTheDocument();
      expect(screen.queryByText('Step 3')).not.toBeInTheDocument();
    });
  });

  describe('skipped steps', () => {
    it('shows skipped indicator for specified steps', () => {
      render(<SetupProgress currentStep={5} totalSteps={6} skippedSteps={[4]} />);
      const skippedStep = screen.getByTestId('step-indicator-4');
      expect(skippedStep).toHaveAttribute('data-skipped', 'true');
    });

    it('shows correct progress with skipped steps', () => {
      render(<SetupProgress currentStep={5} totalSteps={6} skippedSteps={[4]} />);
      const progressBar = screen.getByTestId('progress-bar');
      // Progress should be (5/6) * 100 = 83.33%
      expect(progressBar).toHaveAttribute('data-progress-percent', '83.33333333333334');
    });

    it('handles multiple skipped steps', () => {
      render(<SetupProgress currentStep={6} totalSteps={6} skippedSteps={[4, 5]} />);
      const skippedStep1 = screen.getByTestId('step-indicator-4');
      const skippedStep2 = screen.getByTestId('step-indicator-5');
      expect(skippedStep1).toHaveAttribute('data-skipped', 'true');
      expect(skippedStep2).toHaveAttribute('data-skipped', 'true');
    });

    it('works without skipped steps array', () => {
      render(<SetupProgress currentStep={3} totalSteps={6} />);
      const step4 = screen.getByTestId('step-indicator-4');
      expect(step4).toHaveAttribute('data-skipped', 'false');
    });
  });

  describe('accessibility', () => {
    it('has proper ARIA attributes for progress bar', () => {
      render(<SetupProgress currentStep={2} totalSteps={6} />);
      const progressBar = screen.getByRole('progressbar');
      expect(progressBar).toHaveAttribute('aria-valuenow', '2');
      expect(progressBar).toHaveAttribute('aria-valuemin', '1');
      expect(progressBar).toHaveAttribute('aria-valuemax', '6');
      expect(progressBar).toHaveAttribute('aria-label', 'Setup progress');
    });

    it('has proper ARIA labels on step indicators', () => {
      render(<SetupProgress currentStep={2} totalSteps={6} stepLabels={SETUP_STEP_LABELS} />);
      const completedStep = screen.getByTestId('step-indicator-1');
      const currentStep = screen.getByTestId('step-indicator-2');
      const upcomingStep = screen.getByTestId('step-indicator-3');

      expect(completedStep).toHaveAttribute('aria-label', 'Step 1: Get Started, completed');
      expect(currentStep).toHaveAttribute('aria-label', 'Step 2: Login & Setup, current');
      expect(upcomingStep).toHaveAttribute('aria-label', 'Step 3: Local Models, upcoming');
    });

    it('provides aria labels without step labels', () => {
      render(<SetupProgress currentStep={2} totalSteps={6} />);
      const currentStep = screen.getByTestId('step-indicator-2');
      expect(currentStep).toHaveAttribute('aria-label', 'Step 2, current');
    });
  });

  describe('edge cases', () => {
    it('handles step 1 correctly (no completed steps)', () => {
      render(<SetupProgress currentStep={1} totalSteps={6} />);
      const firstStep = screen.getByTestId('step-indicator-1');
      expect(firstStep).toHaveAttribute('data-current', 'true');
      expect(firstStep).toHaveAttribute('data-completed', 'false');
    });

    it('handles last step correctly (all previous completed)', () => {
      render(<SetupProgress currentStep={6} totalSteps={6} />);
      const lastStep = screen.getByTestId('step-indicator-6');
      const secondToLastStep = screen.getByTestId('step-indicator-5');

      expect(lastStep).toHaveAttribute('data-current', 'true');
      expect(secondToLastStep).toHaveAttribute('data-completed', 'true');
    });

    it('handles invalid current step (too high)', () => {
      render(<SetupProgress currentStep={10} totalSteps={6} />);
      const progressBar = screen.getByTestId('progress-bar');
      const progressPercent = parseFloat(progressBar.getAttribute('data-progress-percent') || '0');
      expect(progressPercent).toBeCloseTo(166.67, 1);
    });

    it('handles zero or negative steps gracefully', () => {
      render(<SetupProgress currentStep={0} totalSteps={6} />);
      const progressBar = screen.getByTestId('progress-bar');
      expect(progressBar).toHaveAttribute('data-progress-percent', '0');
    });
  });
});
