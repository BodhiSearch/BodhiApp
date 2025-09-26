'use client';

import React from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { ApiModelResponse } from '@bodhiapp/ts-client';
import { useRouter } from 'next/navigation';
import { useToast } from '@/hooks/use-toast';

// Import shared components
import { useApiModelForm } from '@/components/api-models/hooks/useApiModelForm';
import { ApiFormatSelector } from '@/components/api-models/form/ApiFormatSelector';
import { BaseUrlInput } from '@/components/api-models/form/BaseUrlInput';
import { ApiKeyInput } from '@/components/api-models/form/ApiKeyInput';
import { PrefixInput } from '@/components/api-models/form/PrefixInput';
import { ModelSelectionSection } from '@/components/api-models/form/ModelSelectionSection';
import { FormActions } from '@/components/api-models/actions/FormActions';

interface ApiModelFormProps {
  mode: 'create' | 'edit' | 'setup';
  initialData?: ApiModelResponse;
  onSuccessRoute?: string;
  onCancelRoute?: string;
}

export default function ApiModelForm({ mode, initialData, onSuccessRoute, onCancelRoute }: ApiModelFormProps) {
  const router = useRouter();
  const { toast } = useToast();

  // Determine default routes based on mode
  const defaultSuccessRoute = mode === 'setup' ? '/ui/setup/complete' : '/ui/models';
  const defaultCancelRoute = mode === 'setup' ? '/ui/setup/complete' : '/ui/models';

  const successRoute = onSuccessRoute || defaultSuccessRoute;
  const cancelRoute = onCancelRoute || defaultCancelRoute;

  const isEditMode = mode === 'edit';

  // Use the centralized business logic hook
  const formLogic = useApiModelForm({
    mode,
    initialData,
    onSuccess: (data) => {
      toast({
        title: isEditMode ? 'API Model Updated' : 'API Model Created',
        description: isEditMode
          ? `Successfully updated ${initialData?.id}`
          : `Successfully created API model: ${data.id}`,
      });
      router.push(successRoute);
    },
    onError: (errorMessage) => {
      toast({
        title: isEditMode ? 'Failed to Update API Model' : 'Failed to Create API Model',
        description: errorMessage,
        variant: 'destructive',
      });
    },
    onCancel: () => {
      router.push(cancelRoute);
    },
  });

  return (
    <form
      onSubmit={formLogic.handleSubmit}
      className="space-y-8 mx-4 my-6"
      data-testid={
        mode === 'edit' ? 'edit-api-model-form' : mode === 'setup' ? 'setup-api-model-form' : 'create-api-model-form'
      }
    >
      <Card>
        <CardHeader>
          <CardTitle>
            {mode === 'edit' ? 'Edit API Model' : mode === 'setup' ? 'Setup API Models' : 'Create New API Model'}
          </CardTitle>
          <CardDescription>
            {mode === 'edit'
              ? 'Update the configuration for your API model'
              : mode === 'setup'
                ? 'Configure cloud-based AI models for your setup'
                : 'Configure a new external AI API model'}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          {/* API Format Selection */}
          <ApiFormatSelector
            value={formLogic.watchedValues.api_format}
            options={formLogic.apiFormatsData}
            onValueChange={formLogic.handleApiFormatChange}
          />

          {/* Base URL Input */}
          <BaseUrlInput {...formLogic.register('base_url')} error={formLogic.errors.base_url?.message} />

          {/* API Key Input */}
          <ApiKeyInput
            {...formLogic.register('api_key')}
            mode={formLogic.mode}
            error={formLogic.errors.api_key?.message}
          />

          {/* Prefix Input */}
          <PrefixInput
            value={formLogic.watchedValues.prefix}
            onChange={(value) => formLogic.setValue('prefix', value)}
            enabled={formLogic.watchedValues.usePrefix}
            onEnabledChange={(enabled) => formLogic.setValue('usePrefix', enabled)}
            error={formLogic.errors.prefix?.message}
          />

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
            fetchStatus={formLogic.fetchModels.status}
          />

          {/* Form Actions */}
          <FormActions
            primaryAction={{
              label: mode === 'edit' ? 'Update API Model' : 'Create API Model',
              type: 'submit',
              disabled: formLogic.isLoading,
              loading: formLogic.isLoading,
              'data-testid': mode === 'edit' ? 'update-api-model-button' : 'create-api-model-button',
            }}
            secondaryAction={
              mode === 'setup'
                ? undefined
                : {
                    label: 'Cancel',
                    onClick: formLogic.handleCancel,
                    'data-testid': 'cancel-button',
                  }
            }
            testConnection={formLogic.testConnection}
          />
        </CardContent>
      </Card>
    </form>
  );
}
