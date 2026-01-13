'use client';

import { TestTube, Loader2, CheckCircle, AlertCircle } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { cn } from '@/lib/utils';

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
  const isDisabled = disabled || !canTest || isLoading;

  const getStatusIcon = () => {
    switch (status) {
      case 'testing':
        return <Loader2 className="h-4 w-4 animate-spin" />;
      case 'success':
        return <CheckCircle className="h-4 w-4 text-green-600" />;
      case 'error':
        return <AlertCircle className="h-4 w-4 text-red-600" />;
      default:
        return <TestTube className="h-4 w-4" />;
    }
  };

  const getButtonText = () => {
    switch (status) {
      case 'testing':
        return 'Testing...';
      case 'success':
        return 'Connection Successful';
      case 'error':
        return 'Connection Failed';
      default:
        return 'Test Connection';
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
      onClick={onTest}
      disabled={isDisabled}
      data-testid={testId}
      className={cn('min-w-[140px]', status !== 'idle' && getStatusColor(), className)}
    >
      <span className="mr-2">{getStatusIcon()}</span>
      {getButtonText()}
    </Button>
  );

  if (!canTest && !isLoading && disabledReason) {
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
