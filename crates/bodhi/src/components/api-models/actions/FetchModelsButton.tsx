'use client';

import { Loader2, Download, CheckCircle, AlertCircle } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { cn } from '@/lib/utils';

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
  const isDisabled = disabled || !canFetch || isLoading;

  const getStatusIcon = () => {
    switch (status) {
      case 'loading':
        return <Loader2 className="h-4 w-4 animate-spin" />;
      case 'success':
        return <CheckCircle className="h-4 w-4 text-green-600" />;
      case 'error':
        return <AlertCircle className="h-4 w-4 text-red-600" />;
      default:
        return <Download className="h-4 w-4" />;
    }
  };

  const getButtonText = () => {
    switch (status) {
      case 'loading':
        return 'Fetching...';
      case 'success':
        return modelCount > 0 ? `Found ${modelCount} Models` : 'Models Fetched';
      case 'error':
        return 'Fetch Failed';
      default:
        return 'Fetch Models';
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
      onClick={onFetch}
      disabled={isDisabled}
      data-testid={testId}
      className={cn(
        variant === 'link' ? 'h-auto p-0' : 'min-w-[120px]',
        status !== 'idle' && variant !== 'link' && getStatusColor(),
        className
      )}
    >
      <span className="mr-2">{getStatusIcon()}</span>
      {getButtonText()}
    </Button>
  );

  if (!canFetch && !isLoading && disabledReason) {
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
