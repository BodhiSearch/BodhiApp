'use client';

import { useState } from 'react';

import { motion } from 'framer-motion';
import { ChevronDown, ChevronUp, CheckCircle2, XCircle, Download } from 'lucide-react';

import { SetupProgress } from '@/app/ui/setup/SetupProgress';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle, CardFooter } from '@/components/ui/card';

// Animation variants
const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.1,
    },
  },
};

const itemVariants = {
  hidden: { y: 20, opacity: 0 },
  visible: {
    y: 0,
    opacity: 1,
  },
};

// Types
interface HardwareInfo {
  os: string;
  gpu: string;
  cpu: string;
  ram: string;
  technicalDetails: {
    gpuDriver: string;
    cpuExtensions: string[];
    gpuCompute: boolean;
    architecture: string;
  };
}

interface EngineOption {
  id: string;
  name: string;
  description: string;
  downloadUrl: string;
  compatible: boolean;
  size: string;
}

// Stub data
const stubHardware: HardwareInfo = {
  os: 'Windows 11 Pro',
  gpu: 'NVIDIA RTX 4080 (12GB)',
  cpu: 'Intel i7 (AVX2 Support)',
  ram: '32GB Available',
  technicalDetails: {
    gpuDriver: 'CUDA 11.8',
    cpuExtensions: ['AVX', 'AVX2', 'SSE4.1', 'SSE4.2'],
    gpuCompute: true,
    architecture: 'x86_64',
  },
};

// Expanded stub data to show different states
const stubEnginesWithStates: (EngineOption & {
  state: 'idle' | 'downloading' | 'complete' | 'error';
  progress?: number;
  error?: string;
})[] = [
  {
    id: 'cuda-opt',
    name: 'CUDA-Optimized Engine',
    description: 'Optimal for NVIDIA GPUs with CUDA support. Best performance for your hardware.',
    downloadUrl: '/api/download/cuda-opt',
    compatible: true,
    size: '85MB',
    state: 'idle',
  },
  {
    id: 'cpu-gpu',
    name: 'CPU+GPU Hybrid',
    description: 'Balanced performance using both CPU and GPU resources.',
    downloadUrl: '/api/download/cpu-gpu',
    compatible: true,
    size: '75MB',
    state: 'downloading',
    progress: 45,
  },
  {
    id: 'cpu-only',
    name: 'CPU-Optimized',
    description: 'Optimized for modern CPUs with AVX2 support.',
    downloadUrl: '/api/download/cpu-opt',
    compatible: true,
    size: '60MB',
    state: 'complete',
  },
];

// Additional engines for expanded list
const additionalEngines: EngineOption[] = [
  {
    id: 'vulkan-opt',
    name: 'Vulkan-Optimized',
    description: 'For GPUs with Vulkan support.',
    downloadUrl: '/api/download/vulkan',
    compatible: false,
    size: '82MB',
  },
  {
    id: 'rocm-opt',
    name: 'ROCm-Optimized',
    description: 'For AMD GPUs with ROCm support.',
    downloadUrl: '/api/download/rocm',
    compatible: true,
    size: '78MB',
  },
];

function DownloadButton({
  engine,
  onDownload,
  fullWidth = false,
}: {
  engine: EngineOption;
  onDownload: () => void;
  fullWidth?: boolean;
}) {
  return (
    <Button className={fullWidth ? 'w-full mt-4' : 'min-w-[120px]'} onClick={onDownload}>
      <Download className="mr-2 h-4 w-4" />
      Download {engine.size}
    </Button>
  );
}

function DownloadProgress({ progress }: { progress: number }) {
  return (
    <div className="space-y-2">
      <div className="h-2 w-full bg-secondary rounded-full overflow-hidden">
        <div className="h-full bg-primary transition-all duration-500" style={{ width: `${progress}%` }} />
      </div>
      <div className="flex justify-between text-sm text-muted-foreground">
        <span>Downloading... {progress}%</span>
        <Button variant="ghost" size="sm">
          Cancel
        </Button>
      </div>
    </div>
  );
}

function DownloadComplete() {
  return (
    <div className="flex items-center gap-2 text-green-500">
      <CheckCircle2 className="h-5 w-5" />
      <span>Download Complete</span>
    </div>
  );
}

function DownloadError({ error, onRetry }: { error: string; onRetry: () => void }) {
  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2 text-destructive">
        <XCircle className="h-5 w-5" />
        <span>{error}</span>
      </div>
      <Button variant="outline" size="sm" onClick={onRetry}>
        Retry Download
      </Button>
    </div>
  );
}

function CompatibilityBadge({ compatible }: { compatible: boolean }) {
  return (
    <Badge variant={compatible ? 'green' : 'destructive'} className="ml-2">
      {compatible ? 'Compatible' : 'Not Compatible'}
    </Badge>
  );
}

function EngineCard({
  engine,
  isSelected,
  onSelect,
  downloadState,
}: {
  engine: EngineOption;
  isSelected: boolean;
  onSelect: () => void;
  downloadState: {
    status: 'idle' | 'downloading' | 'complete' | 'error';
    progress?: number;
    error?: string;
  };
}) {
  return (
    <Card
      className={`cursor-pointer transition-colors h-full flex flex-col ${isSelected ? 'border-primary' : ''}`}
      onClick={onSelect}
    >
      <CardHeader>
        <CardTitle className="text-lg">{engine.name}</CardTitle>
      </CardHeader>
      <CardContent className="flex-1 flex flex-col">
        <p className="text-sm text-muted-foreground flex-1">{engine.description}</p>
        <div className="mt-4">
          {downloadState.status === 'idle' && <DownloadButton engine={engine} onDownload={() => {}} fullWidth />}
          {downloadState.status === 'downloading' && <DownloadProgress progress={downloadState.progress || 0} />}
          {downloadState.status === 'complete' && <DownloadComplete />}
          {downloadState.status === 'error' && (
            <DownloadError error={downloadState.error || 'Download failed'} onRetry={() => {}} />
          )}
        </div>
      </CardContent>
    </Card>
  );
}

function LLMEngineContent() {
  const [showTechnicalDetails, setShowTechnicalDetails] = useState(false);
  const [selectedEngine, setSelectedEngine] = useState<EngineOption>(stubEnginesWithStates[0]);
  const [, setDownloadState] = useState<{
    status: 'idle' | 'downloading' | 'complete' | 'error';
    progress?: number;
    error?: string;
  }>({ status: 'idle' });
  const [showAllEngines, setShowAllEngines] = useState(false);

  const handleDownload = () => {
    setDownloadState({ status: 'downloading', progress: 0 });
    // Simulate download progress
    let progress = 0;
    const interval = setInterval(() => {
      progress += 10;
      if (progress > 100) {
        clearInterval(interval);
        setDownloadState({ status: 'complete' });
      } else {
        setDownloadState({ status: 'downloading', progress });
      }
    }, 500);
  };

  return (
    <motion.div
      className="mx-auto max-w-4xl space-y-8 p-4 md:p-8"
      variants={containerVariants}
      initial="hidden"
      animate="visible"
    >
      <SetupProgress currentStep={3} totalSteps={4} />

      {/* Hardware Analysis Card */}
      <motion.div variants={itemVariants}>
        <Card>
          <CardHeader>
            <CardTitle className="text-center">Hardware Analysis</CardTitle>
          </CardHeader>
          <CardContent className="space-y-6">
            {/* Add recommendation message */}
            <div className="text-center text-muted-foreground mb-4">
              <p>Based on hardware analysis, we have picked the most suitable LLM engine for optimal performance.</p>
              <p className="text-sm mt-2">
                You can choose from our recommendations or explore other available engines.
              </p>
            </div>

            {/* Basic Info */}
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              {Object.entries(stubHardware)
                .filter(([key]) => key !== 'technicalDetails')
                .map(([key, value]) => (
                  <div key={key} className="space-y-1">
                    <div className="text-sm font-medium">{key.toUpperCase()}</div>
                    <div className="text-sm text-muted-foreground">{value}</div>
                  </div>
                ))}
            </div>

            {/* Technical Details */}
            <div className="space-y-4">
              {/* Mobile Toggle Button */}
              <Button
                variant="ghost"
                className="w-full justify-between md:hidden"
                onClick={() => setShowTechnicalDetails(!showTechnicalDetails)}
              >
                Technical Details
                {showTechnicalDetails ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
              </Button>

              {/* Technical Details Content - Always visible on md+ screens */}
              <div className={`space-y-2 text-sm ${showTechnicalDetails ? 'block' : 'hidden'} md:block`}>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  {Object.entries(stubHardware.technicalDetails).map(([key, value]) => (
                    <div key={key} className="space-y-1">
                      <div className="font-medium">{key}</div>
                      <div className="text-muted-foreground">
                        {Array.isArray(value) ? value.join(', ') : value.toString()}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      </motion.div>

      {/* Engine Selection Card */}
      <motion.div variants={itemVariants}>
        <Card>
          <CardHeader>
            <CardTitle className="text-center">Select LLM Engine</CardTitle>
          </CardHeader>
          <CardContent className="space-y-6">
            {/* Add recommended label */}
            <div className="text-sm font-medium text-muted-foreground mb-2">Recommended for Your Hardware</div>

            {/* Recommended Engines */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              {stubEnginesWithStates.map((engine) => (
                <EngineCard
                  key={engine.id}
                  engine={engine}
                  isSelected={selectedEngine.id === engine.id}
                  onSelect={() => setSelectedEngine(engine)}
                  downloadState={{
                    status: engine.state,
                    progress: engine.progress,
                    error: engine.error,
                  }}
                />
              ))}
            </div>

            {/* Update Show All Engines Section */}
            <div className="space-y-4">
              <Button
                variant="ghost"
                className="w-full justify-between"
                onClick={() => setShowAllEngines(!showAllEngines)}
              >
                Show All Available Engines
                {showAllEngines ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
              </Button>

              {showAllEngines && (
                <div className="space-y-2">
                  {additionalEngines.map((engine) => (
                    <div key={engine.id} className="flex items-center gap-4 p-4 border rounded-lg">
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center">
                          <h4 className="font-medium truncate">{engine.name}</h4>
                          <CompatibilityBadge compatible={engine.compatible} />
                        </div>
                        <p className="text-sm text-muted-foreground mt-1">{engine.description}</p>
                        {!engine.compatible && (
                          <p className="text-xs text-destructive mt-1">
                            Warning: This engine may not perform optimally on your hardware
                          </p>
                        )}
                      </div>
                      <div className="flex-shrink-0">
                        <DownloadButton engine={engine} onDownload={handleDownload} />
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </CardContent>
          <CardFooter className="flex justify-end">
            <Button variant="outline">Continue</Button>
          </CardFooter>
        </Card>
      </motion.div>
    </motion.div>
  );
}

export default function LLMEnginePage() {
  return (
    <main className="min-h-screen bg-background">
      <LLMEngineContent />
    </main>
  );
}
