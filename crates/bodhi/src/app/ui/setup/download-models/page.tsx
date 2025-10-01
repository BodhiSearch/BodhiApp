'use client';

import { ModelCard } from '@/app/ui/setup/download-models/ModelCard';
import { ModelInfo, ModelCatalog } from '@/app/ui/setup/download-models/types';
import { SetupContainer, SetupCard, SetupFooter } from '@/app/ui/setup/components';
import AppInitializer from '@/components/AppInitializer';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { useLocalStorage } from '@/hooks/useLocalStorage';
import { useChatModelsCatalog, useEmbeddingModelsCatalog } from '@/hooks/useModelCatalog';
import { useDownloads, usePullModel } from '@/hooks/useModels';
import { FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED, ROUTE_SETUP_API_MODELS } from '@/lib/constants';
import { motion } from 'framer-motion';
import { useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';

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
    <SetupContainer>
      <SetupCard title="Chat Models">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
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
      </SetupCard>

      <SetupCard title="Embedding Models" description="For RAG, semantic search, and document retrieval">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
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
      </SetupCard>

      <SetupFooter
        clarificationText="Downloads will continue in the background. You can download additional models later on the Models page."
        onContinue={() => router.push(ROUTE_SETUP_API_MODELS)}
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
