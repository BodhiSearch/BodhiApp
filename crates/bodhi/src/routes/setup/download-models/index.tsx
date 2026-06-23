import { useEffect, useState } from 'react';

import { useNavigate } from '@tanstack/react-router';
import { createFileRoute } from '@tanstack/react-router';
import { motion } from 'framer-motion';

import AppInitializer from '@/components/AppInitializer';
import { useChatModelsCatalog, useEmbeddingModelsCatalog, useListDownloads, usePullModel } from '@/hooks/models';
import { ModelInfo, ModelCatalog } from '@/hooks/models/model-catalog-types';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { ROUTE_SETUP_API_MODELS } from '@/lib/constants';
import { SetupContainer, SetupFooter } from '@/routes/setup/-components';
import { itemVariants } from '@/routes/setup/-shared/types';

import { ModelCard } from './-components/ModelCard';

export const Route = createFileRoute('/setup/download-models/')({
  component: ModelDownloadPage,
});

export function ModelDownloadContent() {
  const navigate = useNavigate();
  const { showSuccess, showError } = useToastMessages();
  const [enablePolling, setEnablePolling] = useState(false);
  const { data: downloads } = useListDownloads(1, 100, { enablePolling });
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

  const renderGrid = (models: ModelCatalog[]) => (
    <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
      {models.map((model) => {
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
  );

  return (
    <SetupContainer wide>
      <motion.section variants={itemVariants}>
        <div className="mb-6 text-center">
          <h2 className="text-2xl font-bold tracking-tight">Chat Models</h2>
          <p className="mt-1.5 text-sm text-muted-foreground">
            For conversations, content generation, summarization, and Q&amp;A.
          </p>
        </div>
        {renderGrid(chatModels)}
      </motion.section>

      <motion.section variants={itemVariants} className="mt-10">
        <div className="mb-6 text-center">
          <h2 className="text-2xl font-bold tracking-tight">Embedding Models</h2>
          <p className="mt-1.5 text-sm text-muted-foreground">For RAG, semantic search, and document retrieval.</p>
        </div>
        {renderGrid(embeddingModels)}
      </motion.section>

      <SetupFooter
        clarificationText="Downloads will continue in the background. You can download additional models later on the Models page."
        onContinue={() => navigate({ to: ROUTE_SETUP_API_MODELS })}
        buttonTestId="continue-button"
      />
    </SetupContainer>
  );
}

export default function ModelDownloadPage() {
  return (
    <AppInitializer allowedStatus="ready" authenticated={true}>
      <ModelDownloadContent />
    </AppInitializer>
  );
}
