'use client';

import { motion } from 'framer-motion';
import { ExternalLink, Check } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { cn } from '@/lib/utils';

import { ApiProvider } from './constants';

interface ProviderInfoProps {
  provider: ApiProvider;
  isSelected: boolean;
  onSelect: (providerId: string) => void;
  variant?: 'default' | 'compact';
  showCategory?: boolean;
}

export function ProviderInfo({
  provider,
  isSelected,
  onSelect,
  variant = 'default',
  showCategory = false,
}: ProviderInfoProps) {
  return (
    <motion.div
      whileHover={{ scale: variant === 'compact' ? 1.01 : 1.02 }}
      whileTap={{ scale: 0.98 }}
      transition={{ type: 'spring', stiffness: 300 }}
      data-testid={`provider-card-${provider.id}`}
      onClick={() => onSelect(provider.id)}
      className="cursor-pointer"
    >
      <Card
        className={cn(
          'transition-all duration-200',
          isSelected
            ? 'border-primary bg-primary/5 ring-2 ring-primary ring-offset-2'
            : 'hover:border-border hover:shadow-md',
          variant === 'compact' && 'h-auto'
        )}
      >
        <CardHeader className={cn(variant === 'compact' ? 'pb-2' : 'pb-3')}>
          <CardTitle className={cn('flex items-center justify-between', variant === 'compact' ? 'text-lg' : 'text-xl')}>
            <div className="flex items-center gap-3">
              <span className="text-2xl" data-testid={`provider-icon-${provider.id}`}>
                {provider.icon}
              </span>
              <div className="flex flex-col">
                <span data-testid={`provider-name-${provider.id}`}>{provider.name}</span>
                {showCategory && <span className="text-xs text-muted-foreground capitalize">{provider.category}</span>}
              </div>
            </div>
            {isSelected && (
              <div className="flex items-center justify-center w-6 h-6 bg-primary text-primary-foreground rounded-full">
                <Check className="w-4 h-4" data-testid={`provider-selected-${provider.id}`} />
              </div>
            )}
          </CardTitle>
        </CardHeader>
        <CardContent className={cn('space-y-4', variant === 'compact' && 'space-y-2 pt-0')}>
          <p className="text-sm text-muted-foreground" data-testid={`provider-description-${provider.id}`}>
            {provider.description}
          </p>

          {provider.baseUrl && (
            <div className="text-xs text-muted-foreground">
              <span className="font-medium">Endpoint:</span>{' '}
              <code className="bg-muted px-1 py-0.5 rounded text-xs">{provider.baseUrl}</code>
            </div>
          )}

          {provider.commonModels.length > 0 && variant !== 'compact' && (
            <div className="text-xs text-muted-foreground">
              <span className="font-medium">Common Models:</span>{' '}
              <span data-testid={`provider-models-${provider.id}`}>
                {provider.commonModels.slice(0, 3).join(', ')}
                {provider.commonModels.length > 3 && ` +${provider.commonModels.length - 3} more`}
              </span>
            </div>
          )}

          {provider.docUrl && (
            <Button
              variant="ghost"
              size="sm"
              className="w-full justify-start p-0 h-auto text-xs text-primary hover:text-primary/80"
              asChild
              onClick={(e) => e.stopPropagation()}
              data-testid={`provider-docs-${provider.id}`}
            >
              <a href={provider.docUrl} target="_blank" rel="noopener noreferrer" className="flex items-center gap-1">
                <ExternalLink className="w-3 h-3" />
                View Documentation
              </a>
            </Button>
          )}
        </CardContent>
      </Card>
    </motion.div>
  );
}
