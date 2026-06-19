import { Check, Download, ExternalLink } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { ModelInfo } from '@/hooks/models/model-catalog-types';

import { RatingStars } from './RatingStars';

interface ModelCardProps {
  model: ModelInfo;
  onDownload: () => void;
  downloadProgress?: {
    downloadedBytes: number | null;
    totalBytes: number | null;
  };
}

function Spec({ label, value }: { label: string; value: string }) {
  return (
    <span className="inline-flex items-center gap-1.5 rounded-md bg-muted/60 px-2 py-1 text-[11px]">
      <span className="text-muted-foreground">{label}</span>
      <b className="font-semibold">{value}</b>
    </span>
  );
}

export const ModelCard = ({ model, onDownload, downloadProgress }: ModelCardProps) => {
  const isRecommended = model.tier === 'premium';

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
        <Button className="w-full gap-2" variant="outline" disabled>
          <Check className="h-4 w-4" />
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
          <div data-testid="progress-bar" className="download-progress-track h-1.5 w-full">
            <div className="download-progress-fill" style={{ width: `${Math.min(progress, 100)}%` }} />
          </div>
        </div>
      );
    }

    return (
      <Button data-testid="download-button" className="w-full gap-2" onClick={onDownload}>
        <Download className="h-4 w-4" />
        Download
      </Button>
    );
  };

  return (
    <TooltipProvider>
      <article
        data-testid="model-card"
        className={`flex h-full flex-col rounded-[var(--radius-lg)] border bg-card p-5 transition-all duration-200 hover:border-primary/40 hover:shadow-md ${
          isRecommended
            ? 'border-primary/55 shadow-[0_0_0_1px_hsl(var(--primary)/0.25)]'
            : 'border-[hsl(var(--border-strong))]'
        }`}
      >
        <div className="mb-1 flex items-start justify-between gap-3">
          <h3 className="flex items-center gap-1.5 text-base font-bold leading-tight tracking-tight">
            <a
              data-testid="huggingface-link"
              href={`https://huggingface.co/${model.repo}`}
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-1 hover:underline"
            >
              {model.name}
              <ExternalLink className="h-3 w-3 text-muted-foreground" />
            </a>
          </h3>
          {model.badge && (
            <span
              className={`shrink-0 whitespace-nowrap rounded-full px-2 py-0.5 text-[10.5px] font-semibold ${
                isRecommended ? 'bg-primary/[0.14] text-[hsl(var(--primary-hover))]' : 'bg-muted text-muted-foreground'
              }`}
            >
              {model.badge}
            </span>
          )}
        </div>

        <Tooltip>
          <TooltipTrigger asChild>
            <p className="mb-3 cursor-help text-[13px] leading-relaxed text-muted-foreground">
              {model.tooltipContent.useCase}
            </p>
          </TooltipTrigger>
          <TooltipContent side="top" className="max-w-xs">
            <div className="space-y-2">
              <div>
                <p className="mb-1 text-xs font-semibold">Strengths:</p>
                <ul className="list-inside list-disc space-y-0.5 text-xs">
                  {model.tooltipContent.strengths.map((strength, idx) => (
                    <li key={idx}>{strength}</li>
                  ))}
                </ul>
              </div>
              <div>
                <p className="mb-1 text-xs font-semibold">Research Notes:</p>
                <p className="text-xs text-muted-foreground">{model.tooltipContent.researchNotes}</p>
              </div>
            </div>
          </TooltipContent>
        </Tooltip>

        <div className="mb-3 flex flex-wrap gap-1.5">
          <Spec label="Size" value={model.size} />
          <Spec label="Params" value={model.parameters} />
          <Spec label="Context" value={model.contextWindow} />
          <Spec label="Quant" value={model.quantization} />
        </div>

        <div className="mb-4 space-y-1.5 text-[11.5px]">
          <div className="flex items-center gap-2.5">
            <span className="w-14 shrink-0 text-muted-foreground">Quality</span>
            <RatingStars rating={model.ratings.quality} size="sm" />
          </div>
          <div className="flex items-center gap-2.5">
            <span className="w-14 shrink-0 text-muted-foreground">Speed</span>
            <RatingStars rating={model.ratings.speed} size="sm" />
          </div>
        </div>

        <div className="mt-auto">{renderDownloadButton()}</div>
      </article>
    </TooltipProvider>
  );
};
