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
  isEditMode: boolean;
  initialData?: ApiModelResponse;
}

export default function ApiModelForm({ isEditMode, initialData }: ApiModelFormProps) {
  const router = useRouter();
  const { toast } = useToast();

  // Use the centralized business logic hook
  const formLogic = useApiModelForm({
    mode: isEditMode ? 'edit' : 'create',
    initialData,
    onSuccess: (data) => {
      toast({
        title: isEditMode ? 'API Model Updated' : 'API Model Created',
        description: isEditMode
          ? `Successfully updated ${initialData?.id}`
          : `Successfully created API model: ${data.id}`,
      });
      router.push('/ui/models');
    },
    onError: (errorMessage) => {
      toast({
        title: isEditMode ? 'Failed to Update API Model' : 'Failed to Create API Model',
        description: errorMessage,
        variant: 'destructive',
      });
    },
    onCancel: () => {
      router.push('/ui/models');
    },
  });

  return (
    <form
      onSubmit={formLogic.handleSubmit}
      className="space-y-8 mx-4 my-6"
      data-testid={formLogic.isEditMode ? 'edit-api-model-form' : 'create-api-model-form'}
    >
      <Card>
        <CardHeader>
          <CardTitle>{formLogic.isEditMode ? 'Edit API Model' : 'Create New API Model'}</CardTitle>
          <CardDescription>
            {formLogic.isEditMode
              ? 'Update the configuration for your API model'
              : 'Configure a new external AI API model'}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          {/* API Format Selection */}
          <ApiFormatSelector
            value={formLogic.watchedValues.api_format}
            options={formLogic.apiFormatsData}
            onValueChange={formLogic.handleApiFormatChange}
            data-testid="api-model-format"
          />

          {/* Base URL Input */}
          <BaseUrlInput
            {...formLogic.register('base_url')}
            error={formLogic.errors.base_url?.message}
            data-testid="api-model-base-url"
          />

          {/* API Key Input */}
          <ApiKeyInput
            {...formLogic.register('api_key')}
            mode={formLogic.mode}
            error={formLogic.errors.api_key?.message}
            data-testid="api-model-api-key"
          />

          {/* Prefix Input */}
          <PrefixInput
            value={formLogic.watchedValues.prefix}
            onChange={(value) => formLogic.setValue('prefix', value)}
            enabled={formLogic.watchedValues.usePrefix}
            onEnabledChange={(enabled) => formLogic.setValue('usePrefix', enabled)}
            error={formLogic.errors.prefix?.message}
            data-testid="api-model-prefix"
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
          />

          {/* Form Actions */}
          <FormActions
            primaryAction={{
              label: formLogic.isEditMode ? 'Update API Model' : 'Create API Model',
              type: 'submit',
              disabled: formLogic.isLoading,
              loading: formLogic.isLoading,
              'data-testid': formLogic.isEditMode ? 'update-api-model-button' : 'create-api-model-button',
            }}
            secondaryAction={{
              label: 'Cancel',
              onClick: formLogic.handleCancel,
              'data-testid': 'cancel-button',
            }}
            testConnection={formLogic.testConnection}
          />
        </CardContent>
      </Card>
    </form>
  );
}
