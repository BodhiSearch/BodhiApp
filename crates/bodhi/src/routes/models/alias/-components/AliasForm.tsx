import React, { useEffect } from 'react';

import { AliasResponse } from '@bodhiapp/ts-client';
import { zodResolver } from '@hookform/resolvers/zod';
import { useNavigate } from '@tanstack/react-router';
import { HelpCircle } from 'lucide-react';
import { useForm } from 'react-hook-form';

import { Button } from '@/components/ui/button';
import { Form, FormControl, FormField, FormItem, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { useCreateModel, useUpdateModel } from '@/hooks/models';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { hasLocalFileProperties, isUserAlias, isApiAlias } from '@/lib/utils';
import { ALIAS_FORM_TOOLTIPS } from '@/routes/models/-components/tooltips';
import {
  AliasFormData,
  createAliasFormSchema,
  convertFormToApi,
  convertFormToUpdateApi,
  convertApiToForm,
} from '@/schemas/alias';

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
import { RepoCombobox } from './RepoCombobox';

import './local-form.css';

interface AliasFormProps {
  isEditMode: boolean;
  initialData?: AliasResponse;
}

/** Field label with an optional help tooltip + a required marker. */
function LfLabel({
  label,
  tooltip,
  htmlFor,
  required,
}: {
  label: string;
  tooltip?: string;
  htmlFor?: string;
  required?: boolean;
}) {
  return (
    <label className="lf-label" htmlFor={htmlFor}>
      <span>{label}</span>
      {required && <span className="lf-req">*</span>}
      {tooltip && (
        <TooltipProvider>
          <Tooltip delayDuration={300}>
            <TooltipTrigger asChild>
              <HelpCircle className="h-3.5 w-3.5 text-muted-foreground hover:text-foreground transition-colors cursor-help" />
            </TooltipTrigger>
            <TooltipContent sideOffset={8}>
              <p className="max-w-xs text-sm">{tooltip}</p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      )}
    </label>
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
      updateModel.mutate(convertFormToUpdateApi(data));
    } else {
      createModel.mutate(convertFormToApi(data));
    }
  };

  const handleSubmit = form.handleSubmit((data) => onSubmit(data));

  return (
    <Form {...form}>
      <form onSubmit={handleSubmit} data-testid={formTestId}>
        <div className="lf-card">
          <div className="lf-card-head">
            <h1 className="lf-card-title">{isEditMode ? 'Edit Local Model' : 'Create New Local Model'}</h1>
            <p className="lf-card-sub">
              Set up a named alias for a local GGUF model, with runtime flags and request defaults.
            </p>
          </div>

          <div className="lf-card-body">
            <section className="lf-section">
              <div className="lf-section-title">Identity</div>
              <FormField
                control={form.control}
                name="alias"
                render={({ field }) => (
                  <FormItem className="lf-field">
                    <LfLabel label="Alias name" tooltip={ALIAS_FORM_TOOLTIPS.alias} htmlFor="alias" required />
                    <FormControl>
                      <Input
                        {...field}
                        id="alias"
                        data-testid="alias-input"
                        placeholder="e.g. qwen-api"
                        disabled={isEditMode}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </section>

            <div className="lf-divider" />

            <section className="lf-section">
              <div className="lf-section-title">Model file</div>
              <div className="lf-field-row">
                <FormField
                  control={form.control}
                  name="repo"
                  render={({ field }) => (
                    <FormItem className="lf-field">
                      <LfLabel label="Repo" tooltip={ALIAS_FORM_TOOLTIPS.repo} htmlFor="repo-input" />
                      <FormControl>
                        <RepoCombobox value={field.value} onChange={field.onChange} testId="repo-input" />
                      </FormControl>
                      <p className="lf-hint">
                        Suggestions shown — or type any <span className="lf-code">&lt;org&gt;/&lt;repo&gt;</span> to
                        download.
                      </p>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <FormField
                  control={form.control}
                  name="snapshot"
                  render={({ field }) => (
                    <FormItem className="lf-field">
                      <LfLabel
                        label="Snapshot"
                        tooltip="Git reference or commit SHA (defaults to main)"
                        htmlFor="snapshot-input"
                      />
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
                      <p className="lf-hint">
                        Defaults to <span className="lf-code">main</span> — or paste a commit SHA / branch.
                      </p>
                      <FormMessage />
                    </FormItem>
                  )}
                />
              </div>

              <FormField
                control={form.control}
                name="filename"
                render={({ field }) => (
                  <FormItem className="lf-field">
                    <LfLabel label="Quantisation — selects file" tooltip={ALIAS_FORM_TOOLTIPS.filename} />
                    <FormControl>
                      <QuantSelector repo={form.watch('repo')} value={field.value} onSelect={field.onChange} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </section>

            <div className="lf-divider" />

            <section className="lf-section">
              <div className="lf-section-title">Runtime flags</div>
              <FormField
                control={form.control}
                name="context_params"
                render={({ field }) => (
                  <FormItem className="lf-field">
                    <div className="lf-split">
                      <div>
                        <div className="lf-split-label">Active runtime flags</div>
                        <FormControl>
                          <textarea
                            {...field}
                            id="context_params"
                            data-testid="context-params"
                            value={field.value || ''}
                            onChange={(e) => field.onChange(e.target.value)}
                            placeholder={'Enter llama-server parameters, one per line:\n--ctx-size 2048\n--parallel 4'}
                            className="lf-textarea"
                          />
                        </FormControl>
                        <p className="lf-hint mt-1.5">One flag per line. Click a flag on the right to append it.</p>
                      </div>
                      <ParamCatalog
                        label="Available flags — click to add"
                        catalog={RUNTIME_FLAGS}
                        addedKeys={flagKeysInText(field.value || '')}
                        onAdd={(entry) => field.onChange(appendFlagLine(field.value || '', entry))}
                        testIdPrefix="context-flag"
                      />
                    </div>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </section>

            <div className="lf-divider" />

            <section className="lf-section">
              <div className="lf-section-title">Request defaults</div>
              <FormField
                control={form.control}
                name="system_prompt"
                render={({ field }) => (
                  <FormItem className="lf-field">
                    <LfLabel
                      label="System prompt"
                      tooltip="Prepended to every chat request for this alias (as a leading system message), unless the request already supplies one."
                      htmlFor="system_prompt"
                    />
                    <FormControl>
                      <textarea
                        {...field}
                        id="system_prompt"
                        data-testid="system-prompt"
                        value={field.value || ''}
                        onChange={(e) => field.onChange(e.target.value)}
                        placeholder="You are a helpful assistant."
                        className="lf-textarea"
                        style={{ minHeight: 72 }}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="request_params_text"
                render={({ field }) => (
                  <FormItem className="lf-field">
                    <LfLabel
                      label="Request parameters"
                      tooltip="OpenAI-compatible request defaults, one key=value per line: temperature=0.7"
                    />
                    <div className="lf-split">
                      <div>
                        <div className="lf-split-label">Active parameters</div>
                        <FormControl>
                          <textarea
                            {...field}
                            id="request_params_text"
                            data-testid="request-params"
                            value={field.value || ''}
                            onChange={(e) => field.onChange(e.target.value)}
                            placeholder={'temperature=0.7\ntop_p=1.0'}
                            className="lf-textarea"
                          />
                        </FormControl>
                        <p className="lf-hint mt-1.5">
                          Format: <span className="lf-code">key=value</span>. Click a param on the right to append it.
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
                    <FormMessage />
                  </FormItem>
                )}
              />
            </section>
          </div>

          <div className="lf-footer">
            <Button type="button" variant="ghost" onClick={() => navigate({ to: '/models/' })}>
              Cancel
            </Button>
            <Button type="submit" data-testid="submit-alias-form">
              {isEditMode ? 'Update alias' : 'Create alias'}
            </Button>
          </div>
        </div>
      </form>
    </Form>
  );
};

export default AliasForm;
