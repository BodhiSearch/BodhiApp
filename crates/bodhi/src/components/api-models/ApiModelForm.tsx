import React from 'react';

import { ApiAliasResponse } from '@bodhiapp/ts-client';
import { useNavigate } from '@tanstack/react-router';

import { FormActions } from '@/components/api-models/actions/FormActions';
import { ApiFormatSelector } from '@/components/api-models/form/ApiFormatSelector';
import { ApiKeyInput } from '@/components/api-models/form/ApiKeyInput';
import { BaseUrlInput } from '@/components/api-models/form/BaseUrlInput';
import { ExtrasSection } from '@/components/api-models/form/ExtrasSection';
import { FormSection } from '@/components/api-models/form/FormSection';
import { ForwardModeSelector } from '@/components/api-models/form/ForwardModeSelector';
import { LlmLibertyEnvelopeInput } from '@/components/api-models/form/LlmLibertyEnvelopeInput';
import { ModelSelectionSection } from '@/components/api-models/form/ModelSelectionSection';
import { NameInput } from '@/components/api-models/form/NameInput';
import { PrefixInput } from '@/components/api-models/form/PrefixInput';
import { useApiModelForm } from '@/components/api-models/hooks/useApiModelForm';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useToast } from '@/hooks/use-toast';

/** Seed values for create mode, e.g. from the Explore catalog "Configure in Bodhi" bridge. */
export interface ApiModelPrefill {
  api_format?: string;
  base_url?: string;
  model?: string;
}

interface ApiModelFormProps {
  mode: 'create' | 'edit' | 'setup';
  initialData?: ApiAliasResponse;
  prefill?: ApiModelPrefill;
  onSuccessRoute?: string;
  onCancelRoute?: string;
}

export default function ApiModelForm({ mode, initialData, prefill, onSuccessRoute, onCancelRoute }: ApiModelFormProps) {
  const navigate = useNavigate();
  const { toast } = useToast();

  const defaultSuccessRoute = mode === 'setup' ? '/setup/complete/' : '/models/';
  const defaultCancelRoute = mode === 'setup' ? '/setup/complete/' : '/models/';

  const successRoute = onSuccessRoute || defaultSuccessRoute;
  const cancelRoute = onCancelRoute || defaultCancelRoute;

  const isEditMode = mode === 'edit';

  const formLogic = useApiModelForm({
    mode,
    initialData,
    prefill,
    onSuccess: (data) => {
      toast({
        title: isEditMode ? 'API Model Updated' : 'API Model Created',
        description: isEditMode
          ? `Successfully updated ${initialData?.id}`
          : `Successfully created API model: ${data.id}`,
      });
      navigate({ to: successRoute });
    },
    onError: (errorMessage) => {
      toast({
        title: isEditMode ? 'Failed to Update API Model' : 'Failed to Create API Model',
        description: errorMessage,
        variant: 'destructive',
      });
    },
    onCancel: () => {
      navigate({ to: cancelRoute });
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
      <Card className="border-border bg-card shadow-sm">
        {mode !== 'setup' && (
          <CardHeader>
            <CardTitle>{mode === 'edit' ? 'Edit API Model' : 'Create New API Model'}</CardTitle>
            <CardDescription>
              {mode === 'edit'
                ? 'Update the configuration for your API model'
                : 'Configure a new external AI API model'}
            </CardDescription>
          </CardHeader>
        )}
        <CardContent className={`space-y-8 ${mode === 'setup' ? 'pt-6' : ''}`}>
          <FormSection title="Provider Connection">
            <NameInput {...formLogic.register('name')} error={formLogic.errors.name?.message} />

            <ApiFormatSelector
              value={formLogic.watchedValues.api_format}
              options={formLogic.apiFormatsData}
              onValueChange={formLogic.handleApiFormatChange}
              disabled={isEditMode}
            />

            {formLogic.watchedValues.api_format === 'llm_liberty_oauth' ? (
              /* LLM Liberty OAuth: envelope replaces base_url, api_key, and extras */
              <LlmLibertyEnvelopeInput
                value={formLogic.watchedValues.llm_liberty_envelope || ''}
                onChange={(value) => formLogic.setValue('llm_liberty_envelope', value)}
                error={formLogic.errors.llm_liberty_envelope?.message}
                mode={formLogic.mode}
                hasStoredCredentials={Boolean(initialData?.llm_liberty)}
              />
            ) : (
              <>
                <BaseUrlInput {...formLogic.register('base_url')} error={formLogic.errors.base_url?.message} />

                <ApiKeyInput
                  {...formLogic.register('api_key')}
                  mode={formLogic.mode}
                  enabled={formLogic.watchedValues.useApiKey}
                  onEnabledChange={(enabled) => formLogic.setValue('useApiKey', enabled)}
                  error={formLogic.errors.api_key?.message}
                />

                {/* Extra Headers and Body (conditionally shown for formats with defaults) */}
                {formLogic.showExtras && (
                  <ExtrasSection
                    extraHeaders={formLogic.watchedValues.extra_headers || ''}
                    extraBody={formLogic.watchedValues.extra_body || ''}
                    onExtraHeadersChange={(value) => formLogic.setValue('extra_headers', value)}
                    onExtraBodyChange={(value) => formLogic.setValue('extra_body', value)}
                    extraHeadersError={formLogic.errors.extra_headers?.message}
                    extraBodyError={formLogic.errors.extra_body?.message}
                  />
                )}
              </>
            )}
          </FormSection>

          <div className="border-t border-border" />

          <FormSection title="Request Routing">
            <PrefixInput
              value={formLogic.watchedValues.prefix}
              onChange={(value) => formLogic.setValue('prefix', value)}
              enabled={formLogic.watchedValues.usePrefix}
              onEnabledChange={(enabled) => formLogic.setValue('usePrefix', enabled)}
              error={formLogic.errors.prefix?.message}
            />

            <ForwardModeSelector
              forwardAll={formLogic.watchedValues.forward_all_with_prefix || false}
              onForwardAllChange={(value) => formLogic.setValue('forward_all_with_prefix', value)}
              prefixEnabled={formLogic.watchedValues.usePrefix}
              prefix={formLogic.watchedValues.prefix}
              error={formLogic.errors.forward_all_with_prefix?.message}
            />
          </FormSection>

          <div className="border-t border-border" />

          <FormSection title="Model Selection">
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
              disabled={formLogic.watchedValues.forward_all_with_prefix || false}
            />
          </FormSection>

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
