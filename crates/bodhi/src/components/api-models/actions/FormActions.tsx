'use client';

import { Button } from '@/components/ui/button';
import { Loader2 } from 'lucide-react';
import { TestConnectionButton } from './TestConnectionButton';
import { ReactNode } from 'react';

interface FormActionsProps {
  // Primary action (submit)
  primaryAction: {
    label: string;
    onClick?: () => void;
    type?: 'submit' | 'button';
    disabled?: boolean;
    loading?: boolean;
    'data-testid'?: string;
  };

  // Secondary action (cancel/skip)
  secondaryAction?: {
    label: string;
    onClick: () => void;
    variant?: 'outline' | 'ghost' | 'secondary';
    'data-testid'?: string;
  };

  // Test connection integration
  testConnection?: {
    onTest: () => void;
    canTest: boolean;
    isLoading: boolean;
    status: 'idle' | 'testing' | 'success' | 'error';
    disabledReason?: string;
  };

  // Layout and styling
  layout?: 'space-between' | 'end' | 'center';
  className?: string;
  children?: ReactNode;
}

export function FormActions({
  primaryAction,
  secondaryAction,
  testConnection,
  layout = 'space-between',
  className = '',
  children,
}: FormActionsProps) {
  const primaryButton = (
    <Button
      type={primaryAction.type || 'submit'}
      onClick={primaryAction.onClick}
      disabled={primaryAction.disabled || primaryAction.loading}
      data-testid={primaryAction['data-testid'] || 'primary-action-button'}
    >
      {primaryAction.loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
      {primaryAction.label}
    </Button>
  );

  const secondaryButton = secondaryAction && (
    <Button
      type="button"
      variant={secondaryAction.variant || 'outline'}
      onClick={secondaryAction.onClick}
      data-testid={secondaryAction['data-testid'] || 'secondary-action-button'}
    >
      {secondaryAction.label}
    </Button>
  );

  const testButton = testConnection && (
    <TestConnectionButton
      onTest={testConnection.onTest}
      canTest={testConnection.canTest}
      isLoading={testConnection.isLoading}
      status={testConnection.status}
      disabledReason={testConnection.disabledReason}
    />
  );

  const getLayoutClasses = () => {
    switch (layout) {
      case 'end':
        return 'flex justify-end gap-2';
      case 'center':
        return 'flex justify-center gap-2';
      default:
        return 'flex justify-between items-center';
    }
  };

  return (
    <div className={`${getLayoutClasses()} ${className}`}>
      {/* Left side - Test connection or custom children */}
      <div className="flex items-center gap-2">
        {testButton}
        {children}
      </div>

      {/* Right side - Primary/Secondary actions */}
      <div className="flex gap-2">
        {secondaryButton}
        {primaryButton}
      </div>
    </div>
  );
}
