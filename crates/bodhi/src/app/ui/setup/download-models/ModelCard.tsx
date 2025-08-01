import { Card, CardContent, CardHeader, CardTitle, CardFooter } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Download } from 'lucide-react';
import { ModelInfo } from './types';
import { RatingStars } from './RatingStars';

interface ModelCardProps {
  model: ModelInfo;
  onDownload: () => void;
}

export const ModelCard = ({ model, onDownload }: ModelCardProps) => {
  return (
    <Card className="h-full flex flex-col">
      <CardHeader>
        <CardTitle className="text-lg flex justify-between items-start">
          <div>
            {model.name}
            <div className="text-sm font-normal text-muted-foreground">{model.repo}</div>
          </div>
        </CardTitle>
      </CardHeader>
      <CardContent className="flex-1 space-y-4">
        <div className="grid grid-cols-2 gap-4 text-sm">
          <div>
            <div className="font-medium">Size</div>
            <div className="text-muted-foreground">{model.size}</div>
          </div>
          <div>
            <div className="font-medium">Parameters</div>
            <div className="text-muted-foreground">{model.parameters}</div>
          </div>
          <div>
            <div className="font-medium">License</div>
            <div className="text-muted-foreground">{model.license}</div>
          </div>
          <div>
            <div className="font-medium">Quantization</div>
            <div className="text-muted-foreground">{model.quantization}</div>
          </div>
        </div>

        <div className="space-y-2">
          <div className="flex justify-between items-center text-sm">
            <span>Quality</span>
            <RatingStars rating={model.ratings.quality} />
          </div>
          <div className="flex justify-between items-center text-sm">
            <span>Speed</span>
            <RatingStars rating={model.ratings.speed} />
          </div>
          <div className="flex justify-between items-center text-sm">
            <span>Accuracy</span>
            <RatingStars rating={model.ratings.accuracy} />
          </div>
        </div>
      </CardContent>
      <CardFooter className="flex gap-2">
        {model.downloadState?.status === 'pending' ? (
          <>
            <div className="flex-1">
              <div className="h-2 w-full bg-secondary rounded-full overflow-hidden">
                {model.downloadState.progress !== undefined ? (
                  <div
                    className="h-full bg-primary transition-all duration-500"
                    style={{ width: `${model.downloadState.progress}%` }}
                  />
                ) : (
                  <div className="h-full bg-primary animate-progress-infinite" />
                )}
              </div>
              <div className="flex justify-between text-sm text-muted-foreground mt-2">
                <span>
                  {model.downloadState.progress !== undefined && <>{model.downloadState.progress}%</>}
                  {model.downloadState.speed && (
                    <>
                      {model.downloadState.progress !== undefined && ' • '}
                      {model.downloadState.speed}
                    </>
                  )}
                </span>
                {model.downloadState.timeRemaining && <span>{model.downloadState.timeRemaining} remaining</span>}
              </div>
            </div>
            <Button variant="outline" size="sm" disabled>
              Downloading
            </Button>
          </>
        ) : (
          <Button className="w-full" disabled={model.downloadState?.status === 'completed'} onClick={onDownload}>
            <Download className="mr-2 h-4 w-4" />
            {model.downloadState?.status === 'completed' ? 'Download Complete' : 'Download Model'}
          </Button>
        )}
      </CardFooter>
    </Card>
  );
};
