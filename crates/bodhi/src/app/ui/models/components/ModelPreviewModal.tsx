'use client';

import React from 'react';

import type { AliasResponse, ModelMetadata } from '@bodhiapp/ts-client';
import { CheckCircle2, XCircle, Eye, EyeOff } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Separator } from '@/components/ui/separator';
import { hasModelMetadata, isApiAlias } from '@/lib/utils';

interface ModelPreviewModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  model: AliasResponse;
}

interface CapabilityRowProps {
  label: string;
  value: boolean | null | undefined;
  testId: string;
}

function CapabilityRow({ label, value, testId }: CapabilityRowProps) {
  if (value === undefined || value === null) return null;

  return (
    <div className="flex items-center justify-between py-1">
      <span className="text-sm text-muted-foreground">{label}</span>
      <div className="flex items-center gap-2" data-testid={testId}>
        {value ? (
          <>
            <CheckCircle2 className="h-4 w-4 text-green-600" />
            <span className="text-sm font-medium">Supported</span>
          </>
        ) : (
          <>
            <XCircle className="h-4 w-4 text-muted-foreground" />
            <span className="text-sm text-muted-foreground">Not supported</span>
          </>
        )}
      </div>
    </div>
  );
}

interface MetadataFieldProps {
  label: string;
  value: string | number | null | undefined;
  testId: string;
}

function MetadataField({ label, value, testId }: MetadataFieldProps) {
  if (value === undefined || value === null) return null;

  return (
    <div className="flex items-center justify-between py-1">
      <span className="text-sm text-muted-foreground">{label}</span>
      <span className="text-sm font-medium" data-testid={testId}>
        {typeof value === 'number' ? value.toLocaleString() : value}
      </span>
    </div>
  );
}

export function ModelPreviewModal({ open, onOpenChange, model }: ModelPreviewModalProps) {
  const isApiModel = isApiAlias(model);
  const isLocalModel = hasModelMetadata(model);

  const metadata: ModelMetadata | null | undefined = isLocalModel ? model.metadata : undefined;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-2xl max-h-[80vh] overflow-y-auto" data-testid="model-preview-modal">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Eye className="h-5 w-5" />
            Model Preview
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-4">
          {/* Basic Info Card */}
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-base">Basic Information</CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              <div className="flex items-center justify-between py-1">
                <span className="text-sm text-muted-foreground">Name</span>
                <span className="text-sm font-medium" data-testid="preview-basic-alias">
                  {isApiAlias(model) ? model.id : hasModelMetadata(model) ? model.alias : ''}
                </span>
              </div>
              {hasModelMetadata(model) && (
                <>
                  <div className="flex items-center justify-between py-1">
                    <span className="text-sm text-muted-foreground">Repository</span>
                    <span className="text-sm font-medium" data-testid="preview-basic-repo">
                      {model.repo}
                    </span>
                  </div>
                  <div className="flex items-center justify-between py-1">
                    <span className="text-sm text-muted-foreground">Filename</span>
                    <span className="text-sm font-medium" data-testid="preview-basic-filename">
                      {model.filename}
                    </span>
                  </div>
                  <div className="flex items-center justify-between py-1">
                    <span className="text-sm text-muted-foreground">Snapshot</span>
                    <span className="text-sm font-mono text-xs" data-testid="preview-basic-snapshot">
                      {model.snapshot}
                    </span>
                  </div>
                </>
              )}
              <div className="flex items-center justify-between py-1">
                <span className="text-sm text-muted-foreground">Type</span>
                <Badge variant={isApiModel ? 'default' : 'secondary'} data-testid="preview-basic-source">
                  {model.source.toUpperCase()}
                </Badge>
              </div>
            </CardContent>
          </Card>

          {/* API Model Configuration */}
          {isApiAlias(model) && (
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base">API Configuration</CardTitle>
              </CardHeader>
              <CardContent className="space-y-2">
                <MetadataField label="API Format" value={model.api_format} testId="preview-api-format" />
                <div className="flex items-center justify-between py-1">
                  <span className="text-sm text-muted-foreground">Base URL</span>
                  <span
                    className="text-sm font-medium font-mono text-blue-600 max-w-md truncate"
                    data-testid="preview-api-base-url"
                  >
                    {model.base_url}
                  </span>
                </div>
                {model.prefix && <MetadataField label="Prefix" value={model.prefix} testId="preview-api-prefix" />}
                <div className="flex items-center justify-between py-1">
                  <span className="text-sm text-muted-foreground">Forward All</span>
                  <span className="text-sm font-medium" data-testid="preview-api-forward-all">
                    {model.forward_all_with_prefix ? 'Enabled' : 'Disabled'}
                  </span>
                </div>
                {model.models && model.models.length > 0 && (
                  <div className="pt-2">
                    <span className="text-sm text-muted-foreground">Available Models:</span>
                    <div className="flex flex-wrap gap-1 mt-2" data-testid="preview-api-models">
                      {model.models.slice(0, 10).map((m: string) => (
                        <Badge key={m} variant="outline" className="text-xs">
                          {m}
                        </Badge>
                      ))}
                      {model.models.length > 10 && (
                        <Badge variant="outline" className="text-xs">
                          +{model.models.length - 10} more
                        </Badge>
                      )}
                    </div>
                  </div>
                )}
              </CardContent>
            </Card>
          )}

          {/* Capabilities Card (Local models only) */}
          {isLocalModel && metadata?.capabilities && (
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base">Capabilities</CardTitle>
              </CardHeader>
              <CardContent className="space-y-2">
                <CapabilityRow label="Vision" value={metadata.capabilities.vision} testId="preview-capability-vision" />
                <CapabilityRow label="Audio" value={metadata.capabilities.audio} testId="preview-capability-audio" />
                <CapabilityRow
                  label="Thinking"
                  value={metadata.capabilities.thinking}
                  testId="preview-capability-thinking"
                />
                <Separator className="my-2" />
                <div className="text-sm font-medium text-muted-foreground mb-2">Tool Capabilities</div>
                <CapabilityRow
                  label="Function Calling"
                  value={metadata.capabilities.tools?.function_calling}
                  testId="preview-capability-function-calling"
                />
                <CapabilityRow
                  label="Structured Output"
                  value={metadata.capabilities.tools?.structured_output}
                  testId="preview-capability-structured-output"
                />
              </CardContent>
            </Card>
          )}

          {/* Context Limits Card (Local models only) */}
          {isLocalModel && metadata?.context && (
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base">Context Limits</CardTitle>
              </CardHeader>
              <CardContent className="space-y-2">
                <MetadataField
                  label="Max Input Tokens"
                  value={metadata.context.max_input_tokens}
                  testId="preview-context-max-input"
                />
                <MetadataField
                  label="Max Output Tokens"
                  value={metadata.context.max_output_tokens}
                  testId="preview-context-max-output"
                />
              </CardContent>
            </Card>
          )}

          {/* Architecture Card (Local models only) */}
          {isLocalModel && metadata?.architecture && (
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base">Architecture</CardTitle>
              </CardHeader>
              <CardContent className="space-y-2">
                <MetadataField
                  label="Format"
                  value={metadata.architecture.format}
                  testId="preview-architecture-format"
                />
                <MetadataField
                  label="Family"
                  value={metadata.architecture.family}
                  testId="preview-architecture-family"
                />
                <MetadataField
                  label="Parameter Count"
                  value={metadata.architecture.parameter_count}
                  testId="preview-architecture-parameter-count"
                />
                <MetadataField
                  label="Quantization"
                  value={metadata.architecture.quantization}
                  testId="preview-architecture-quantization"
                />
              </CardContent>
            </Card>
          )}

          {/* No Metadata Available */}
          {isLocalModel && !metadata && (
            <Card>
              <CardContent className="py-6">
                <div className="flex flex-col items-center gap-2 text-center">
                  <EyeOff className="h-8 w-8 text-muted-foreground" />
                  <p className="text-sm text-muted-foreground">No metadata available for this model.</p>
                  <p className="text-xs text-muted-foreground">
                    Click &quot;Refresh All&quot; to extract metadata from the GGUF file.
                  </p>
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}
