'use client';

import { motion } from 'framer-motion';

import { ApiProvider, API_PROVIDERS } from './constants';
import { ProviderInfo } from './ProviderInfo';

interface ProviderSelectorProps {
  selectedProviderId?: string | null;
  onProviderSelect: (provider: ApiProvider | null) => void;
  providers?: ApiProvider[];
  variant?: 'default' | 'compact';
  showCategory?: boolean;
  title?: string;
  description?: string;
  className?: string;
}

const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.1,
    },
  },
};

const itemVariants = {
  hidden: { y: 20, opacity: 0 },
  visible: {
    y: 0,
    opacity: 1,
  },
};

export function ProviderSelector({
  selectedProviderId,
  onProviderSelect,
  providers = API_PROVIDERS,
  variant = 'default',
  showCategory = false,
  title = 'Choose Your AI Provider',
  description = 'Select an AI provider to connect to cloud-based models.',
  className = '',
}: ProviderSelectorProps) {
  const handleProviderSelect = (providerId: string) => {
    const provider = providers.find((p) => p.id === providerId);
    onProviderSelect(provider || null);
  };

  return (
    <div className={className} data-testid="provider-selector">
      {title && (
        <div className="mb-4">
          <h3 className="text-lg font-semibold">{title}</h3>
          {description && <p className="text-sm text-muted-foreground mt-1">{description}</p>}
        </div>
      )}

      <motion.div
        className={`grid gap-4 ${variant === 'compact' ? 'grid-cols-1 sm:grid-cols-2 lg:grid-cols-3' : 'grid-cols-1 md:grid-cols-2'}`}
        variants={containerVariants}
        initial="hidden"
        animate="visible"
      >
        {providers.map((provider) => (
          <motion.div key={provider.id} variants={itemVariants}>
            <ProviderInfo
              provider={provider}
              isSelected={selectedProviderId === provider.id}
              onSelect={handleProviderSelect}
              variant={variant}
              showCategory={showCategory}
            />
          </motion.div>
        ))}
      </motion.div>
    </div>
  );
}
