'use client';

import { BenefitCard } from '@/app/ui/setup/BenefitCard';
import { SetupModeCard } from '@/app/ui/setup/SetupModeCard';
import { SetupProgress } from '@/app/ui/setup/SetupProgress';
import { containerVariants, itemVariants } from '@/app/ui/setup/types';
import { WelcomeCard } from '@/app/ui/setup/WelcomeCard';
import AppInitializer from '@/components/AppInitializer';
import { useToastMessages } from '@/hooks/use-toast-messages';
import { useLocalStorage } from '@/hooks/useLocalStorage';
import { useSetupApp } from '@/hooks/useQuery';
import {
  FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED,
  ROUTE_SETUP_DOWNLOAD_MODELS,
  ROUTE_SETUP_RESOURCE_ADMIN,
} from '@/lib/constants';
import { motion } from 'framer-motion';
import { useRouter } from 'next/navigation';
import { useEffect } from 'react';

const benefits = [
  {
    title: 'Complete Privacy',
    description: 'Your chat data stays on your device. No data sharing/transfer out of your device.',
    icon: '🔒',
  },
  {
    title: 'Always Free',
    description: 'Run unlimited AI inferences locally without usage fees or API costs.',
    icon: '💰',
  },
  {
    title: 'Full Control',
    description: 'Choose and run any compatible LLM model. Customize settings.',
    icon: '🚀',
  },
  {
    title: 'Local Performance',
    description: "Direct access to your hardware's capabilities without latency.",
    icon: '⚡',
  },
  {
    title: 'AI for Everyone',
    description:
      'Experience AI without technical complexity. Simple, intuitive UI unlocks the power of LLM for everyone.',
    icon: '🙏',
  },
  {
    title: 'Solid Foundation',
    description: 'Leverages open-source eco-system: HuggingFace, llama.cpp, etc.',
    icon: '🌟',
  },
];

function SetupContent() {
  const router = useRouter();
  const { showError } = useToastMessages();
  const [, setHasShownModelsPage] = useLocalStorage(FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED, false);

  useEffect(() => {
    setHasShownModelsPage(false);
  }, [setHasShownModelsPage]);

  const { mutate: setup, isLoading } = useSetupApp({
    onSuccess: (appInfo) => {
      if (appInfo.status === 'resource-admin') {
        router.push(ROUTE_SETUP_RESOURCE_ADMIN);
      } else {
        router.push(ROUTE_SETUP_DOWNLOAD_MODELS);
      }
    },
    onError: (error) => {
      showError('Error', error);
    },
  });

  const handleSetup = () => {
    setup({});
  };

  return (
    <main className="min-h-screen bg-background p-4 md:p-8">
      <motion.div
        className="mx-auto max-w-4xl space-y-8"
        variants={containerVariants}
        initial="hidden"
        animate="visible"
      >
        <SetupProgress currentStep={1} totalSteps={4} />
        <WelcomeCard />

        <motion.div className="grid grid-cols-1 md:grid-cols-2 gap-4" variants={itemVariants}>
          {benefits.map((benefit) => (
            <BenefitCard key={benefit.title} {...benefit} />
          ))}
        </motion.div>

        <SetupModeCard onSetup={handleSetup} isLoading={isLoading} />
      </motion.div>
    </main>
  );
}

export default function SetupPage() {
  return (
    <AppInitializer allowedStatus="setup" authenticated={false}>
      <SetupContent />
    </AppInitializer>
  );
}
