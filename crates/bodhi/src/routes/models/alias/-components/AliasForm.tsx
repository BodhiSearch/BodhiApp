import React, { useEffect } from 'react';

import { AliasResponse } from '@bodhiapp/ts-client';
import { zodResolver } from '@hookform/resolvers/zod';
import { useNavigate } from '@tanstack/react-router';
import { HelpCircle } from 'lucide-react';
import { useForm } from 'react-hook-form';

import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { useCreateModel, useUpdateModel } from '@/hooks/models';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { hasLocalFileProperties, isUserAlias, isApiAlias } from '@/lib/utils';
import {
  AliasFormData,
  createAliasFormSchema,
  convertFormToApi,
  convertFormToUpdateApi,
  convertApiToForm,
} from '@/schemas/alias';

import { ALIAS_FORM_TOOLTIPS } from '../../-components/tooltips';

import { ParamCatalog } from './ParamCatalog';
import {
  RUNTIME_FLAGS,
  REQUEST_PARAMS,
  appendFlagLine,
  appendParamLine,
  flagKeysInText,
  paramKeysInText,
} from './paramCatalogs';
import { QuantSelector } from './QuantSelector';

interface AliasFormProps {
  isEditMode: boolean;
  initialData?: AliasResponse;
}

function FormFieldWithTooltip({
  label,
  tooltip,
  children,
  htmlFor,
}: {
  label: string;
  tooltip: string;
  children: React.ReactNode;
  htmlFor: string;
}) {
  return (
    <>
      <div className="flex items-center gap-2 mb-2">
        <FormLabel
          className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
          htmlFor={htmlFor}
        >
          {label}
        </FormLabel>
        <TooltipProvider>
          <Tooltip delayDuration={300}>
            <TooltipTrigger asChild>
              <HelpCircle className="h-4 w-4 text-muted-foreground hover:text-foreground transition-colors cursor-help" />
            </TooltipTrigger>
            <TooltipContent sideOffset={8}>
              <p className="max-w-xs text-sm">{tooltip}</p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      </div>
      {children}
    </>
  );
}

const AliasForm: React.FC<AliasFormProps> = ({ isEditMode, initialData }) => {
  const navigate = useNavigate();
  const { showSuccess, showError } = useToastMessages();
  const formTestId = isEditMode ? 'form-edit-alias' : 'form-create-alias';

  const form = useForm<AliasFormData>({
    resolver: zodResolver(createAliasFormSchema),
    mode: 'onSubmit',
    defaultValues:
      initialData && hasLocalFileProperties(initialData)
        ? convertApiToForm(initialData)
        : {
            alias: '',
            repo: '',
            filename: '',
            snapshot: '',
            request_params_text: '',
            system_prompt: '',
            context_params: '',
          },
  });

  const createModel = useCreateModel({
    onSuccess: (model) => {
      const identifier = model.source === 'api' ? model.id : model.alias;
      showSuccess('Success', `Alias ${identifier} successfully created`);
      navigate({ to: '/models/' });
    },
    onError: (message) => {
      showError('Error', message);
    },
  });

  const editId = initialData
    ? isUserAlias(initialData)
      ? initialData.id
      : isApiAlias(initialData)
        ? initialData.id
        : ''
    : '';

  const updateModel = useUpdateModel(editId, {
    onSuccess: (model) => {
      const identifier = 'alias' in model ? model.alias : 'id' in model ? model.id : '';
      showSuccess('Success', `Alias ${identifier} successfully updated`);
      navigate({ to: '/models/' });
    },
    onError: (message) => {
      showError('Error', message);
    },
  });

  // Changing the repo invalidates the previously-selected quant (different repo, different files).
  useEffect(() => {
    const subscription = form.watch((_value, { name }) => {
      if (name === 'repo') {
        form.setValue('filename', '');
      }
    });
    return () => subscription.unsubscribe();
  }, [form]);

  const onSubmit = (data: AliasFormData) => {
    if (isEditMode) {
      const updateApiData = convertFormToUpdateApi(data);
      updateModel.mutate(updateApiData);
    } else {
      const createApiData = convertFormToApi(data);
      createModel.mutate(createApiData);
    }
  };

  const handleSubmit = form.handleSubmit((data) => onSubmit(data));

  return (
    <Form {...form}>
      <form onSubmit={handleSubmit} className="space-y-8" data-testid={formTestId}>
        <Card>
          <CardHeader>
            <CardTitle>{isEditMode ? 'Edit' : 'New'} Model Alias</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <FormField
              control={form.control}
              name="alias"
              render={({ field }) => (
                <FormItem>
                  <FormFieldWithTooltip label="Alias" tooltip={ALIAS_FORM_TOOLTIPS.alias} htmlFor="alias">
                    <FormControl>
                      <Input {...field} id="alias" data-testid="alias-input" disabled={isEditMode} />
                    </FormControl>
                  </FormFieldWithTooltip>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="repo"
              render={({ field }) => (
                <FormItem>
                  <FormFieldWithTooltip label="Repo" tooltip={ALIAS_FORM_TOOLTIPS.repo} htmlFor="repo-input">
                    <FormControl>
                      <Input
                        {...field}
                        id="repo-input"
                        data-testid="repo-input"
                        placeholder="org/repo"
                        className="font-mono"
                      />
                    </FormControl>
                  </FormFieldWithTooltip>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="filename"
              render={({ field }) => (
                <FormItem>
                  <FormFieldWithTooltip
                    label="Quantisation"
                    tooltip={ALIAS_FORM_TOOLTIPS.filename}
                    htmlFor="quant-selector"
                  >
                    <FormControl>
                      <QuantSelector repo={form.watch('repo')} value={field.value} onSelect={field.onChange} />
                    </FormControl>
                  </FormFieldWithTooltip>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="snapshot"
              render={({ field }) => (
                <FormItem>
                  <FormFieldWithTooltip
                    label="Snapshot"
                    tooltip="Git reference or commit SHA for the model version (defaults to main)"
                    htmlFor="snapshot-input"
                  >
                    <FormControl>
                      <Input
                        {...field}
                        id="snapshot-input"
                        data-testid="snapshot-input"
                        value={field.value || ''}
                        placeholder="main"
                        className="font-mono"
                      />
                    </FormControl>
                  </FormFieldWithTooltip>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="context_params"
              render={({ field }) => (
                <FormItem>
                  <FormFieldWithTooltip
                    label="Context Parameters"
                    tooltip="Enter llama-server parameters, one per line: --ctx-size 2048, --parallel 4"
                    htmlFor="context_params"
                  >
                    <FormControl>
                      <div className="grid gap-3 md:grid-cols-[1fr_18rem]">
                        <div>
                          <Textarea
                            {...field}
                            id="context_params"
                            data-testid="context-params"
                            value={field.value || ''}
                            onChange={(e) => field.onChange(e.target.value)}
                            placeholder="Enter llama-server parameters, one per line:&#10;--ctx-size 2048&#10;--parallel 4"
                            rows={8}
                            className="font-mono text-sm h-full"
                          />
                          <p className="text-sm text-muted-foreground mt-1.5">
                            One flag per line. Click a flag on the right to append it.
                          </p>
                        </div>
                        <ParamCatalog
                          label="Available flags — click to add"
                          catalog={RUNTIME_FLAGS}
                          addedKeys={flagKeysInText(field.value || '')}
                          onAdd={(entry) => field.onChange(appendFlagLine(field.value || '', entry))}
                          testIdPrefix="context-flag"
                        />
                      </div>
                    </FormControl>
                  </FormFieldWithTooltip>
                  <FormMessage />
                </FormItem>
              )}
            />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Request Defaults</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <FormField
              control={form.control}
              name="system_prompt"
              render={({ field }) => (
                <FormItem>
                  <FormFieldWithTooltip
                    label="System Prompt"
                    tooltip="Prepended to every chat request for this alias (as a leading system message), unless the request already supplies one."
                    htmlFor="system_prompt"
                  >
                    <FormControl>
                      <Textarea
                        {...field}
                        id="system_prompt"
                        data-testid="system-prompt"
                        value={field.value || ''}
                        onChange={(e) => field.onChange(e.target.value)}
                        placeholder="You are a helpful assistant."
                        rows={3}
                      />
                    </FormControl>
                  </FormFieldWithTooltip>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="request_params_text"
              render={({ field }) => (
                <FormItem>
                  <FormFieldWithTooltip
                    label="Request Parameters"
                    tooltip="OpenAI-compatible request defaults, one key=value per line: temperature=0.7"
                    htmlFor="request_params_text"
                  >
                    <FormControl>
                      <div className="grid gap-3 md:grid-cols-[1fr_18rem]">
                        <div>
                          <Textarea
                            {...field}
                            id="request_params_text"
                            data-testid="request-params"
                            value={field.value || ''}
                            onChange={(e) => field.onChange(e.target.value)}
                            placeholder="temperature=0.7&#10;top_p=1.0"
                            rows={8}
                            className="font-mono text-sm h-full"
                          />
                          <p className="text-sm text-muted-foreground mt-1.5">
                            Format: <span className="font-mono">key=value</span>. Click a param on the right to append
                            it.
                          </p>
                        </div>
                        <ParamCatalog
                          label="Available parameters — click to add"
                          catalog={REQUEST_PARAMS}
                          addedKeys={paramKeysInText(field.value || '')}
                          onAdd={(entry) => field.onChange(appendParamLine(field.value || '', entry))}
                          testIdPrefix="request-param"
                        />
                      </div>
                    </FormControl>
                  </FormFieldWithTooltip>
                  <FormMessage />
                </FormItem>
              )}
            />
          </CardContent>
        </Card>

        <div className="flex justify-center mt-8">
          <Button type="submit" data-testid="submit-alias-form">
            {isEditMode ? 'Update' : 'Create'} Model Alias
          </Button>
        </div>
      </form>
    </Form>
  );
};

export default AliasForm;
