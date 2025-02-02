'use client';

import { motion } from 'framer-motion';
import { useRouter } from 'next/navigation';
import { useSetupApp } from '@/hooks/useQuery';
import AppInitializer from '@/components/AppInitializer';
import { SetupProgress } from '@/app/ui/setup/SetupProgress';
import { BenefitCard } from '@/app/ui/setup/BenefitCard';
import { WelcomeCard } from '@/app/ui/setup/WelcomeCard';
import { SetupModeCard } from '@/app/ui/setup/SetupModeCard';
import { containerVariants, itemVariants } from '@/app/ui/setup/types';
import { useEffect } from 'react';
import { useLocalStorage } from '@/hooks/useLocalStorage';

const benefits = [
  {
    title: 'Complete Privacy',
    description:
      'Your chat data stays on your device. No data sharing/transfer out of your device.',
    icon: '🔒',
  },
  {
    title: 'Always Free',
    description:
      'Run unlimited AI inferences locally without usage fees or API costs.',
    icon: '💰',
  },
  {
    title: 'Full Control',
    description: 'Choose and run any compatible LLM model. Customize settings.',
    icon: '🚀',
  },
  {
    title: 'Local Performance',
    description:
      "Direct access to your hardware's capabilities without latency.",
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
    description:
      'Leverages open-source eco-system: HuggingFace, llama.cpp, etc.',
    icon: '🌟',
  },
];

const setupModes = [
  {
    title: 'Authenticated Mode',
    description: 'Recommended',
    benefits: [
      'User authentication',
      'Multi-user support with RBAC',
      'Secure API endpoints',
      'API Tokens',
      'Resource usage tracking (coming soon)',
      'User/token level usage quotas (coming soon)',
      'Compatible with future Auth-only mode features',
    ],
    icon: '🔐',
    recommended: true,
  },
  {
    title: 'Non-Authenticated Mode',
    description: 'For quick/minimal setup',
    benefits: [
      'Quick setup',
      'No email/login required',
      'No authentication or user management feature',
      'No API tokens',
      'No user/token-wise usage tracking',
      'Public/insecure API endpoints',
      'Not compatible with Auth-only mode features',
    ],
    icon: '⚡️',
    recommended: false,
  },
];

function SetupContent() {
  const router = useRouter();
  const [, setHasShownModelsPage] = useLocalStorage(
    'shown-download-models-page',
    true
  );

  useEffect(() => {
    setHasShownModelsPage(false);
  }, [setHasShownModelsPage]);

  const { mutate: setup, isLoading } = useSetupApp({
    onSuccess: (appInfo) => {
      if (appInfo.status === 'resource-admin') {
        router.push('/ui/setup/resource-admin');
      } else {
        router.push('/ui/setup/download-models');
      }
    },
  });

  const handleSetup = (authz: boolean) => {
    setup({ authz });
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

        <motion.div
          className="grid grid-cols-1 md:grid-cols-2 gap-4"
          variants={itemVariants}
        >
          {benefits.map((benefit) => (
            <BenefitCard key={benefit.title} {...benefit} />
          ))}
        </motion.div>

        <SetupModeCard
          setupModes={setupModes}
          onSetup={handleSetup}
          isLoading={isLoading}
        />
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
