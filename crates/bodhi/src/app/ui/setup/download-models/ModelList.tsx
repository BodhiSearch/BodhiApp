import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { ChevronDown, ChevronUp, Download } from 'lucide-react';
import { ModelInfo } from '@/app/ui/setup/download-models/types';
import { RatingStars } from '@/app/ui/setup/download-models/RatingStars';

interface ModelListProps {
  additionalModels: Record<string, ModelInfo[]>;
}

export const ModelList = ({ additionalModels }: ModelListProps) => {
  const [expandedCategory, setExpandedCategory] = useState<string | null>(null);

  return (
    <div className="space-y-2">
      {Object.entries(additionalModels).map(([category, models]) => (
        <div key={category} className="border-b border-border last:border-0">
          <Button
            variant="ghost"
            className="w-full justify-between py-3 h-auto"
            onClick={() => setExpandedCategory(expandedCategory === category ? null : category)}
          >
            {category}
            {expandedCategory === category ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
          </Button>

          {expandedCategory === category && (
            <div className="space-y-2 p-2">
              {models.map((model) => (
                <div key={model.id} className="flex flex-col gap-2 p-3 rounded-lg bg-card">
                  <div className="flex items-center justify-between">
                    <div>
                      <h4 className="font-medium">{model.name}</h4>
                      <p className="text-xs text-muted-foreground">{model.repo}</p>
                    </div>
                  </div>

                  <div className="grid grid-cols-3 gap-2 text-sm">
                    <div>
                      <div className="text-xs text-muted-foreground">Size</div>
                      <div>{model.size}</div>
                    </div>
                    <div>
                      <div className="text-xs text-muted-foreground">Parameters</div>
                      <div>{model.parameters}</div>
                    </div>
                    <div>
                      <div className="text-xs text-muted-foreground">License</div>
                      <div>{model.license}</div>
                    </div>
                  </div>

                  <div className="flex items-center gap-4">
                    <div className="flex items-center gap-2">
                      <span className="text-xs text-muted-foreground">Quality</span>
                      <RatingStars rating={model.ratings.quality} />
                    </div>
                  </div>

                  <div className="flex justify-center">
                    {model.downloadState?.status === 'pending' ? (
                      <div className="w-full flex gap-2 items-center">
                        <div className="flex-1">
                          <div className="h-2 w-full bg-secondary rounded-full overflow-hidden">
                            <div
                              className="h-full bg-primary transition-all duration-500"
                              style={{
                                width: `${model.downloadState.progress}%`,
                              }}
                            />
                          </div>
                          <div className="flex justify-between text-sm text-muted-foreground mt-2">
                            <span>
                              {model.downloadState.progress}% • {model.downloadState.speed}
                            </span>
                            <span>{model.downloadState.timeRemaining} remaining</span>
                          </div>
                        </div>
                        <Button variant="outline" size="sm">
                          Cancel
                        </Button>
                      </div>
                    ) : (
                      <Button disabled={model.downloadState?.status === 'completed'}>
                        <Download className="mr-2 h-4 w-4" />
                        {model.downloadState?.status === 'completed' ? 'Download Complete' : 'Download'}
                      </Button>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      ))}
    </div>
  );
};
