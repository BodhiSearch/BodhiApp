import { TestTube } from 'lucide-react';

import { StatusButton } from './StatusButton';

interface TestConnectionButtonProps {
  onTest: () => void;
  canTest: boolean;
  isLoading: boolean;
  status: 'idle' | 'testing' | 'success' | 'error';
  disabled?: boolean;
  disabledReason?: string;
  variant?: 'default' | 'outline' | 'secondary';
  size?: 'default' | 'sm' | 'lg';
  className?: string;
  'data-testid'?: string;
}

export function TestConnectionButton({
  onTest,
  canTest,
  isLoading,
  status,
  disabled = false,
  disabledReason,
  variant = 'outline',
  size = 'default',
  className = '',
  'data-testid': testId = 'test-connection-button',
}: TestConnectionButtonProps) {
  return (
    <StatusButton
      status={status === 'testing' ? 'busy' : status}
      isLoading={isLoading}
      enabled={canTest}
      onClick={onTest}
      idleIcon={<TestTube className="h-4 w-4" />}
      idleText="Test Connection"
      busyText="Testing..."
      successText="Connection Successful"
      errorText="Connection Failed"
      disabled={disabled}
      disabledReason={disabledReason}
      variant={variant}
      size={size}
      className={className}
      baseClassName="min-w-[140px]"
      dataStatus={status}
      data-testid={testId}
    />
  );
}
