'use client';

import React from 'react';
import { motion } from 'framer-motion';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Separator } from '@/components/ui/separator';

// Import shared components
import { useApiModelForm } from '@/components/api-models/hooks/useApiModelForm';
import { ProviderSelector } from '@/components/api-models/providers/ProviderSelector';
import { ApiKeyInput } from '@/components/api-models/form/ApiKeyInput';
import { BaseUrlInput } from '@/components/api-models/form/BaseUrlInput';
import { ModelSelectionSection } from '@/components/api-models/form/ModelSelectionSection';
import { TestConnectionButton } from '@/components/api-models/actions/TestConnectionButton';
import { FetchModelsButton } from '@/components/api-models/actions/FetchModelsButton';

interface ApiModelSetupFormProps {
  onComplete: () => void;
  onSkip: () => void;
}

export function ApiModelSetupForm({ onComplete, onSkip }: ApiModelSetupFormProps) {
  // Use the centralized business logic hook
  const formLogic = useApiModelForm({
    mode: 'setup',
    onSuccess: onComplete,
    autoSelectCommon: true, // Auto-select common models in setup
  });

  const isComplete = Boolean(
    formLogic.selectedProvider &&
      formLogic.watchedValues.api_key &&
      formLogic.watchedValues.base_url &&
      formLogic.watchedValues.models?.length > 0
  );

  return (
    <div className="space-y-8" data-testid="api-model-setup-form">
      {/* Provider Selection */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <span className="text-2xl">☁️</span>
            Choose Your AI Provider
          </CardTitle>
          <CardDescription>
            Select an AI provider to access cloud-based models like GPT-4, Claude, and more.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <ProviderSelector
            selectedProviderId={formLogic.selectedProvider?.id}
            onProviderSelect={formLogic.handleProviderSelect}
            variant="default"
            showCategory={true}
            title=""
            description=""
          />
        </CardContent>
      </Card>

      {/* Configuration Form */}
      {formLogic.selectedProvider && (
        <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.3 }}>
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <span className="text-2xl">🔑</span>
                Configure {formLogic.selectedProvider?.name}
              </CardTitle>
              <CardDescription>
                Enter your API credentials to connect to {formLogic.selectedProvider?.name} services.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <form onSubmit={formLogic.handleSubmit} className="space-y-6">
                {/* API Key Input */}
                <ApiKeyInput
                  {...formLogic.register('api_key')}
                  mode="setup"
                  error={formLogic.errors.api_key?.message}
                  data-testid="setup-api-key"
                  helpText={
                    formLogic.selectedProvider?.docUrl
                      ? `Don't have an API key? Get one from ${formLogic.selectedProvider.name}`
                      : undefined
                  }
                  docUrl={formLogic.selectedProvider?.docUrl}
                />

                {/* Base URL (for OpenAI-compatible providers) */}
                <BaseUrlInput
                  {...formLogic.register('base_url')}
                  error={formLogic.errors.base_url?.message}
                  data-testid="setup-base-url"
                  showWhen={formLogic.selectedProvider?.id === 'openai-compatible'}
                  helpText="Enter the complete API endpoint URL for your provider"
                />

                <Separator />

                {/* Test Connection & Fetch Models */}
                <div className="space-y-4">
                  <div className="flex flex-col sm:flex-row gap-2">
                    <TestConnectionButton
                      onTest={formLogic.testConnection.onTest}
                      canTest={formLogic.testConnection.canTest}
                      isLoading={formLogic.testConnection.isLoading}
                      status={formLogic.testConnection.status}
                      disabledReason={formLogic.testConnection.disabledReason}
                      variant="outline"
                      className="flex-1"
                      data-testid="test-connection"
                    />

                    <FetchModelsButton
                      onFetch={formLogic.fetchModels.onFetch}
                      canFetch={formLogic.fetchModels.canFetch}
                      isLoading={formLogic.fetchModels.isLoading}
                      status={formLogic.fetchModels.status}
                      disabledReason={formLogic.fetchModels.disabledReason}
                      modelCount={formLogic.fetchModels.availableModels.length}
                      variant="outline"
                      className="flex-1"
                      data-testid="fetch-models"
                    />
                  </div>
                </div>

                {/* Model Selection */}
                <ModelSelectionSection
                  selectedModels={formLogic.watchedValues.models || []}
                  availableModels={formLogic.fetchModels.availableModels}
                  onModelSelect={formLogic.handleModelSelect}
                  onModelRemove={formLogic.handleModelRemove}
                  onModelsSelectAll={formLogic.handleModelsSelectAll}
                  onFetchModels={formLogic.fetchModels.onFetch}
                  isFetchingModels={formLogic.fetchModels.isLoading}
                  canFetch={formLogic.fetchModels.canFetch}
                  fetchDisabledReason={formLogic.fetchModels.disabledReason}
                  error={formLogic.errors.models?.message}
                  provider={formLogic.selectedProvider}
                  autoSelectCommon={true}
                />

                <Separator />

                {/* Form Actions */}
                <div className="flex justify-between">
                  <button
                    type="button"
                    className="px-4 py-2 text-sm border border-border rounded-md hover:bg-muted transition-colors"
                    onClick={onSkip}
                    data-testid="skip-api-setup"
                  >
                    Skip for Now
                  </button>
                  <button
                    type="submit"
                    disabled={!isComplete || formLogic.isLoading}
                    className="px-4 py-2 text-sm bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex items-center gap-2"
                    data-testid="complete-api-setup"
                  >
                    {formLogic.isLoading ? (
                      <>
                        <div className="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin" />
                        Configuring...
                      </>
                    ) : (
                      'Complete Setup'
                    )}
                  </button>
                </div>
              </form>
            </CardContent>
          </Card>
        </motion.div>
      )}
    </div>
  );
}
