'use client';

import { recommendedModels } from '@/app/ui/setup/download-models/data';
import { ModelCard } from '@/app/ui/setup/download-models/ModelCard';
import { ModelInfo } from '@/app/ui/setup/download-models/types';
import { SetupProgress } from '../SetupProgress';
import { containerVariants, itemVariants } from '../types';
import AppInitializer from '@/components/AppInitializer';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { useLocalStorage } from '@/hooks/useLocalStorage';
import { useDownloads, usePullModel } from '@/hooks/useQuery';
import { FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED, ROUTE_SETUP_COMPLETE } from '@/lib/constants';
import { motion } from 'framer-motion';
import { useRouter } from '@/lib/navigation';
import { useEffect } from 'react';
import { BodhiLogo } from '../BodhiLogo';

export function ModelDownloadContent() {
  const router = useRouter();
  const { showSuccess, showError } = useToastMessages();
  const { data: downloads } = useDownloads(1, 100);

  const { mutate: pullModel } = usePullModel({
    onSuccess: () => {
      showSuccess('Success', 'Model downloaded successfully');
    },
    onError: (message) => {
      showError('Error', message);
    },
  });
  const [, setHasShownModelsPage] = useLocalStorage(FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED, true);

  useEffect(() => {
    setHasShownModelsPage(true);
  }, [setHasShownModelsPage]);

  const handleModelDownload = (model: ModelInfo) => {
    pullModel({
      repo: model.repo,
      filename: model.filename,
    });
  };

  return (
    <main className="min-h-screen bg-background">
      <motion.div
        className="mx-auto max-w-4xl space-y-8 p-4 md:p-8"
        variants={containerVariants}
        initial="hidden"
        animate="visible"
      >
        <SetupProgress currentStep={3} totalSteps={4} />
        <BodhiLogo />

        <motion.div variants={itemVariants}>
          <Card>
            <CardHeader>
              <CardTitle className="text-center">Recommended Models</CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                {recommendedModels.map((model) => {
                  const status =
                    downloads?.data.find((d) => d.repo === model.repo && d.filename === model.filename)?.status ||
                    'idle';
                  model.downloadState = { status };
                  return <ModelCard key={model.id} model={model} onDownload={() => handleModelDownload(model)} />;
                })}
              </div>
            </CardContent>
          </Card>
        </motion.div>

        {/* Background Download Notice */}
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
          <Button variant="outline" onClick={() => router.push(ROUTE_SETUP_COMPLETE)}>
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
