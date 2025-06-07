import { BenefitCard } from '@/components/setup/BenefitCard';
import { SetupModeCard } from '@/components/setup/SetupModeCard';
import { SetupProgress } from '@/components/setup/SetupProgress';
import { containerVariants, itemVariants } from '@/components/setup/types';
import { WelcomeCard } from '@/components/setup/WelcomeCard';
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
import { useRouter } from '@/lib/navigation';
import { useEffect } from 'react';

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
  const { showError } = useToastMessages();
  const [, setHasShownModelsPage] = useLocalStorage(
    FLAG_MODELS_DOWNLOAD_PAGE_DISPLAYED,
    false
  );

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
