import { Download, Check, Info, ExternalLink } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle, CardFooter } from '@/components/ui/card';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';

import { RatingStars } from './RatingStars';
import { ModelInfo } from './types';

interface ModelCardProps {
  model: ModelInfo;
  onDownload: () => void;
  downloadProgress?: {
    downloadedBytes: number | null;
    totalBytes: number | null;
  };
}

export const ModelCard = ({ model, onDownload, downloadProgress }: ModelCardProps) => {
  const computeProgress = () => {
    if (!downloadProgress?.totalBytes || downloadProgress.totalBytes === 0) return 0;
    if (!downloadProgress?.downloadedBytes) return 0;
    return (downloadProgress.downloadedBytes / downloadProgress.totalBytes) * 100;
  };

  const formatBytes = (bytes: number | null | undefined) => {
    if (!bytes || bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  };

  const renderDownloadButton = () => {
    if (model.downloadState?.status === 'completed') {
      return (
        <Button className="w-full" variant="outline" disabled>
          <Check className="mr-2 h-4 w-4" />
          Downloaded
        </Button>
      );
    }

    if (model.downloadState?.status === 'pending') {
      const progress = computeProgress();
      return (
        <div className="w-full space-y-1">
          <div className="flex items-center justify-between text-xs text-muted-foreground">
            <span>{progress.toFixed(0)}%</span>
            {downloadProgress?.totalBytes && (
              <span data-testid="byte-display" className="text-xs">
                {formatBytes(downloadProgress.downloadedBytes)} / {formatBytes(downloadProgress.totalBytes)}
              </span>
            )}
          </div>
          <div data-testid="progress-bar" className="w-full bg-secondary rounded-full h-1.5">
            <div
              className="bg-primary h-1.5 rounded-full transition-all duration-300"
              style={{ width: `${Math.min(progress, 100)}%` }}
            />
          </div>
        </div>
      );
    }

    return (
      <Button data-testid="download-button" className="w-full" size="sm" onClick={onDownload}>
        <Download className="mr-2 h-3 w-3" />
        Download
      </Button>
    );
  };

  return (
    <TooltipProvider>
      <Card className="h-full flex flex-col hover:shadow-md transition-shadow">
        <CardHeader className="pb-2 space-y-1">
          <div className="flex items-start justify-between gap-2">
            <Tooltip>
              <TooltipTrigger asChild>
                <CardTitle className="text-sm font-semibold leading-tight cursor-help flex items-start gap-1">
                  <a
                    data-testid="huggingface-link"
                    href={`https://huggingface.co/${model.repo}`}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="hover:underline flex items-center gap-1"
                  >
                    {model.name}
                    <ExternalLink className="h-3 w-3" />
                  </a>
                  <Info className="h-3 w-3 mt-0.5 text-muted-foreground" />
                </CardTitle>
              </TooltipTrigger>
              <TooltipContent side="top" className="max-w-xs">
                <div className="space-y-2">
                  <div>
                    <p className="font-semibold text-xs mb-1">Use Case:</p>
                    <p className="text-xs">{model.tooltipContent.useCase}</p>
                  </div>
                  <div>
                    <p className="font-semibold text-xs mb-1">Strengths:</p>
                    <ul className="text-xs list-disc list-inside space-y-0.5">
                      {model.tooltipContent.strengths.map((strength, idx) => (
                        <li key={idx}>{strength}</li>
                      ))}
                    </ul>
                  </div>
                  <div>
                    <p className="font-semibold text-xs mb-1">Research Notes:</p>
                    <p className="text-xs text-muted-foreground">{model.tooltipContent.researchNotes}</p>
                  </div>
                </div>
              </TooltipContent>
            </Tooltip>
            {model.badge && (
              <Badge variant="secondary" className="text-[10px] px-1.5 py-0 shrink-0">
                {model.badge}
              </Badge>
            )}
          </div>
        </CardHeader>

        <CardContent className="flex-1 space-y-2 pb-2">
          <div className="grid grid-cols-2 gap-x-3 gap-y-1 text-[11px]">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Size:</span>
              <span className="font-medium">{model.size}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Params:</span>
              <span className="font-medium">{model.parameters}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Quant:</span>
              <span className="font-medium">{model.quantization}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Context:</span>
              <span className="font-medium">{model.contextWindow}</span>
            </div>
            {model.benchmarks.mmlu !== undefined && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">MMLU:</span>
                <span className="font-medium">{model.benchmarks.mmlu}</span>
              </div>
            )}
            {model.benchmarks.humanEval !== undefined && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">HumanEval:</span>
                <span className="font-medium">{model.benchmarks.humanEval}</span>
              </div>
            )}
            {model.benchmarks.mteb !== undefined && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">MTEB:</span>
                <span className="font-medium">{model.benchmarks.mteb}</span>
              </div>
            )}
          </div>

          <div className="pt-1 border-t space-y-1">
            <Tooltip>
              <TooltipTrigger asChild>
                <div className="flex justify-between items-center text-[11px] cursor-help">
                  <span className="text-muted-foreground">Quality</span>
                  <RatingStars rating={model.ratings.quality} size="xs" />
                </div>
              </TooltipTrigger>
              <TooltipContent side="left" className="text-xs">
                Based on benchmark accuracy (MMLU, BBH, GPQA)
              </TooltipContent>
            </Tooltip>

            <Tooltip>
              <TooltipTrigger asChild>
                <div className="flex justify-between items-center text-[11px] cursor-help">
                  <span className="text-muted-foreground">Speed</span>
                  <RatingStars rating={model.ratings.speed} size="xs" />
                </div>
              </TooltipTrigger>
              <TooltipContent side="left" className="text-xs">
                Inference speed and model efficiency
              </TooltipContent>
            </Tooltip>

            <Tooltip>
              <TooltipTrigger asChild>
                <div className="flex justify-between items-center text-[11px] cursor-help">
                  <span className="text-muted-foreground">Specialty</span>
                  <RatingStars rating={model.ratings.specialization} size="xs" />
                </div>
              </TooltipTrigger>
              <TooltipContent side="left" className="text-xs">
                Domain-specific strength (Reasoning/Coding/Multilingual/Context)
              </TooltipContent>
            </Tooltip>
          </div>
        </CardContent>

        <CardFooter className="pt-2">{renderDownloadButton()}</CardFooter>
      </Card>
    </TooltipProvider>
  );
};
