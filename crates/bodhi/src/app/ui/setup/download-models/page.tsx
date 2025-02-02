'use client';

import { motion } from 'framer-motion';
import { useState } from 'react';
import { SetupProgress } from '@/components/setup/SetupProgress';
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardFooter,
} from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { ChevronDown, ChevronUp, Download, Star } from 'lucide-react';

// Animation variants remain same as other pages
const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: { staggerChildren: 0.1 },
  },
};

const itemVariants = {
  hidden: { y: 20, opacity: 0 },
  visible: { y: 0, opacity: 1 },
};

// Types
interface ModelInfo {
  id: string;
  name: string;
  repo: string;
  fileName: string;
  quantization: string;
  size: string;
  parameters: string;
  leaderboardRank: number;
  category: 'small' | 'medium' | 'large' | 'extra-large';
  ratings: {
    quality: number;
    speed: number;
    accuracy: number;
  };
  license: string;
  downloadState?: {
    status: 'idle' | 'downloading' | 'complete' | 'error';
    progress?: number;
    speed?: string;
    timeRemaining?: string;
  };
}

// Stub data
const recommendedModels: ModelInfo[] = [
  {
    id: 'mistral-7b',
    name: 'Mistral-7B',
    repo: 'mistralai/Mistral-7B-v0.1',
    fileName: 'mistral-7b-q4_K_M.gguf',
    quantization: 'Q4_K_M',
    size: '4.1GB',
    parameters: '7B',
    leaderboardRank: 3,
    category: 'medium',
    ratings: { quality: 4.5, speed: 4, accuracy: 4.5 },
    license: 'Apache 2.0',
    downloadState: { status: 'idle' },
  },
  {
    id: 'phi2',
    name: 'Phi-2',
    repo: 'microsoft/phi-2',
    fileName: 'phi-2-q4_K_M.gguf',
    quantization: 'Q4_K_M',
    size: '2.1GB',
    parameters: '2.7B',
    leaderboardRank: 5,
    category: 'small',
    ratings: { quality: 4, speed: 5, accuracy: 4 },
    license: 'MIT',
    downloadState: {
      status: 'downloading',
      progress: 45,
      speed: '10.5 MB/s',
      timeRemaining: '5 minutes',
    },
  },
  {
    id: 'neural-7b',
    name: 'Neural-7B',
    repo: 'neural/neural-7b',
    fileName: 'neural-7b-q4_K_M.gguf',
    quantization: 'Q4_K_M',
    size: '3.9GB',
    parameters: '7B',
    leaderboardRank: 7,
    category: 'medium',
    ratings: { quality: 4, speed: 4, accuracy: 4 },
    license: 'Apache 2.0',
    downloadState: { status: 'complete' },
  },
];

// Additional models grouped by size
const additionalModels: Record<string, ModelInfo[]> = {
  'Small Models (1-3B)': [
    {
      id: 'phi-1.5',
      name: 'Phi-1.5',
      repo: 'microsoft/phi-1.5',
      fileName: 'phi-1.5-q4_K_M.gguf',
      quantization: 'Q4_K_M',
      size: '1.8GB',
      parameters: '1.3B',
      leaderboardRank: 12,
      category: 'small',
      ratings: { quality: 3.5, speed: 5, accuracy: 3.5 },
      license: 'MIT',
      downloadState: { status: 'idle' },
    },
    {
      id: 'tinyllama',
      name: 'TinyLlama',
      repo: 'TinyLlama/TinyLlama-1.1B',
      fileName: 'tinyllama-1.1b-q4_K_M.gguf',
      quantization: 'Q4_K_M',
      size: '0.6GB',
      parameters: '1.1B',
      leaderboardRank: 15,
      category: 'small',
      ratings: { quality: 3, speed: 5, accuracy: 3 },
      license: 'Apache 2.0',
      downloadState: { status: 'idle' },
    },
  ],
  'Medium Models (4-7B)': [
    {
      id: 'openchat-3.5',
      name: 'OpenChat 3.5',
      repo: 'openchat/openchat-3.5',
      fileName: 'openchat-3.5-q4_K_M.gguf',
      quantization: 'Q4_K_M',
      size: '4.3GB',
      parameters: '7B',
      leaderboardRank: 4,
      category: 'medium',
      ratings: { quality: 4.5, speed: 4, accuracy: 4.5 },
      license: 'Apache 2.0',
      downloadState: { status: 'idle' },
    },
    {
      id: 'stable-beluga',
      name: 'StableBeluga 7B',
      repo: 'stabilityai/stable-beluga-7b',
      fileName: 'stable-beluga-7b-q4_K_M.gguf',
      quantization: 'Q4_K_M',
      size: '4.1GB',
      parameters: '7B',
      leaderboardRank: 8,
      category: 'medium',
      ratings: { quality: 4, speed: 4, accuracy: 4 },
      license: 'Apache 2.0',
      downloadState: {
        status: 'downloading',
        progress: 23,
        speed: '8.2 MB/s',
        timeRemaining: '8 minutes',
      },
    },
  ],
  'Large Models (8-13B)': [
    {
      id: 'mixtral-8x7b',
      name: 'Mixtral 8x7B',
      repo: 'mistralai/Mixtral-8x7B',
      fileName: 'mixtral-8x7b-q4_K_M.gguf',
      quantization: 'Q4_K_M',
      size: '26GB',
      parameters: '47B',
      leaderboardRank: 1,
      category: 'large',
      ratings: { quality: 5, speed: 3, accuracy: 5 },
      license: 'Apache 2.0',
      downloadState: { status: 'idle' },
    },
    {
      id: 'llama2-13b',
      name: 'Llama 2 13B',
      repo: 'meta-llama/Llama-2-13b',
      fileName: 'llama-2-13b-q4_K_M.gguf',
      quantization: 'Q4_K_M',
      size: '7.5GB',
      parameters: '13B',
      leaderboardRank: 6,
      category: 'large',
      ratings: { quality: 4.5, speed: 3.5, accuracy: 4.5 },
      license: 'Meta License',
      downloadState: { status: 'complete' },
    },
  ],
};

// Components
function RatingStars({ rating }: { rating: number }) {
  return (
    <div className="flex items-center gap-1">
      {[1, 2, 3, 4, 5].map((star) => (
        <Star
          key={star}
          className={`h-4 w-4 ${
            star <= rating
              ? 'fill-primary text-primary'
              : 'fill-muted text-muted-foreground'
          }`}
        />
      ))}
    </div>
  );
}

function ModelCard({ model }: { model: ModelInfo }) {
  return (
    <Card className="h-full flex flex-col">
      <CardHeader>
        <CardTitle className="text-lg flex justify-between items-start">
          <div>
            {model.name}
            <div className="text-sm font-normal text-muted-foreground">
              {model.repo}
            </div>
          </div>
          <span className="text-sm text-muted-foreground">
            #{model.leaderboardRank}
          </span>
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
        {model.downloadState?.status === 'downloading' ? (
          <>
            <div className="flex-1">
              <div className="h-2 w-full bg-secondary rounded-full overflow-hidden">
                <div
                  className="h-full bg-primary transition-all duration-500"
                  style={{ width: `${model.downloadState.progress}%` }}
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
          </>
        ) : (
          <Button
            className="w-full"
            disabled={model.downloadState?.status === 'complete'}
          >
            <Download className="mr-2 h-4 w-4" />
            {model.downloadState?.status === 'complete'
              ? 'Download Complete'
              : 'Download Model'}
          </Button>
        )}
      </CardFooter>
    </Card>
  );
}

function ModelList() {
  const [expandedCategory, setExpandedCategory] = useState<string | null>(null);

  return (
    <div className="space-y-2">
      {Object.entries(additionalModels).map(([category, models]) => (
        <div key={category} className="border-b border-border last:border-0">
          <Button
            variant="ghost"
            className="w-full justify-between py-3 h-auto"
            onClick={() =>
              setExpandedCategory(
                expandedCategory === category ? null : category
              )
            }
          >
            {category}
            {expandedCategory === category ? (
              <ChevronUp className="h-4 w-4" />
            ) : (
              <ChevronDown className="h-4 w-4" />
            )}
          </Button>

          {expandedCategory === category && (
            <div className="space-y-2 p-2">
              {models.map((model) => (
                <div
                  key={model.id}
                  className="flex flex-col gap-2 p-3 rounded-lg bg-card"
                >
                  {/* Model Name and Rank */}
                  <div className="flex items-center justify-between">
                    <div>
                      <h4 className="font-medium">{model.name}</h4>
                      <p className="text-xs text-muted-foreground">
                        {model.repo}
                      </p>
                    </div>
                    <span className="text-sm text-muted-foreground">
                      #{model.leaderboardRank}
                    </span>
                  </div>

                  {/* Model Details */}
                  <div className="grid grid-cols-3 gap-2 text-sm">
                    <div>
                      <div className="text-xs text-muted-foreground">Size</div>
                      <div>{model.size}</div>
                    </div>
                    <div>
                      <div className="text-xs text-muted-foreground">
                        Parameters
                      </div>
                      <div>{model.parameters}</div>
                    </div>
                    <div>
                      <div className="text-xs text-muted-foreground">
                        License
                      </div>
                      <div>{model.license}</div>
                    </div>
                  </div>

                  {/* Ratings */}
                  <div className="flex items-center gap-4">
                    <div className="flex items-center gap-2">
                      <span className="text-xs text-muted-foreground">
                        Quality
                      </span>
                      <RatingStars rating={model.ratings.quality} />
                    </div>
                  </div>

                  {/* Download Button and Status */}
                  <div className="flex justify-center">
                    {model.downloadState?.status === 'downloading' ? (
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
                              {model.downloadState.progress}% •{' '}
                              {model.downloadState.speed}
                            </span>
                            <span>
                              {model.downloadState.timeRemaining} remaining
                            </span>
                          </div>
                        </div>
                        <Button variant="outline" size="sm">
                          Cancel
                        </Button>
                      </div>
                    ) : (
                      <Button
                        disabled={model.downloadState?.status === 'complete'}
                      >
                        <Download className="mr-2 h-4 w-4" />
                        {model.downloadState?.status === 'complete'
                          ? 'Download Complete'
                          : 'Download'}
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
}

function ModelDownloadContent() {
  return (
    <motion.div
      className="mx-auto max-w-4xl space-y-8 p-4 md:p-8"
      variants={containerVariants}
      initial="hidden"
      animate="visible"
    >
      <SetupProgress currentStep={4} totalSteps={5} />

      {/* Recommended Models */}
      <motion.div variants={itemVariants}>
        <Card>
          <CardHeader>
            <CardTitle className="text-center">Recommended Models</CardTitle>
          </CardHeader>
          <CardContent className="space-y-6">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              {recommendedModels.map((model) => (
                <ModelCard key={model.id} model={model} />
              ))}
            </div>
          </CardContent>
        </Card>
      </motion.div>

      {/* Additional Models */}
      <motion.div variants={itemVariants}>
        <Card>
          <CardHeader>
            <CardTitle className="text-center">Additional Models</CardTitle>
          </CardHeader>
          <CardContent>
            <ModelList />
          </CardContent>
        </Card>
      </motion.div>

      {/* Background Download Notice */}
      <motion.div variants={itemVariants}>
        <Card>
          <CardContent className="py-4">
            <p className="text-sm text-center text-muted-foreground">
              Downloads will continue in the background. You can track download
              progress in the Models section after setup is complete.
            </p>
          </CardContent>
        </Card>
      </motion.div>

      <motion.div variants={itemVariants} className="flex justify-end">
        <Button variant="outline">Continue</Button>
      </motion.div>
    </motion.div>
  );
}

export default function ModelDownloadPage() {
  return (
    <main className="min-h-screen bg-background">
      <ModelDownloadContent />
    </main>
  );
}
