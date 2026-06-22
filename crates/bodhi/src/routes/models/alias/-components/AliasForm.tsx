import React, { useEffect, useState } from 'react';

import { AliasResponse } from '@bodhiapp/ts-client';
import { zodResolver } from '@hookform/resolvers/zod';
import { ChevronDown, ChevronUp, HelpCircle } from 'lucide-react';
import { useNavigate } from '@tanstack/react-router';
import { useForm } from 'react-hook-form';
import { z } from 'zod';

import { ALIAS_FORM_TOOLTIPS } from '../../-components/tooltips';
import { QuantSelector } from './QuantSelector';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { useCreateModel, useUpdateModel } from '@/hooks/models';
import { hasLocalFileProperties, isUserAlias, isApiAlias } from '@/lib/utils';
import {
  AliasFormData,
  createAliasFormSchema,
  requestParamsSchema,
  convertFormToApi,
  convertFormToUpdateApi,
  convertApiToForm,
} from '@/schemas/alias';

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
  const [isRequestExpanded, setIsRequestExpanded] = useState(
    isEditMode && initialData && isUserAlias(initialData) && Object.keys(initialData.request_params || {}).length > 0
  );
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
            request_params: {},
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

  const handleSubmit = form.handleSubmit(
    (data) => {
      const requestErrors = form.formState.errors.request_params;
      if (requestErrors) setIsRequestExpanded(true);

      if (!requestErrors) {
        onSubmit(data);
      }
    },
    (errors) => {
      if (errors.request_params) setIsRequestExpanded(true);
    }
  );

  const renderParamFields = (paramType: 'request_params') => {
    const schema = requestParamsSchema;
    return Object.entries(schema.shape).map(([key, field]) => {
      const fieldId = `${paramType}-${key}`;
      return (
        <FormField
          key={key}
          control={form.control}
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          name={`${paramType}.${key}` as any}
          render={({ field: formField }) => (
            <FormItem className="space-y-2 mb-4">
              <FormFieldWithTooltip
                label={key}
                tooltip={ALIAS_FORM_TOOLTIPS[key as keyof typeof ALIAS_FORM_TOOLTIPS]}
                htmlFor={fieldId}
              >
                <FormControl>
                  {Array.isArray(field) ? (
                    <Input
                      {...formField}
                      id={fieldId}
                      data-testid={`request-param-${key}`}
                      value={
                        formField.value
                          ? Array.isArray(formField.value)
                            ? formField.value.join(',')
                            : formField.value
                          : ''
                      }
                      onChange={(e) =>
                        formField.onChange(
                          e.target.value ? e.target.value.split(',').map((item) => item.trim()) : undefined
                        )
                      }
                      placeholder="Comma-separated values"
                    />
                  ) : (
                    <Input
                      {...formField}
                      id={fieldId}
                      data-testid={`request-param-${key}`}
                      type={field instanceof z.ZodNumber ? 'number' : 'text'}
                      min={field instanceof z.ZodNumber ? (field.minValue ?? undefined) : undefined}
                      max={field instanceof z.ZodNumber ? (field.maxValue ?? undefined) : undefined}
                      step={field instanceof z.ZodNumber && !field.isInt ? 0.1 : undefined}
                      value={formField.value ?? ''}
                      onChange={(e) => {
                        const inputValue = e.target.value;
                        if (inputValue === '') {
                          formField.onChange(undefined);
                        } else if (field instanceof z.ZodNumber) {
                          const numValue = field.isInt ? parseInt(inputValue, 10) : parseFloat(inputValue);
                          formField.onChange(isNaN(numValue) ? undefined : numValue);
                        } else {
                          formField.onChange(inputValue);
                        }
                      }}
                    />
                  )}
                </FormControl>
              </FormFieldWithTooltip>
              <FormMessage />
            </FormItem>
          )}
        />
      );
    });
  };

  return (
    <Form {...form}>
      <form onSubmit={handleSubmit} className="space-y-8 mx-4 my-6" data-testid={formTestId}>
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
                      <Textarea
                        {...field}
                        id="context_params"
                        data-testid="context-params"
                        value={field.value || ''}
                        onChange={(e) => field.onChange(e.target.value)}
                        placeholder="Enter llama-server parameters, one per line:&#10;--ctx-size 2048&#10;--parallel 4"
                        rows={4}
                        className="font-mono text-sm"
                      />
                    </FormControl>
                  </FormFieldWithTooltip>
                  <FormMessage />
                </FormItem>
              )}
            />
          </CardContent>
        </Card>

        <Card className="h-auto">
          <CardHeader
            className="cursor-pointer flex flex-row items-center justify-between"
            onClick={() => setIsRequestExpanded(!isRequestExpanded)}
            data-testid="request-params-toggle"
          >
            <CardTitle>Request Parameters</CardTitle>
            {isRequestExpanded ? <ChevronUp size={20} /> : <ChevronDown size={20} />}
          </CardHeader>
          <CardContent
            className={`overflow-hidden transition-all duration-300 ease-in-out ${isRequestExpanded ? 'max-h-[1000px] opacity-100' : 'max-h-0 opacity-0'}`}
          >
            {renderParamFields('request_params')}
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
