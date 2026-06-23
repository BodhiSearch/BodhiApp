import { type ReactNode } from 'react';

import { Loader2, CheckCircle, AlertCircle } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { cn } from '@/lib/utils';

export type StatusButtonStatus = 'idle' | 'busy' | 'success' | 'error';

interface StatusButtonProps {
  status: StatusButtonStatus;
  isLoading: boolean;
  enabled: boolean;
  onClick: () => void;
  idleIcon: ReactNode;
  idleText: ReactNode;
  busyText: ReactNode;
  successText: ReactNode;
  errorText: ReactNode;
  disabled?: boolean;
  disabledReason?: string;
  variant?: 'default' | 'outline' | 'secondary' | 'link';
  size?: 'default' | 'sm' | 'lg';
  className?: string;
  baseClassName?: string;
  applyStatusColor?: boolean;
  dataStatus?: string;
  'data-testid'?: string;
}

export function StatusButton({
  status,
  isLoading,
  enabled,
  onClick,
  idleIcon,
  idleText,
  busyText,
  successText,
  errorText,
  disabled = false,
  disabledReason,
  variant = 'outline',
  size = 'default',
  className = '',
  baseClassName,
  applyStatusColor = true,
  dataStatus,
  'data-testid': testId,
}: StatusButtonProps) {
  const isDisabled = disabled || !enabled || isLoading;

  const getStatusIcon = () => {
    switch (status) {
      case 'busy':
        return <Loader2 className="h-4 w-4 animate-spin" />;
      case 'success':
        return <CheckCircle className="h-4 w-4 text-green-600" />;
      case 'error':
        return <AlertCircle className="h-4 w-4 text-red-600" />;
      default:
        return idleIcon;
    }
  };

  const getButtonText = () => {
    switch (status) {
      case 'busy':
        return busyText;
      case 'success':
        return successText;
      case 'error':
        return errorText;
      default:
        return idleText;
    }
  };

  const getStatusColor = () => {
    switch (status) {
      case 'success':
        return 'text-green-600 border-green-200 bg-green-50 hover:bg-green-100';
      case 'error':
        return 'text-red-600 border-red-200 bg-red-50 hover:bg-red-100';
      default:
        return '';
    }
  };

  const button = (
    <Button
      type="button"
      variant={variant}
      size={size}
      onClick={onClick}
      disabled={isDisabled}
      data-testid={testId}
      data-status={dataStatus}
      className={cn(baseClassName, applyStatusColor && status !== 'idle' && getStatusColor(), className)}
    >
      <span className="mr-2">{getStatusIcon()}</span>
      {getButtonText()}
    </Button>
  );

  if (!enabled && !isLoading && disabledReason) {
    return (
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <span>{button}</span>
          </TooltipTrigger>
          <TooltipContent>
            <p>{disabledReason}</p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    );
  }

  return button;
}
