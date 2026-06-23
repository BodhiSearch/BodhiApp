import { Download } from 'lucide-react';

import { StatusButton } from './StatusButton';

interface FetchModelsButtonProps {
  onFetch: () => void;
  canFetch: boolean;
  isLoading: boolean;
  status: 'idle' | 'loading' | 'success' | 'error';
  disabled?: boolean;
  disabledReason?: string;
  modelCount?: number;
  variant?: 'default' | 'outline' | 'secondary' | 'link';
  size?: 'default' | 'sm' | 'lg';
  className?: string;
  'data-testid'?: string;
}

export function FetchModelsButton({
  onFetch,
  canFetch,
  isLoading,
  status,
  disabled = false,
  disabledReason,
  modelCount = 0,
  variant = 'outline',
  size = 'default',
  className = '',
  'data-testid': testId = 'fetch-models-button',
}: FetchModelsButtonProps) {
  return (
    <StatusButton
      status={status === 'loading' ? 'busy' : status}
      isLoading={isLoading}
      enabled={canFetch}
      onClick={onFetch}
      idleIcon={<Download className="h-4 w-4" />}
      idleText="Fetch Models"
      busyText="Fetching..."
      successText={modelCount > 0 ? `Found ${modelCount} Models` : 'Models Fetched'}
      errorText="Fetch Failed"
      disabled={disabled}
      disabledReason={disabledReason}
      variant={variant}
      size={size}
      className={className}
      baseClassName={variant === 'link' ? 'h-auto p-0' : 'min-w-[120px]'}
      applyStatusColor={variant !== 'link'}
      data-testid={testId}
    />
  );
}
