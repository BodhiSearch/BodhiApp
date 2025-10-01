'use client';

import { ModelCard } from '@/app/ui/setup/download-models/ModelCard';
import { ModelInfo, ModelCatalog } from '@/app/ui/setup/download-models/types';
import { SETUP_STEPS, SETUP_STEP_LABELS, SETUP_TOTAL_STEPS } from '@/app/ui/setup/constants';
import { SetupProgress } from '@/app/ui/setup/SetupProgress';
import { containerVariants, itemVariants } from '@/app/ui/setup/types';
import AppInitializer from '@/components/AppInitializer';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { useLocalStorage } from '@/hooks/useLocalStorage';
import { useChatModelsCatalog, useEmbeddingModelsCatalog } from '@/hooks/useModelCatalog';
import { useDownloads, usePullModel } from '@/hooks/useModels';
import { FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED, ROUTE_SETUP_API_MODELS } from '@/lib/constants';
import { motion } from 'framer-motion';
import { useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';
import { BodhiLogo } from '@/app/ui/setup/BodhiLogo';

export function ModelDownloadContent() {
  const router = useRouter();
  const { showSuccess, showError } = useToastMessages();
  const [enablePolling, setEnablePolling] = useState(false);
  const { data: downloads } = useDownloads(1, 100, { enablePolling });
  const { data: chatModels } = useChatModelsCatalog();
  const { data: embeddingModels } = useEmbeddingModelsCatalog();

  const { mutate: pullModel } = usePullModel({
    onSuccess: () => {
      showSuccess('Success', 'Model download started');
    },
    onError: (message) => {
      showError('Error', message);
    },
  });
  const [, setHasShownModelsPage] = useLocalStorage(FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED, true);

  useEffect(() => {
    setHasShownModelsPage(true);
  }, [setHasShownModelsPage]);

  useEffect(() => {
    const hasPendingDownloads = downloads?.data.some((download) => download.status === 'pending') ?? false;
    setEnablePolling(hasPendingDownloads);
  }, [downloads]);

  const handleModelDownload = (model: ModelCatalog) => {
    pullModel({
      repo: model.repo,
      filename: model.filename,
    });
  };

  const getDownloadState = (model: ModelCatalog): ModelInfo => {
    const download = downloads?.data.find((d) => d.repo === model.repo && d.filename === model.filename);
    if (!download) return { ...model, downloadState: { status: 'idle' as const } };

    return {
      ...model,
      downloadState: { status: download.status },
    };
  };

  const getDownloadProgress = (model: ModelCatalog) => {
    const download = downloads?.data.find((d) => d.repo === model.repo && d.filename === model.filename);
    if (!download || download.status !== 'pending') return undefined;

    return {
      downloadedBytes: download.downloaded_bytes ?? null,
      totalBytes: download.total_bytes ?? null,
    };
  };

  return (
    <main className="min-h-screen bg-background">
      <motion.div
        className="mx-auto max-w-7xl space-y-6 p-4 md:p-8"
        variants={containerVariants}
        initial="hidden"
        animate="visible"
      >
        <SetupProgress
          currentStep={SETUP_STEPS.DOWNLOAD_MODELS}
          totalSteps={SETUP_TOTAL_STEPS}
          stepLabels={SETUP_STEP_LABELS}
        />
        <BodhiLogo />

        <motion.div variants={itemVariants}>
          <Card>
            <CardHeader>
              <CardTitle className="text-center text-lg">Chat Models</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                {chatModels.map((model) => {
                  const modelWithState = getDownloadState(model);
                  const downloadProgress = getDownloadProgress(model);
                  return (
                    <ModelCard
                      key={model.id}
                      model={modelWithState}
                      onDownload={() => handleModelDownload(model)}
                      downloadProgress={downloadProgress}
                    />
                  );
                })}
              </div>
            </CardContent>
          </Card>
        </motion.div>

        <motion.div variants={itemVariants}>
          <Card>
            <CardHeader>
              <CardTitle className="text-center text-lg">Embedding Models</CardTitle>
              <p className="text-center text-sm text-muted-foreground">
                For RAG, semantic search, and document retrieval
              </p>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                {embeddingModels.map((model) => {
                  const modelWithState = getDownloadState(model);
                  const downloadProgress = getDownloadProgress(model);
                  return (
                    <ModelCard
                      key={model.id}
                      model={modelWithState}
                      onDownload={() => handleModelDownload(model)}
                      downloadProgress={downloadProgress}
                    />
                  );
                })}
              </div>
            </CardContent>
          </Card>
        </motion.div>

        <motion.div variants={itemVariants}>
          <Card>
            <CardContent className="py-4">
              <p className="text-sm text-center text-muted-foreground">
                Downloads will continue in the background. You can download additional models later on the Models page.
              </p>
            </CardContent>
          </Card>
        </motion.div>

        <motion.div variants={itemVariants} className="flex justify-end">
          <Button data-testid="continue-button" variant="outline" onClick={() => router.push(ROUTE_SETUP_API_MODELS)}>
            Continue
          </Button>
        </motion.div>
      </motion.div>
    </main>
  );
}

export default function ModelDownloadPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <ModelDownloadContent />
    </AppInitializer>
  );
}
