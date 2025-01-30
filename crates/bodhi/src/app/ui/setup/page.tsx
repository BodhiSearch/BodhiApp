'use client';

import { motion } from 'framer-motion';
import { useRouter } from 'next/navigation';
import Image from 'next/image';
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { useSetupApp } from '@/hooks/useQuery';
import AppInitializer from '@/components/AppInitializer';
import { SetupProgress } from '@/components/setup/SetupProgress';
import { BenefitCard } from '@/components/setup/BenefitCard';

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

function SetupContent() {
  const router = useRouter();

  const { mutate: setup, isLoading } = useSetupApp({
    onSuccess: (appInfo) => {
      if (appInfo.status === 'resource-admin') {
        router.push('/ui/setup/resource-admin');
      }
    },
  });

  const handleSetup = (authz: boolean) => {
    setup({ authz });
  };

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
      description:
        'Choose and run any compatible LLM model. Customize settings.',
      icon: '🚀',
    },
    {
      title: 'Local Performance',
      description:
        "Direct access to your hardware's capabilities without latency.",
      icon: '⚡',
    },
    {
      title: 'Wisdom for All',
      description:
        'Experience AI without technical complexity. Simple, intuitive UI unlocks the power of LLM for everyone.',
      icon: '🙏',
    },
    {
      title: 'Solid Foundation',
      description:
        'Built on proven open-source pillars: HuggingFace, llama.cpp.',
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
        'API Tokens (scoped)',
        'Resource usage tracking (coming soon)',
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
        'No authentication or user management',
        'No API tokens',
        'No Resource usage tracking',
        'Unsecure API endpoints',
        'Not compatible with Auth-only mode features',
      ],
      icon: '⚡️',
      recommended: false,
    },
  ];

  const setupRequirements = [
    {
      title: 'Internet',
      description:
        'Reliable internet connection needed throughout setup for downloading models and engines',
      icon: '🌐',
      details:
        'Recommended: High-speed connection for faster downloads (models are typically 4GB+)',
    },
    {
      title: 'Storage',
      description: 'Sufficient disk space for LLMs',
      icon: '💾',
      details: 'Recommended: 5GB free space',
    },
    {
      title: 'Email',
      description: 'For authenticated mode setup',
      icon: '📧',
      details: 'Used to create admin account',
    },
  ];

  return (
    <main className="min-h-screen bg-background p-4 md:p-8">
      <motion.div
        className="mx-auto max-w-4xl space-y-8"
        variants={containerVariants}
        initial="hidden"
        animate="visible"
      >
        {/* Progress Indicator */}
        <SetupProgress currentStep={1} totalSteps={5} />

        {/* Logo */}
        <motion.div variants={itemVariants} className="text-center">
          <Image
            src="/bodhi-logo/bodhi-logo-240.svg"
            alt="Bodhi App Logo"
            width={120}
            height={120}
            className="mx-auto"
            priority
          />
        </motion.div>

        {/* Welcome Card */}
        <motion.div variants={itemVariants}>
          <Card>
            <CardHeader>
              <CardTitle className="text-center text-3xl font-bold">
                Welcome to Bodhi App
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <p className="text-center text-muted-foreground">
                Run AI Models Locally, Privately, and Completely Free
              </p>
              <div className="prose dark:prose-invert mx-auto text-center">
                <p>
                  &quot;Bodhi&quot; (बोधि) comes from ancient Sanskrit/Pali, it
                  means deep wisdom/intelligence, the ultimate goal of every
                  being. <br />
                  We believe the ongoing revolution of Aritificial Intelligence
                  should be the same: <br />
                  private, free, and <strong>
                    accessible to everyone
                  </strong>. <br />
                  <br />
                  Bodhi App is our step towards democratizing AI.
                </p>
              </div>
            </CardContent>
          </Card>
        </motion.div>

        {/* Benefits Grid */}
        <motion.div
          className="grid grid-cols-1 md:grid-cols-2 gap-4"
          variants={itemVariants}
        >
          {benefits.map((benefit) => (
            <BenefitCard key={benefit.title} {...benefit} />
          ))}
        </motion.div>

        {/* Setup Requirements */}
        <motion.div variants={itemVariants}>
          <Card>
            <CardHeader>
              <CardTitle className="text-center">Setup Requirements</CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                {setupRequirements.map((req) => (
                  <div
                    key={req.title}
                    className="p-4 rounded-lg border bg-card text-card-foreground"
                  >
                    <div className="flex items-center gap-2 mb-2">
                      <span className="text-2xl">{req.icon}</span>
                      <h3 className="font-semibold">{req.title}</h3>
                    </div>
                    <p className="text-sm text-muted-foreground mb-2">
                      {req.description}
                    </p>
                    <p className="text-xs text-muted-foreground">
                      {req.details}
                    </p>
                  </div>
                ))}
              </div>
              <div className="text-sm text-muted-foreground text-center">
                <p>
                  Please ensure the above requirements are met before proceeding
                  with the setup.
                  <br />
                  The setup process includes downloading necessary components
                  tailored to your hardware and cannot be completed offline.
                </p>
              </div>
            </CardContent>
          </Card>
        </motion.div>

        {/* Setup Mode Comparison */}
        <motion.div variants={itemVariants}>
          <Card>
            <CardHeader>
              <CardTitle className="text-center">
                Choose Your Setup Mode
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {setupModes.map((mode) => (
                  <div key={mode.title} className="space-y-4">
                    <div className="flex items-center gap-2">
                      <span className="text-2xl">{mode.icon}</span>
                      <div>
                        <h3 className="font-semibold">{mode.title}</h3>
                        <p className="text-sm text-muted-foreground">
                          {mode.description}
                        </p>
                      </div>
                      {mode.recommended && (
                        <span className="ml-auto inline-flex items-center rounded-full bg-primary/10 px-2.5 py-0.5 text-xs font-medium text-primary">
                          Recommended
                        </span>
                      )}
                    </div>
                    <ul className="space-y-2 text-sm">
                      {mode.benefits.map((benefit, index) => (
                        <li key={index} className="flex items-start gap-2">
                          <span className="text-primary">•</span>
                          <span>{benefit}</span>
                        </li>
                      ))}
                    </ul>
                  </div>
                ))}
              </div>

              <div className="pt-6 space-y-4">
                <Button
                  className="w-full"
                  size="lg"
                  onClick={() => handleSetup(true)}
                  disabled={isLoading}
                >
                  Setup Authenticated Instance →
                </Button>
                <div className="relative">
                  <div className="absolute inset-0 flex items-center">
                    <span className="w-full border-t" />
                  </div>
                  <div className="relative flex justify-center text-xs uppercase">
                    <span className="bg-background px-2 text-muted-foreground">
                      Or
                    </span>
                  </div>
                </div>
                <Button
                  variant="outline"
                  className="w-full"
                  size="lg"
                  onClick={() => handleSetup(false)}
                  disabled={isLoading}
                >
                  Setup Unauthenticated Instance →
                </Button>
              </div>
            </CardContent>
            <CardFooter className="flex flex-col gap-4">
              <div className="flex items-center gap-2 p-4 border rounded-lg bg-yellow-500/10 text-yellow-600 dark:text-yellow-400">
                <span className="text-2xl">⚠️</span>
                <div className="text-sm">
                  <p>
                    You cannot switch your choice later. You will need to
                    reinstall the app, losing your data, in the process.
                  </p>
                </div>
              </div>
              <p className="text-sm text-muted-foreground text-center w-full">
                Please review the features carefully and make your choice based
                on your requirements.
              </p>
            </CardFooter>
          </Card>
        </motion.div>
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
