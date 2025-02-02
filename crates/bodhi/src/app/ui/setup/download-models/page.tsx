'use client';

import {
  additionalModels,
  recommendedModels,
} from '@/app/ui/setup/download-models/data';
import { ModelCard } from '@/app/ui/setup/download-models/ModelCard';
import { ModelList } from '@/app/ui/setup/download-models/ModelList';
import { SetupProgress } from '@/app/ui/setup/SetupProgress';
import { containerVariants, itemVariants } from '@/app/ui/setup/types';
import AppInitializer from '@/components/AppInitializer';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { useLocalStorage } from '@/hooks/useLocalStorage';
import { motion } from 'framer-motion';
import { useEffect } from 'react';

function ModelDownloadContent() {
  const [, setHasShownModelsPage] = useLocalStorage(
    'shown-download-models-page',
    true
  );

  useEffect(() => {
    setHasShownModelsPage(false);
  }, [setHasShownModelsPage]);

  return (
    <main className="min-h-screen bg-background">
      <motion.div
        className="mx-auto max-w-4xl space-y-8 p-4 md:p-8"
        variants={containerVariants}
        initial="hidden"
        animate="visible"
      >
        <SetupProgress currentStep={3} totalSteps={4} />

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
              <ModelList additionalModels={additionalModels} />
            </CardContent>
          </Card>
        </motion.div>

        {/* Background Download Notice */}
        <motion.div variants={itemVariants}>
          <Card>
            <CardContent className="py-4">
              <p className="text-sm text-center text-muted-foreground">
                Downloads will continue in the background. You can track
                download progress in the Models section after setup is complete.
              </p>
            </CardContent>
          </Card>
        </motion.div>

        <motion.div variants={itemVariants} className="flex justify-end">
          <Button variant="outline">Continue</Button>
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
